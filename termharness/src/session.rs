use std::{
    io::Read,
    io::Write,
    sync::{Arc, Mutex},
    thread,
    thread::JoinHandle,
};

use anyhow::Result;
use portable_pty::{Child, CommandBuilder, MasterPty, PtySize, native_pty_system};

use crate::terminal::TerminalSize;

pub struct Session {
    pub child: Box<dyn Child + Send + Sync>,
    pub master: Box<dyn MasterPty + Send>,
    pub writer: Box<dyn Write + Send>,
    pub output: Arc<Mutex<Vec<u8>>>,
    pub reader_thread: Option<JoinHandle<()>>,
    pub size: TerminalSize,
}

impl Session {
    /// Spawn a new session by executing the given command in a pseudo-terminal with the specified size.
    pub fn spawn(mut cmd: CommandBuilder, size: TerminalSize) -> Result<Self> {
        let pty = native_pty_system();
        let pair = pty.openpty(PtySize {
            rows: size.rows,
            cols: size.cols,
            pixel_width: 0,
            pixel_height: 0,
        })?;

        // Set the TERM environment variable to ensure consistent terminal behavior.
        // Considaration: This should ideally be configurable,
        // but for now we hardcode it to ensure tests run reliably.
        cmd.env("TERM", "xterm-256color");
        let child = pair.slave.spawn_command(cmd)?;
        drop(pair.slave);

        let master = pair.master;
        let output = Arc::new(Mutex::new(Vec::new()));
        let output_reader = Arc::clone(&output);
        let mut reader = master.try_clone_reader()?;
        let reader_thread = thread::spawn(move || {
            let mut buf = [0_u8; 4096];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => output_reader
                        .lock()
                        .expect("failed to lock output buffer")
                        .extend_from_slice(&buf[..n]),
                    Err(err) if err.kind() == std::io::ErrorKind::Interrupted => continue,
                    Err(_) => break,
                }
            }
        });

        let writer = master.take_writer()?;
        Ok(Self {
            child,
            master,
            writer,
            output,
            reader_thread: Some(reader_thread),
            size,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod session {
        use super::*;

        mod spawn {
            use super::*;

            #[test]
            fn success() -> Result<()> {
                let mut cmd = CommandBuilder::new("echo");
                cmd.arg("Hello, world!");
                let mut session = Session::spawn(cmd, TerminalSize::new(24, 80))?;

                // Wait for the child process to exit and the reader thread to finish.
                session.child.wait()?;
                if let Some(reader_thread) = session.reader_thread.take() {
                    reader_thread.join().expect("reader thread panicked");
                }

                let output = session.output.lock().unwrap();
                let output = String::from_utf8_lossy(&output);
                assert!(output.contains("Hello, world!"));
                Ok(())
            }
        }
    }
}

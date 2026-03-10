use std::{
    io::Read,
    io::Write,
    sync::{Arc, Mutex},
    thread,
    thread::JoinHandle,
};

use alacritty_terminal::{
    event::VoidListener,
    term::{Config, Term, cell::Flags, test::TermSize},
    vte::ansi::Processor,
};
use anyhow::Result;
use portable_pty::{Child, CommandBuilder, MasterPty, PtySize, native_pty_system};

use crate::screen::pad_to_cols;
use crate::terminal::TerminalSize;

struct Screen {
    parser: Processor,
    terminal: Term<VoidListener>,
}

impl Screen {
    fn new(size: TerminalSize) -> Self {
        let size = TermSize::new(size.cols as usize, size.rows as usize);
        Self {
            parser: Processor::new(),
            terminal: Term::new(Config::default(), &size, VoidListener),
        }
    }

    fn process(&mut self, chunk: &[u8]) {
        self.parser.advance(&mut self.terminal, chunk);
    }

    fn resize(&mut self, size: TerminalSize) {
        let size = TermSize::new(size.cols as usize, size.rows as usize);
        self.terminal.resize(size);
    }

    fn snapshot(&self, size: TerminalSize) -> Vec<String> {
        let mut lines = Vec::with_capacity(size.rows as usize);
        let mut current_line = None;

        for indexed in self.terminal.grid().display_iter() {
            if current_line != Some(indexed.point.line.0) {
                lines.push(String::new());
                current_line = Some(indexed.point.line.0);
            }

            let line = lines
                .last_mut()
                .expect("display iterator should yield rows");
            if indexed.cell.flags.contains(Flags::WIDE_CHAR_SPACER) {
                continue;
            }

            line.push(indexed.cell.c);
            if let Some(zerowidth) = indexed.cell.zerowidth() {
                for ch in zerowidth {
                    line.push(*ch);
                }
            }
        }

        lines.resize(size.rows as usize, String::new());
        lines
            .into_iter()
            .map(|line| pad_to_cols(size.cols, &line))
            .collect()
    }
}

pub struct Session {
    pub child: Box<dyn Child + Send + Sync>,
    pub master: Box<dyn MasterPty + Send>,
    pub writer: Box<dyn Write + Send>,
    pub output: Arc<Mutex<Vec<u8>>>,
    screen: Arc<Mutex<Screen>>,
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
        let screen = Arc::new(Mutex::new(Screen::new(size)));
        let screen_reader = Arc::clone(&screen);
        let mut reader = master.try_clone_reader()?;
        let reader_thread = thread::spawn(move || {
            let mut buf = [0_u8; 4096];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        let chunk = &buf[..n];
                        output_reader
                            .lock()
                            .expect("failed to lock output buffer")
                            .extend_from_slice(chunk);
                        screen_reader
                            .lock()
                            .expect("failed to lock screen parser")
                            .process(chunk);
                    }
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
            screen,
            reader_thread: Some(reader_thread),
            size,
        })
    }

    pub fn resize(&mut self, size: TerminalSize) -> Result<()> {
        self.master.resize(PtySize {
            rows: size.rows,
            cols: size.cols,
            pixel_width: 0,
            pixel_height: 0,
        })?;
        self.screen
            .lock()
            .expect("failed to lock screen parser")
            .resize(size);
        self.size = size;
        Ok(())
    }

    pub fn screen_snapshot(&self) -> Vec<String> {
        self.screen
            .lock()
            .expect("failed to lock screen parser")
            .snapshot(self.size)
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

        mod screen {
            use super::*;

            #[test]
            fn resize_reflows_wrapped_lines() {
                let mut screen = Screen::new(TerminalSize::new(3, 8));
                screen.process(b"abcdefghij");

                assert_eq!(
                    screen.snapshot(TerminalSize::new(3, 8)),
                    vec![
                        "abcdefgh".to_string(),
                        "ij      ".to_string(),
                        "        ".to_string(),
                    ]
                );

                screen.resize(TerminalSize::new(3, 6));

                assert_eq!(
                    screen.snapshot(TerminalSize::new(3, 6)),
                    vec![
                        "abcdef".to_string(),
                        "ghij  ".to_string(),
                        "      ".to_string(),
                    ]
                );
            }
        }
    }
}

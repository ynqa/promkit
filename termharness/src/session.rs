use std::{
    io::Read,
    io::Write,
    sync::{Arc, Mutex},
    thread,
    thread::JoinHandle,
};

use crate::terminal::TerminalSize;
use alacritty_terminal::{
    event::VoidListener,
    index::{Column, Line, Point},
    term::{Config, Term, cell::Flags, test::TermSize},
    vte::ansi::Processor,
};
use anyhow::Result;
use portable_pty::{Child, CommandBuilder, MasterPty, PtySize, native_pty_system};
use unicode_width::UnicodeWidthStr;

fn pad_to_cols(cols: u16, content: &str) -> String {
    let width = content.width();
    assert!(
        width <= cols as usize,
        "line width {width} exceeds terminal width {cols}"
    );

    let mut line = String::from(content);
    line.push_str(&" ".repeat(cols as usize - width));
    line
}

fn cursor_position_request_count(buffer: &[u8]) -> usize {
    buffer
        .windows(4)
        .filter(|window| *window == b"\x1b[6n")
        .count()
}

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

    fn with_cursor(size: TerminalSize, row: u16, col: u16) -> Self {
        let mut screen = Self::new(size);
        screen.set_cursor_position(row, col);
        screen
    }

    fn process(&mut self, chunk: &[u8]) {
        self.parser.advance(&mut self.terminal, chunk);
    }

    fn resize(&mut self, size: TerminalSize) {
        let size = TermSize::new(size.cols as usize, size.rows as usize);
        self.terminal.resize(size);
    }

    fn cursor_position(&self) -> (u16, u16) {
        let point = self.terminal.grid().cursor.point;
        let row = u16::try_from(point.line.0).expect("cursor row should be non-negative") + 1;
        let col = u16::try_from(point.column.0).expect("cursor column should fit in u16") + 1;
        (row, col)
    }

    fn set_cursor_position(&mut self, row: u16, col: u16) {
        let cursor = &mut self.terminal.grid_mut().cursor;
        cursor.point = Point::new(
            Line(i32::from(row.saturating_sub(1))),
            Column(usize::from(col.saturating_sub(1))),
        );
        cursor.input_needs_wrap = false;
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

type SharedWriter = Arc<Mutex<Box<dyn Write + Send>>>;

pub struct Session {
    pub child: Box<dyn Child + Send + Sync>,
    pub master: Box<dyn MasterPty + Send>,
    pub writer: SharedWriter,
    pub output: Arc<Mutex<Vec<u8>>>,
    screen: Arc<Mutex<Screen>>,
    pub reader_thread: Option<JoinHandle<()>>,
    pub size: TerminalSize,
}

impl Session {
    /// Spawn a new session by executing the given command in a pseudo-terminal with the specified size.
    pub fn spawn(cmd: CommandBuilder, size: TerminalSize) -> Result<Self> {
        Self::spawn_with_cursor(cmd, size, None)
    }

    pub fn spawn_with_cursor(
        mut cmd: CommandBuilder,
        size: TerminalSize,
        cursor_position: Option<(u16, u16)>,
    ) -> Result<Self> {
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
        let screen = Arc::new(Mutex::new(match cursor_position {
            Some((row, col)) => Screen::with_cursor(size, row, col),
            None => Screen::new(size),
        }));
        let screen_reader = Arc::clone(&screen);
        let writer = Arc::new(Mutex::new(master.take_writer()?));
        let writer_reader = Arc::clone(&writer);
        let mut reader = master.try_clone_reader()?;
        let reader_thread = thread::spawn(move || {
            let mut buf = [0_u8; 4096];
            let mut tail = Vec::new();
            loop {
                match reader.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        let chunk = &buf[..n];
                        output_reader
                            .lock()
                            .expect("failed to lock output buffer")
                            .extend_from_slice(chunk);
                        let mut scan = tail;
                        scan.extend_from_slice(chunk);

                        let (response_count, cursor_position) = {
                            let mut screen =
                                screen_reader.lock().expect("failed to lock screen parser");
                            screen.process(chunk);
                            (
                                cursor_position_request_count(&scan),
                                screen.cursor_position(),
                            )
                        };

                        if response_count > 0 {
                            let response =
                                format!("\x1b[{};{}R", cursor_position.0, cursor_position.1);
                            let mut writer =
                                writer_reader.lock().expect("failed to lock session writer");
                            for _ in 0..response_count {
                                if writer.write_all(response.as_bytes()).is_err()
                                    || writer.flush().is_err()
                                {
                                    return;
                                }
                            }
                        }

                        let keep_from = scan.len().saturating_sub(3);
                        tail = scan.split_off(keep_from);
                    }
                    Err(err) if err.kind() == std::io::ErrorKind::Interrupted => continue,
                    Err(_) => break,
                }
            }
        });
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

            #[test]
            fn responds_to_cursor_position_requests() -> Result<()> {
                let mut cmd = CommandBuilder::new("/bin/bash");
                cmd.arg("-lc");
                cmd.arg(r#"printf 'abc\033[6n'; IFS= read -rsd R pos; printf '%sR' "$pos""#);
                let mut session = Session::spawn(cmd, TerminalSize::new(24, 80))?;

                session.child.wait()?;
                if let Some(reader_thread) = session.reader_thread.take() {
                    reader_thread.join().expect("reader thread panicked");
                }

                let output = session.output.lock().unwrap();
                assert!(
                    String::from_utf8_lossy(&output).contains("\x1b[1;4R"),
                    "expected DSR response in output, got {:?}",
                    String::from_utf8_lossy(&output),
                );
                Ok(())
            }

            #[test]
            fn responds_from_custom_initial_cursor_position() -> Result<()> {
                let mut cmd = CommandBuilder::new("/bin/bash");
                cmd.arg("-lc");
                cmd.arg(r#"printf '\033[6n'; IFS= read -rsd R pos; printf '%sR' "$pos""#);
                let mut session =
                    Session::spawn_with_cursor(cmd, TerminalSize::new(24, 80), Some((24, 1)))?;

                session.child.wait()?;
                if let Some(reader_thread) = session.reader_thread.take() {
                    reader_thread.join().expect("reader thread panicked");
                }

                let output = session.output.lock().unwrap();
                assert!(
                    String::from_utf8_lossy(&output).contains("\x1b[24;1R"),
                    "expected DSR response in output, got {:?}",
                    String::from_utf8_lossy(&output),
                );
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

mod support;

use std::{
    io::{Read, Write},
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use portable_pty::{native_pty_system, CommandBuilder, MasterPty, PtySize};
use support::{assert_screen_eq, pad_to_cols, Screen};

const TERMINAL_ROWS: u16 = 6;
const TERMINAL_COLS: u16 = 80;
const RESIZED_TERMINAL_COLS: u16 = 72;
const INITIAL_CURSOR_ROW: u16 = 6;
const INITIAL_CURSOR_COL: u16 = 1;
const INPUT_TEXT: &str = "abcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefghijklmnopqr";

fn render_screen(snapshot: &[u8], rows: u16, cols: u16) -> Vec<String> {
    let mut parser = vt100::Parser::new(rows, cols, 0);
    parser.process(snapshot);
    parser
        .screen()
        .rows(0, cols)
        .map(|row| pad_to_cols(cols, &row))
        .collect()
}

fn assert_screen(snapshot: &[u8], rows: u16, cols: u16, expected: &[String]) {
    let actual = render_screen(snapshot, rows, cols);
    assert_screen_eq(expected, &actual);
}

fn spawn_readline() -> anyhow::Result<(
    Box<dyn portable_pty::Child + Send + Sync>,
    Box<dyn MasterPty + Send>,
    Box<dyn Write + Send>,
    Arc<Mutex<Vec<u8>>>,
    thread::JoinHandle<()>,
)> {
    let pty = native_pty_system();
    let pair = pty.openpty(PtySize {
        rows: TERMINAL_ROWS,
        cols: TERMINAL_COLS,
        pixel_width: 0,
        pixel_height: 0,
    })?;

    let mut cmd = CommandBuilder::new(env!("CARGO_BIN_EXE_readline"));
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
    Ok((child, master, writer, output, reader_thread))
}

#[test]
fn resize_wrap() -> anyhow::Result<()> {
    let expected = Screen::new(RESIZED_TERMINAL_COLS, TERMINAL_ROWS)
        .line(3, "Hi!")
        .line(
            4,
            "❯❯ abcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefghijklmnopq",
        )
        .line(5, "r")
        .build();

    let (mut child, master, mut writer, output, reader_thread) = spawn_readline()?;

    let cpr = format!("\x1b[{};{}R", INITIAL_CURSOR_ROW, INITIAL_CURSOR_COL);
    writer.write_all(cpr.as_bytes())?;
    writer.flush()?;
    thread::sleep(Duration::from_millis(200));

    writer.write_all(INPUT_TEXT.as_bytes())?;
    writer.flush()?;
    thread::sleep(Duration::from_millis(250));

    for cols in (RESIZED_TERMINAL_COLS..TERMINAL_COLS).rev() {
        master.resize(PtySize {
            rows: TERMINAL_ROWS,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        })?;
        thread::sleep(Duration::from_millis(150));
    }

    let snapshot = output.lock().expect("failed to lock output buffer").clone();
    assert_screen(&snapshot, TERMINAL_ROWS, RESIZED_TERMINAL_COLS, &expected);

    writer.write_all(b"\r")?;
    writer.flush()?;
    drop(writer);

    let _ = child.wait()?;
    reader_thread.join().expect("reader thread panicked");
    Ok(())
}

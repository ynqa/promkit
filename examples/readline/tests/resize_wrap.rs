use std::{io::Write, thread, time::Duration};

use portable_pty::{CommandBuilder, PtySize};
use termharness::{
    assert_screen_eq,
    screen::{pad_to_cols, Screen},
    session::Session,
    terminal::TerminalSize,
};

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

#[test]
fn resize_wrap() -> anyhow::Result<()> {
    let expected = Screen::new(RESIZED_TERMINAL_COLS, TERMINAL_ROWS)
        .line(3, "Hi!")?
        .line(
            4,
            "❯❯ abcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefghijklmnopq",
        )?
        .line(5, "r")?
        .build();

    let cmd = CommandBuilder::new(env!("CARGO_BIN_EXE_readline"));
    let mut session = Session::spawn(cmd, TerminalSize::new(TERMINAL_ROWS, TERMINAL_COLS))?;

    let cpr = format!("\x1b[{};{}R", INITIAL_CURSOR_ROW, INITIAL_CURSOR_COL);
    session.writer.write_all(cpr.as_bytes())?;
    session.writer.flush()?;
    thread::sleep(Duration::from_millis(200));

    session.writer.write_all(INPUT_TEXT.as_bytes())?;
    session.writer.flush()?;
    thread::sleep(Duration::from_millis(250));

    for cols in (RESIZED_TERMINAL_COLS..TERMINAL_COLS).rev() {
        session.master.resize(PtySize {
            rows: TERMINAL_ROWS,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        })?;
        thread::sleep(Duration::from_millis(150));
    }

    let snapshot = session
        .output
        .lock()
        .expect("failed to lock output buffer")
        .clone();
    assert_screen(&snapshot, TERMINAL_ROWS, RESIZED_TERMINAL_COLS, &expected);

    session.writer.write_all(b"\r")?;
    session.writer.flush()?;
    drop(session.writer);

    let _ = session.child.wait()?;
    session
        .reader_thread
        .take()
        .expect("reader thread should exist")
        .join()
        .expect("reader thread panicked");
    Ok(())
}

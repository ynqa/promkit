use std::{io::Write, thread, time::Duration};

use portable_pty::CommandBuilder;
use termharness::{
    assert_screen_eq,
    screen::{pad_to_cols, Screen},
    session::Session,
    terminal::TerminalSize,
};

const TERMINAL_ROWS: u16 = 6;
const TERMINAL_COLS: u16 = 80;
const INITIAL_CURSOR_ROW: u16 = 6;
const INITIAL_CURSOR_COL: u16 = 1;
const INPUT_TEXT: &str = "abcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefghijklmnopqr";
const INSERTED_TEXT: &str = "HELLOWORLD!!!!";
const LEFT_MOVES: usize = 20;

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
fn middle_insert_wrap() -> anyhow::Result<()> {
    let expected = Screen::new(TERMINAL_COLS, TERMINAL_ROWS)
        .line(3, "Hi!")?
        .line(
            4,
            "❯❯ abcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxHELLOWORLD!!!!yzabcdefghijk",
        )?
        .line(5, "lmnopqr")?
        .build();

    let cmd = CommandBuilder::new(env!("CARGO_BIN_EXE_readline"));
    let mut session = Session::spawn(cmd, TerminalSize::new(TERMINAL_ROWS, TERMINAL_COLS))?;

    let cpr = format!("\x1b[{};{}R", INITIAL_CURSOR_ROW, INITIAL_CURSOR_COL);
    session.writer.write_all(cpr.as_bytes())?;
    session.writer.flush()?;
    thread::sleep(Duration::from_millis(200));

    session.writer.write_all(INPUT_TEXT.as_bytes())?;
    session.writer.flush()?;
    thread::sleep(Duration::from_millis(120));

    for _ in 0..LEFT_MOVES {
        session.writer.write_all(b"\x1b[D")?;
        session.writer.flush()?;
        thread::sleep(Duration::from_millis(20));
    }

    session.writer.write_all(INSERTED_TEXT.as_bytes())?;
    session.writer.flush()?;
    thread::sleep(Duration::from_millis(250));

    let snapshot = session
        .output
        .lock()
        .expect("failed to lock output buffer")
        .clone();
    assert_screen(&snapshot, TERMINAL_ROWS, TERMINAL_COLS, &expected);

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

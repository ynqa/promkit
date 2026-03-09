use std::{io::Write, thread, time::Duration};

use portable_pty::{CommandBuilder, PtySize};
use termharness::{
    screen::pad_to_cols,
    screen_assert::format_screen,
    session::Session,
    terminal::TerminalSize,
};

const TERMINAL_ROWS: u16 = 10;
const TERMINAL_COLS: u16 = 32;
const RESIZED_TERMINAL_COLS: u16 = 28;

fn render_screen(snapshot: &[u8], rows: u16, cols: u16) -> Vec<String> {
    let mut parser = vt100::Parser::new(rows, cols, 0);
    parser.process(snapshot);
    parser
        .screen()
        .rows(0, cols)
        .map(|row| pad_to_cols(cols, &row))
        .collect()
}

fn print_screen(label: &str, session: &Session) {
    let snapshot = session
        .output
        .lock()
        .expect("failed to lock output buffer")
        .clone();
    let screen = render_screen(&snapshot, TERMINAL_ROWS, TERMINAL_COLS);

    println!("== {label} ==");
    for line in format_screen(&screen, TERMINAL_ROWS as usize) {
        println!("{line}");
    }
}

fn resize(session: &mut Session, cols: u16) -> anyhow::Result<()> {
    session.master.resize(PtySize {
        rows: TERMINAL_ROWS,
        cols,
        pixel_width: 0,
        pixel_height: 0,
    })?;
    thread::sleep(Duration::from_millis(120));
    print_screen(&format!("resize -> {cols} cols"), session);
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let mut cmd = CommandBuilder::new("/bin/zsh");
    cmd.arg("-fi");
    cmd.env("PS1", "❯❯ ");
    cmd.env("RPS1", "");
    cmd.env("RPROMPT", "");
    cmd.env("PROMPT_EOL_MARK", "");
    let mut session = Session::spawn(cmd, TerminalSize::new(TERMINAL_ROWS, TERMINAL_COLS))?;

    thread::sleep(Duration::from_millis(300));
    print_screen("spawn", &session);

    let move_cursor_to_bottom = format!("printf '\\x1b[{};1H'\r", TERMINAL_ROWS);
    session.writer.write_all(move_cursor_to_bottom.as_bytes())?;
    session.writer.flush()?;
    thread::sleep(Duration::from_millis(300));
    print_screen("move cursor to bottom", &session);

    session
        .writer
        .write_all(b"echo \"ynqa is a software engineer\"\r")?;
    session.writer.flush()?;
    thread::sleep(Duration::from_millis(300));
    print_screen("run echo", &session);

    session
        .writer
        .write_all(b"this is terminal test suite!")?;
    session.writer.flush()?;
    thread::sleep(Duration::from_millis(200));
    print_screen("type text", &session);

    for cols in (RESIZED_TERMINAL_COLS..TERMINAL_COLS).rev() {
        resize(&mut session, cols)?;
    }
    for cols in (RESIZED_TERMINAL_COLS + 1)..=TERMINAL_COLS {
        resize(&mut session, cols)?;
    }

    session.writer.write_all(b"\x03exit\r")?;
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

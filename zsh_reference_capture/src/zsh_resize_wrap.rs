use std::{io::Write, thread, time::Duration};

use portable_pty::CommandBuilder;
use termharness::{screen_assert::format_screen, session::Session, terminal::TerminalSize};

const TERMINAL_ROWS: u16 = 10;
const TERMINAL_COLS: u16 = 40;
const RESIZED_TERMINAL_COLS: u16 = 20;
const TIMES_TO_MOVE_CURSOR_LEFT: usize = 30;

fn print_screen(label: &str, session: &Session) {
    let screen = session.screen_snapshot();

    println!("== {label} ==");
    for line in format_screen(&screen, TERMINAL_ROWS as usize) {
        println!("{line}");
    }
}

fn resize(session: &mut Session, cols: u16) -> anyhow::Result<()> {
    session.resize(TerminalSize::new(TERMINAL_ROWS, cols))?;
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
        .write_all(b"\"ynqa is a software engineer\"\r")?;
    session.writer.flush()?;
    thread::sleep(Duration::from_millis(300));
    print_screen("run echo", &session);

    session.writer.write_all(b"this is terminal test suite!")?;
    session.writer.flush()?;
    thread::sleep(Duration::from_millis(200));
    print_screen("type text", &session);

    // IMPORTANT: Move the zsh edit cursor far enough left so that, at the
    // narrowest resized width, it is no longer sitting on the active input's
    // reflow boundary. Otherwise, growing the terminal back can reflow the
    // active input differently and push older wrapped output out of view.
    for _ in 0..TIMES_TO_MOVE_CURSOR_LEFT {
        session.writer.write_all(b"\x1b[D")?;
    }
    session.writer.flush()?;
    thread::sleep(Duration::from_millis(200));
    print_screen("move cursor left", &session);
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

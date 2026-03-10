use std::{io::Write, thread, time::Duration};

use portable_pty::CommandBuilder;
use termharness::{screen_assert::format_screen, session::Session, terminal::TerminalSize};

const TERMINAL_ROWS: u16 = 10;
const TERMINAL_COLS: u16 = 40;
const INPUT_TEXT: &str = "ynqa is a software engineer who writes terminal tools every day";
const INSERTED_TEXT: &str = " and open source maintainer";
const TIMES_TO_MOVE_CURSOR_LEFT: usize = 36;

fn print_screen(label: &str, session: &Session) {
    let screen = session.screen_snapshot();

    println!("== {label} ==");
    for line in format_screen(&screen, TERMINAL_ROWS as usize) {
        println!("{line}");
    }
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

    session.writer.write_all(INPUT_TEXT.as_bytes())?;
    session.writer.flush()?;
    thread::sleep(Duration::from_millis(200));
    print_screen("type text", &session);

    for _ in 0..TIMES_TO_MOVE_CURSOR_LEFT {
        session.writer.write_all(b"\x1b[D")?;
    }
    session.writer.flush()?;
    thread::sleep(Duration::from_millis(200));
    print_screen("move cursor left", &session);

    session.writer.write_all(INSERTED_TEXT.as_bytes())?;
    session.writer.flush()?;
    thread::sleep(Duration::from_millis(250));
    print_screen("insert text", &session);

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

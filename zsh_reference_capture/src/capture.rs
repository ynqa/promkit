use std::io::Write;

use portable_pty::CommandBuilder;
use termharness::{screen_assert::format_screen, session::Session, terminal::TerminalSize};

pub fn spawn_zsh_session(rows: u16, cols: u16) -> anyhow::Result<Session> {
    let mut cmd = CommandBuilder::new("/bin/zsh");
    cmd.arg("-fi");
    cmd.env("PS1", "❯❯ ");
    cmd.env("RPS1", "");
    cmd.env("RPROMPT", "");
    cmd.env("PROMPT_EOL_MARK", "");
    Session::spawn(cmd, TerminalSize::new(rows, cols))
}

pub fn send_bytes(session: &mut Session, bytes: &[u8]) -> anyhow::Result<()> {
    session.writer.write_all(bytes)?;
    session.writer.flush()?;
    Ok(())
}

pub fn print_screen(label: &str, session: &Session, rows: usize) {
    let screen = session.screen_snapshot();

    println!("== {label} ==");
    for line in format_screen(&screen, rows) {
        println!("{line}");
    }
}

pub fn move_cursor_to(session: &mut Session, row: u16, col: u16) -> anyhow::Result<()> {
    let command = format!("printf '\\x1b[{};{}H'\r", row, col);
    send_bytes(session, command.as_bytes())
}

pub fn move_cursor_left(session: &mut Session, times: usize) -> anyhow::Result<()> {
    for _ in 0..times {
        session.writer.write_all(b"\x1b[D")?;
    }
    session.writer.flush()?;
    Ok(())
}

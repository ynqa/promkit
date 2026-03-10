use std::io::Write;

use portable_pty::CommandBuilder;
use termharness::{session::Session, terminal::TerminalSize};

/// Spawn a session with the given command and terminal size.
pub fn spawn_session(cmd: CommandBuilder, rows: u16, cols: u16) -> anyhow::Result<Session> {
    Session::spawn(cmd, TerminalSize::new(rows, cols))
}

/// Spawn a zsh session with the given terminal size.
pub fn spawn_zsh_session(rows: u16, cols: u16) -> anyhow::Result<Session> {
    let mut cmd = CommandBuilder::new("/bin/zsh");
    cmd.arg("-fi");
    cmd.env("PS1", "❯❯ ");
    cmd.env("RPS1", "");
    cmd.env("RPROMPT", "");
    cmd.env("PROMPT_EOL_MARK", "");
    spawn_session(cmd, rows, cols)
}

/// Send bytes to the session's stdin.
pub fn send_bytes(session: &mut Session, bytes: &[u8]) -> anyhow::Result<()> {
    session.writer.write_all(bytes)?;
    session.writer.flush()?;
    Ok(())
}

/// Move the cursor to the given row and column (1-indexed).
pub fn move_cursor_to(session: &mut Session, row: u16, col: u16) -> anyhow::Result<()> {
    let command = format!("printf '\\x1b[{};{}H'\r", row, col);
    send_bytes(session, command.as_bytes())
}

/// Move the cursor left by the given number of times.
pub fn move_cursor_left(session: &mut Session, times: usize) -> anyhow::Result<()> {
    for _ in 0..times {
        session.writer.write_all(b"\x1b[D")?;
    }
    session.writer.flush()?;
    Ok(())
}

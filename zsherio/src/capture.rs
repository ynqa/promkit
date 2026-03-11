use std::io::Write;

use portable_pty::CommandBuilder;
use termharness::{session::Session, terminal::TerminalSize};

/// Spawn a session with the given command and terminal size.
pub fn spawn_session(cmd: CommandBuilder, rows: u16, cols: u16) -> anyhow::Result<Session> {
    Session::spawn(cmd, TerminalSize::new(rows, cols))
}

/// Spawn a session with the given command, terminal size, and initial cursor position.
pub fn spawn_session_with_cursor(
    cmd: CommandBuilder,
    rows: u16,
    cols: u16,
    cursor_row: u16,
    cursor_col: u16,
) -> anyhow::Result<Session> {
    Session::spawn_with_cursor(
        cmd,
        TerminalSize::new(rows, cols),
        Some((cursor_row, cursor_col)),
    )
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
    let mut writer = session
        .writer
        .lock()
        .expect("failed to lock session writer");
    writer.write_all(bytes)?;
    writer.flush()?;
    Ok(())
}

/// Move the cursor to the given row and column (1-indexed).
pub fn move_cursor_to(session: &mut Session, row: u16, col: u16) -> anyhow::Result<()> {
    let command = format!("printf '\\x1b[{};{}H'\r", row, col);
    send_bytes(session, command.as_bytes())
}

/// Clear the visible screen and move the cursor to the given row and column (1-indexed).
///
/// This is useful when positioning the prompt via a shell command because the command itself
/// is echoed before it runs. Clearing after execution prevents that setup command from
/// remaining in subsequent screen snapshots.
pub fn clear_screen_and_move_cursor_to(
    session: &mut Session,
    row: u16,
    col: u16,
) -> anyhow::Result<()> {
    let command = format!("printf '\\x1b[2J\\x1b[{};{}H'\r", row, col);
    send_bytes(session, command.as_bytes())
}

/// Move the cursor left by the given number of times.
pub fn move_cursor_left(session: &mut Session, times: usize) -> anyhow::Result<()> {
    let mut writer = session
        .writer
        .lock()
        .expect("failed to lock session writer");
    for _ in 0..times {
        writer.write_all(b"\x1b[D")?;
    }
    writer.flush()?;
    Ok(())
}

use std::io::Write;

use termharness::session::Session;

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

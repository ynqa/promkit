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

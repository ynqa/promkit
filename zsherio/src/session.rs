use portable_pty::CommandBuilder;
use termharness::session::Session;

/// Spawn a session with the given command, terminal size, and initial cursor position.
pub fn spawn_session(
    cmd: CommandBuilder,
    term_size: (u16, u16),
    cursor_pos: Option<(u16, u16)>,
) -> anyhow::Result<Session> {
    Session::spawn(cmd, term_size, cursor_pos)
}

/// Spawn a zsh session with the given terminal size.
pub fn spawn_zsh_session(
    term_size: (u16, u16),
    cursor_pos: Option<(u16, u16)>,
) -> anyhow::Result<Session> {
    let mut cmd = CommandBuilder::new("/bin/zsh");
    cmd.arg("-fi");
    cmd.env("PS1", "❯❯ ");
    cmd.env("RPS1", "");
    cmd.env("RPROMPT", "");
    cmd.env("PROMPT_EOL_MARK", "");
    spawn_session(cmd, term_size, cursor_pos)
}

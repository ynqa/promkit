use std::{path::PathBuf, process::Command, thread, time::Duration};

use portable_pty::CommandBuilder;
use zsh_reference_capture::capture::{move_cursor_left, print_screen, send_bytes, spawn_session};

const TERMINAL_ROWS: u16 = 10;
const TERMINAL_COLS: u16 = 40;
const INPUT_TEXT: &str = "ynqa is a software engineer who writes terminal tools every day";
const INSERTED_TEXT: &str = " and open source maintainer";
const TIMES_TO_MOVE_CURSOR_LEFT: usize = 36;

fn main() -> anyhow::Result<()> {
    build_zsh_pretend()?;

    let mut session = spawn_session(
        CommandBuilder::new(zsh_pretend_binary_path()?),
        TERMINAL_ROWS,
        TERMINAL_COLS,
    )?;

    wait_for_prompt(&session)?;
    print_screen("move cursor to bottom", &session, TERMINAL_ROWS as usize);

    send_bytes(&mut session, INPUT_TEXT.as_bytes())?;
    thread::sleep(Duration::from_millis(200));
    print_screen("type text", &session, TERMINAL_ROWS as usize);

    move_cursor_left(&mut session, TIMES_TO_MOVE_CURSOR_LEFT)?;
    thread::sleep(Duration::from_millis(200));
    print_screen("move cursor left", &session, TERMINAL_ROWS as usize);

    send_bytes(&mut session, INSERTED_TEXT.as_bytes())?;
    thread::sleep(Duration::from_millis(250));
    print_screen("insert text", &session, TERMINAL_ROWS as usize);

    Ok(())
}

fn wait_for_prompt(session: &termharness::session::Session) -> anyhow::Result<()> {
    let deadline = std::time::Instant::now() + Duration::from_secs(2);
    while std::time::Instant::now() < deadline {
        let screen = session.screen_snapshot();
        if screen.last().is_some_and(|line| line.starts_with("❯❯ ")) {
            return Ok(());
        }
        thread::sleep(Duration::from_millis(20));
    }

    Err(anyhow::anyhow!("timed out waiting for prompt"))
}

fn build_zsh_pretend() -> anyhow::Result<()> {
    let status = Command::new("cargo")
        .args(["build", "-q", "-p", "zsh-pretend", "--bin", "zsh-pretend"])
        .status()?;
    if !status.success() {
        return Err(anyhow::anyhow!("failed to build zsh-pretend"));
    }
    Ok(())
}

fn zsh_pretend_binary_path() -> anyhow::Result<PathBuf> {
    let current_exe = std::env::current_exe()?;
    let parent = current_exe
        .parent()
        .ok_or_else(|| anyhow::anyhow!("failed to resolve executable directory"))?;
    Ok(parent.join("zsh-pretend"))
}

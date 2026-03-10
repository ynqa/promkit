use std::{path::PathBuf, process::Command, thread, time::Duration};

use portable_pty::CommandBuilder;
use zsherio::capture::{move_cursor_left, print_screen, send_bytes, spawn_session};

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

    respond_to_cursor_position_request(&mut session, TERMINAL_ROWS, 1)?;
    wait_for_prompt(&session)?;
    print_screen("startup", &session, TERMINAL_ROWS as usize);

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

fn respond_to_cursor_position_request(
    session: &mut termharness::session::Session,
    row: u16,
    col: u16,
) -> anyhow::Result<()> {
    let deadline = std::time::Instant::now() + Duration::from_secs(2);
    while std::time::Instant::now() < deadline {
        let requested = {
            let output = session
                .output
                .lock()
                .expect("failed to lock session output buffer");
            output.windows(4).any(|window| window == b"\x1b[6n")
        };
        if requested {
            let response = format!("\x1b[{row};{col}R");
            send_bytes(session, response.as_bytes())?;
            return Ok(());
        }
        thread::sleep(Duration::from_millis(20));
    }

    Err(anyhow::anyhow!(
        "timed out waiting for cursor position request"
    ))
}

fn wait_for_prompt(session: &termharness::session::Session) -> anyhow::Result<()> {
    let deadline = std::time::Instant::now() + Duration::from_secs(2);
    while std::time::Instant::now() < deadline {
        let screen = session.screen_snapshot();
        if screen.iter().any(|line| line.starts_with("❯❯ ")) {
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

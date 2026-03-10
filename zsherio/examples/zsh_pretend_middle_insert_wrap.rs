use std::{ffi::OsStr, path::PathBuf, process::Command, thread, time::Duration};

use portable_pty::CommandBuilder;
use termharness::session::Session;
use zsherio::Scenario;
use zsherio::capture::{move_cursor_left, send_bytes, spawn_session};

const TERMINAL_ROWS: u16 = 10;
const TERMINAL_COLS: u16 = 40;
const INPUT_TEXT: &str = "ynqa is a software engineer who writes terminal tools every day";
const INSERTED_TEXT: &str = " and open source maintainer";
const TIMES_TO_MOVE_CURSOR_LEFT: usize = 36;

fn scenario() -> Scenario {
    Scenario::new("middle_insert_wrap")
        .step("startup", Duration::ZERO, |session| {
            respond_to_cursor_position_request(session, TERMINAL_ROWS, 1)?;
            wait_for_prompt(session)
        })
        .step("type text", Duration::from_millis(200), |session| {
            send_bytes(session, INPUT_TEXT.as_bytes())
        })
        .step("move cursor left", Duration::from_millis(200), |session| {
            move_cursor_left(session, TIMES_TO_MOVE_CURSOR_LEFT)
        })
        .step("insert text", Duration::from_millis(250), |session| {
            send_bytes(session, INSERTED_TEXT.as_bytes())
        })
}

fn main() -> anyhow::Result<()> {
    build_zsh_pretend()?;

    let mut session = spawn_session(
        CommandBuilder::new(zsh_pretend_binary_path()?),
        TERMINAL_ROWS,
        TERMINAL_COLS,
    )?;

    let run = scenario().run("zsh-pretend", &mut session)?;
    run.write_to_stdout()
}

fn respond_to_cursor_position_request(
    session: &mut Session,
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

fn wait_for_prompt(session: &Session) -> anyhow::Result<()> {
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
    let mut command = Command::new("cargo");
    command.args(["build", "-q", "-p", "zsh-pretend", "--bin", "zsh-pretend"]);
    if target_binary_dir()?.file_name() == Some(OsStr::new("release")) {
        command.arg("--release");
    }

    let status = command.status()?;
    if !status.success() {
        return Err(anyhow::anyhow!("failed to build zsh-pretend"));
    }
    Ok(())
}

fn zsh_pretend_binary_path() -> anyhow::Result<PathBuf> {
    Ok(target_binary_dir()?.join("zsh-pretend"))
}

fn target_binary_dir() -> anyhow::Result<PathBuf> {
    let current_exe = std::env::current_exe()?;
    let executable_dir = current_exe
        .parent()
        .ok_or_else(|| anyhow::anyhow!("failed to resolve executable directory"))?;

    if executable_dir.file_name() == Some(OsStr::new("examples")) {
        return executable_dir
            .parent()
            .map(|path| path.to_path_buf())
            .ok_or_else(|| anyhow::anyhow!("failed to resolve target directory"));
    }

    Ok(executable_dir.to_path_buf())
}

use std::{path::PathBuf, thread, time::Duration};

use portable_pty::CommandBuilder;
use termharness::session::Session;
use zsherio::{
    ScenarioRun,
    capture::{clear_screen_and_move_cursor_to, send_bytes, spawn_session, spawn_zsh_session},
    scenarios::resize_wrap::{TERMINAL_COLS, TERMINAL_ROWS, scenario},
};

const ZSH_PRETEND_BIN: &str = env!("CARGO_BIN_EXE_zsh-pretend");

#[test]
fn zsh_pretend_matches_zsh_for_resize_wrap() -> anyhow::Result<()> {
    let expected = run_zsh()?;
    let actual = run_zsh_pretend()?;

    write_run_artifact(&expected)?;
    write_run_artifact(&actual)?;

    assert_eq!(
        actual.records,
        expected.records,
        "zsh-pretend output diverged from zsh\n\nexpected:\n{}\nactual:\n{}",
        render_run(&expected)?,
        render_run(&actual)?,
    );

    Ok(())
}

fn run_zsh() -> anyhow::Result<ScenarioRun> {
    let mut session = spawn_zsh_session(TERMINAL_ROWS, TERMINAL_COLS)?;

    clear_screen_and_move_cursor_to(&mut session, TERMINAL_ROWS, 1)?;
    thread::sleep(Duration::from_millis(300));

    scenario().run("zsh", &mut session)
}

fn run_zsh_pretend() -> anyhow::Result<ScenarioRun> {
    let mut session = spawn_session(
        CommandBuilder::new(ZSH_PRETEND_BIN),
        TERMINAL_ROWS,
        TERMINAL_COLS,
    )?;

    respond_to_cursor_position_request(&mut session, TERMINAL_ROWS, 1)?;
    wait_for_prompt(&session)?;

    scenario().run("zsh-pretend", &mut session)
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

fn render_run(run: &ScenarioRun) -> anyhow::Result<String> {
    let mut output = Vec::new();
    run.write_to(&mut output)?;
    Ok(String::from_utf8(output)?)
}

fn write_run_artifact(run: &ScenarioRun) -> anyhow::Result<()> {
    run.write_to_path(&artifact_path(run))
}

fn artifact_path(run: &ScenarioRun) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(".artifacts")
        .join(&run.scenario_name)
        .join(format!("{}.txt", run.target_name))
}

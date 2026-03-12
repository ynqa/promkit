mod common;

use portable_pty::CommandBuilder;
use zsherio::{
    scenarios::middle_prompt_start::{
        scenario, START_CURSOR_COL, START_CURSOR_ROW, TERMINAL_COLS, TERMINAL_ROWS,
    },
    session::spawn_session_with_cursor,
    ScenarioRun,
};

use crate::common::{assert_scenario_runs_match, wait_for_prompt, write_scenario_run_artifact};

const ZSH_PRETEND_BIN: &str = env!("CARGO_BIN_EXE_zsh-pretend");

#[test]
fn zsh_pretend_matches_zsh_for_middle_prompt_start() -> anyhow::Result<()> {
    let expected = run_zsh()?;
    let actual = run_zsh_pretend()?;

    write_scenario_run_artifact(&expected)?;
    write_scenario_run_artifact(&actual)?;

    assert_scenario_runs_match(&expected, &actual)?;

    Ok(())
}

fn run_zsh() -> anyhow::Result<ScenarioRun> {
    let mut cmd = CommandBuilder::new("/bin/zsh");
    cmd.arg("-fi");
    cmd.env("PS1", "❯❯ ");
    cmd.env("RPS1", "");
    cmd.env("RPROMPT", "");
    cmd.env("PROMPT_EOL_MARK", "");
    let mut session = spawn_session_with_cursor(
        cmd,
        TERMINAL_ROWS,
        TERMINAL_COLS,
        START_CURSOR_ROW,
        START_CURSOR_COL,
    )?;
    wait_for_prompt(&session, |line| line.contains("❯❯ "))?;

    scenario().run("zsh", &mut session)
}

fn run_zsh_pretend() -> anyhow::Result<ScenarioRun> {
    let mut session = spawn_session_with_cursor(
        CommandBuilder::new(ZSH_PRETEND_BIN),
        TERMINAL_ROWS,
        TERMINAL_COLS,
        START_CURSOR_ROW,
        START_CURSOR_COL,
    )?;

    wait_for_prompt(&session, |line| line.contains("❯❯ "))?;

    scenario().run("zsh-pretend", &mut session)
}

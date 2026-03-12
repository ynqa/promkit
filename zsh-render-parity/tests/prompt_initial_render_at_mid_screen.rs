mod common;

use portable_pty::CommandBuilder;
use zsherio::{
    scenarios::prompt_initial_render_at_mid_screen::{
        scenario, START_CURSOR_COL, START_CURSOR_ROW, TERMINAL_COLS, TERMINAL_ROWS,
    },
    session::{spawn_session, spawn_zsh_session},
    ScenarioRun,
};

use crate::common::{assert_scenario_runs_match, wait_for_prompt, write_scenario_run_artifact};

const ZSH_PRETEND_BIN: &str = env!("CARGO_BIN_EXE_zsh-pretend");

#[test]
fn zsh_pretend_parity_prompt_initial_render_at_mid_screen() -> anyhow::Result<()> {
    let expected = run_zsh()?;
    let actual = run_zsh_pretend()?;

    write_scenario_run_artifact(&expected)?;
    write_scenario_run_artifact(&actual)?;

    assert_scenario_runs_match(&expected, &actual)?;

    Ok(())
}

fn run_zsh() -> anyhow::Result<ScenarioRun> {
    let mut session = spawn_zsh_session(
        (TERMINAL_ROWS, TERMINAL_COLS),
        Some((START_CURSOR_ROW, START_CURSOR_COL)),
    )?;
    wait_for_prompt(&session, |line| line.contains("❯❯ "))?;

    scenario().run("zsh", &mut session)
}

fn run_zsh_pretend() -> anyhow::Result<ScenarioRun> {
    let mut session = spawn_session(
        CommandBuilder::new(ZSH_PRETEND_BIN),
        (TERMINAL_ROWS, TERMINAL_COLS),
        Some((START_CURSOR_ROW, START_CURSOR_COL)),
    )?;

    wait_for_prompt(&session, |line| line.contains("❯❯ "))?;

    scenario().run("zsh-pretend", &mut session)
}

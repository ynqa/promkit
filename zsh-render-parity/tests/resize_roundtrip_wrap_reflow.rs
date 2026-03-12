mod common;

use std::{thread, time::Duration};

use portable_pty::CommandBuilder;
use zsherio::{
    opts::clear_screen_and_move_cursor_to,
    scenarios::resize_roundtrip_wrap_reflow::{scenario, TERMINAL_COLS, TERMINAL_ROWS},
    session::{spawn_session, spawn_zsh_session},
    ScenarioRun,
};

use crate::common::{assert_scenario_runs_match, wait_for_prompt, write_scenario_run_artifact};

const ZSH_PRETEND_BIN: &str = env!("CARGO_BIN_EXE_zsh-pretend");

#[test]
#[ignore = "timing-sensitive and currently unsupported: matching zsh under aggressive \
            resize-wrap is too hard right now; run manually with `cargo test --release --test \
            resize_roundtrip_wrap_reflow`"]
fn zsh_pretend_parity_resize_roundtrip_wrap_reflow() -> anyhow::Result<()> {
    let expected = run_zsh()?;
    let actual = run_zsh_pretend()?;

    write_scenario_run_artifact(&expected)?;
    write_scenario_run_artifact(&actual)?;

    assert_scenario_runs_match(&expected, &actual)?;

    Ok(())
}

fn run_zsh() -> anyhow::Result<ScenarioRun> {
    let mut session = spawn_zsh_session((TERMINAL_ROWS, TERMINAL_COLS), None)?;

    clear_screen_and_move_cursor_to(&mut session, TERMINAL_ROWS, 1)?;
    thread::sleep(Duration::from_millis(300));

    scenario().run("zsh", &mut session)
}

fn run_zsh_pretend() -> anyhow::Result<ScenarioRun> {
    let mut session = spawn_session(
        CommandBuilder::new(ZSH_PRETEND_BIN),
        (TERMINAL_ROWS, TERMINAL_COLS),
        Some((TERMINAL_ROWS, 1)),
    )?;

    wait_for_prompt(&session, |line| line.starts_with("❯❯ "))?;

    scenario().run("zsh-pretend", &mut session)
}

mod common;

use std::{thread, time::Duration};

use portable_pty::CommandBuilder;
use zsherio::{
    capture::{clear_screen_and_move_cursor_to, spawn_session_with_cursor, spawn_zsh_session},
    scenarios::resize_wrap::{scenario, TERMINAL_COLS, TERMINAL_ROWS},
    ScenarioRun,
};

use self::common::{assert_scenario_runs_match, wait_for_prompt, write_run_artifact};

const ZSH_PRETEND_BIN: &str = env!("CARGO_BIN_EXE_zsh-pretend");

#[test]
#[ignore = "timing-sensitive and currently unsupported: matching zsh under aggressive \
            resize-wrap is too hard right now; run manually with `cargo test --release --test \
            resize_wrap`"]
fn zsh_pretend_matches_zsh_for_resize_wrap() -> anyhow::Result<()> {
    let expected = run_zsh()?;
    let actual = run_zsh_pretend()?;

    write_run_artifact(&expected)?;
    write_run_artifact(&actual)?;

    assert_scenario_runs_match(&expected, &actual)?;

    Ok(())
}

fn run_zsh() -> anyhow::Result<ScenarioRun> {
    let mut session = spawn_zsh_session(TERMINAL_ROWS, TERMINAL_COLS)?;

    clear_screen_and_move_cursor_to(&mut session, TERMINAL_ROWS, 1)?;
    thread::sleep(Duration::from_millis(300));

    scenario().run("zsh", &mut session)
}

fn run_zsh_pretend() -> anyhow::Result<ScenarioRun> {
    let mut session = spawn_session_with_cursor(
        CommandBuilder::new(ZSH_PRETEND_BIN),
        TERMINAL_ROWS,
        TERMINAL_COLS,
        TERMINAL_ROWS,
        1,
    )?;

    wait_for_prompt(&session, |line| line.starts_with("❯❯ "))?;

    scenario().run("zsh-pretend", &mut session)
}

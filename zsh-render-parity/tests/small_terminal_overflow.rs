mod common;

use std::{thread, time::Duration};

use portable_pty::CommandBuilder;
use zsherio::{
    capture::{clear_screen_and_move_cursor_to, spawn_session_with_cursor, spawn_zsh_session},
    scenarios::small_terminal_overflow::{scenario, TERMINAL_COLS, TERMINAL_ROWS},
    ScenarioRun,
};

use self::common::{wait_for_prompt, write_run_artifact, render_scenario_run};

const ZSH_PRETEND_BIN: &str = env!("CARGO_BIN_EXE_zsh-pretend");

#[test]
fn zsh_pretend_matches_zsh_for_small_terminal_overflow() -> anyhow::Result<()> {
    let expected = run_zsh()?;
    let actual = run_zsh_pretend()?;

    write_run_artifact(&expected)?;
    write_run_artifact(&actual)?;

    assert_runs_match(&expected, &actual)?;

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

/// In the tiny overflow scenario, real zsh draws a start-ellipsis marker
/// (`>....`) on the first visible row when the logical input starts before
/// the viewport.
///
/// This marker is emitted by zle refresh internals and could not be disabled
/// via runtime prompt options in this harness, so `zsh` and `zsh-pretend`
/// intentionally differ on the first rendered row (`r00`).
///
/// Reference:
/// - https://github.com/zsh-users/zsh/blob/zsh-5.9/Src/Zle/zle_refresh.c#L1677
///
/// To keep this test focused on wrap/scroll behavior, we require strict
/// equality for scenario shape (step count, labels, row count) and compare
/// screen content from the second row (`r01`) onward.
fn runs_match_from_second_line(expected: &ScenarioRun, actual: &ScenarioRun) -> bool {
    if expected.records.len() != actual.records.len() {
        return false;
    }

    expected
        .records
        .iter()
        .zip(&actual.records)
        .all(|(expected_record, actual_record)| {
            expected_record.label == actual_record.label
                && expected_record.screen.len() == actual_record.screen.len()
                && expected_record
                    .screen
                    .iter()
                    .skip(1)
                    .eq(actual_record.screen.iter().skip(1))
        })
}

fn assert_runs_match(expected: &ScenarioRun, actual: &ScenarioRun) -> anyhow::Result<()> {
    if runs_match_from_second_line(expected, actual) {
        return Ok(());
    } else {
        anyhow::bail!(
            "zsh-pretend output diverged from zsh (ignoring first line of each screen)\n\n== expected ==\n{}\n== actual ==\n{}",
            render_scenario_run(expected)?,
            render_scenario_run(actual)?,
        )
    }
}

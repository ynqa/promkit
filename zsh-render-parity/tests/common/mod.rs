use std::{path::PathBuf, thread, time::Duration};

use termharness::session::Session;
use zsherio::ScenarioRun;

pub const ZSH_PRETEND_BIN: &str = env!("CARGO_BIN_EXE_zsh-pretend");
const PROMPT_WAIT_TIMEOUT: Duration = Duration::from_secs(2);
const PROMPT_POLL_INTERVAL: Duration = Duration::from_millis(20);

/// Wait until the session's screen contains a line that satisfies `is_prompt_line`,
/// or return an error if the timeout is reached.
pub fn wait_for_prompt(
    session: &Session,
    is_prompt_line: impl Fn(&str) -> bool,
) -> anyhow::Result<()> {
    let deadline = std::time::Instant::now() + PROMPT_WAIT_TIMEOUT;
    while std::time::Instant::now() < deadline {
        let screen = session.screen_snapshot();
        if screen.iter().any(|line| is_prompt_line(line)) {
            return Ok(());
        }
        thread::sleep(PROMPT_POLL_INTERVAL);
    }

    Err(anyhow::anyhow!("timed out waiting for prompt"))
}

/// Assert that the two scenario runs match, and if not,
/// return an error with a detailed diff of their outputs.
pub fn assert_scenario_runs_match(
    expected: &ScenarioRun,
    actual: &ScenarioRun,
) -> anyhow::Result<()> {
    if actual.records == expected.records {
        return Ok(());
    }

    anyhow::bail!(
        "zsh-pretend output diverged from zsh\n\n== expected ==\n{}\n== actual ==\n{}",
        render_scenario_run(expected)?,
        render_scenario_run(actual)?,
    )
}

pub fn render_scenario_run(run: &ScenarioRun) -> anyhow::Result<String> {
    let mut output = Vec::new();
    run.write_to(&mut output)?;
    Ok(String::from_utf8(output)?)
}

pub fn write_scenario_run_artifact(run: &ScenarioRun) -> anyhow::Result<()> {
    run.write_to_path(&artifact_path(run))
}

fn artifact_path(run: &ScenarioRun) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(".artifacts")
        .join(&run.scenario_name)
        .join(format!("{}.txt", run.target_name))
}

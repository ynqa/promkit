use std::{
    fmt,
    fs::File,
    io::{self, Write},
    path::Path,
    sync::Arc,
    thread,
    time::Duration,
};

use anyhow::{bail, ensure};
use termharness::screen_assert::format_screen;
use termharness::session::Session;

pub type StepAction = Arc<dyn Fn(&mut Session) -> anyhow::Result<()> + Send + Sync>;

#[derive(Clone)]
pub struct Scenario {
    pub name: &'static str,
    pub steps: Vec<ScenarioStep>,
}

#[derive(Clone)]
pub struct ScenarioStep {
    pub label: &'static str,
    pub compare: bool,
    pub settle: Duration,
    pub action: StepAction,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScenarioRecord {
    pub label: String,
    pub compare: bool,
    pub screen: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScenarioRun {
    pub scenario_name: String,
    pub target_name: String,
    pub records: Vec<ScenarioRecord>,
}

impl Scenario {
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            steps: Vec::new(),
        }
    }

    pub fn step<F>(
        mut self,
        label: &'static str,
        compare: bool,
        settle: Duration,
        action: F,
    ) -> Self
    where
        F: Fn(&mut Session) -> anyhow::Result<()> + Send + Sync + 'static,
    {
        self.steps
            .push(ScenarioStep::new(label, compare, settle, action));
        self
    }

    pub fn run(
        &self,
        target_name: impl Into<String>,
        session: &mut Session,
    ) -> anyhow::Result<ScenarioRun> {
        let mut records = Vec::with_capacity(self.steps.len());

        for step in &self.steps {
            (step.action)(session)?;
            thread::sleep(step.settle);

            let screen = session.screen_snapshot();
            records.push(ScenarioRecord {
                label: step.label.to_string(),
                compare: step.compare,
                screen: format_screen(&screen, screen.len()),
            });
        }

        Ok(ScenarioRun {
            scenario_name: self.name.to_string(),
            target_name: target_name.into(),
            records,
        })
    }
}

impl ScenarioStep {
    pub fn new<F>(label: &'static str, compare: bool, settle: Duration, action: F) -> Self
    where
        F: Fn(&mut Session) -> anyhow::Result<()> + Send + Sync + 'static,
    {
        Self {
            label,
            compare,
            settle,
            action: Arc::new(action),
        }
    }
}

impl ScenarioRun {
    pub fn compare(&self, other: &Self) -> anyhow::Result<()> {
        ensure!(
            self.scenario_name == other.scenario_name,
            "scenario mismatch: '{}' != '{}'",
            self.scenario_name,
            other.scenario_name
        );

        let expected = self.comparable_records();
        let actual = other.comparable_records();

        ensure!(
            expected.len() == actual.len(),
            "comparable step count mismatch for scenario '{}': '{}' has {}, '{}' has {}",
            self.scenario_name,
            self.target_name,
            expected.len(),
            other.target_name,
            actual.len()
        );

        for (index, (expected, actual)) in expected.iter().zip(actual.iter()).enumerate() {
            ensure!(
                expected.label == actual.label,
                "step label mismatch at comparable step {index} for scenario '{}': '{}' != '{}'",
                self.scenario_name,
                expected.label,
                actual.label
            );

            if expected.screen != actual.screen {
                bail!(
                    "screen mismatch for scenario '{}' at step '{}' ({} vs {})",
                    self.scenario_name,
                    expected.label,
                    self.target_name,
                    other.target_name,
                );
            }
        }

        Ok(())
    }

    pub fn write_to<W: Write>(&self, mut writer: W) -> anyhow::Result<()> {
        for (index, record) in self.records.iter().enumerate() {
            writeln!(writer, "== {} ==", record.label)?;
            for line in &record.screen {
                writeln!(writer, "{line}")?;
            }
            if index + 1 != self.records.len() {
                writeln!(writer)?;
            }
        }
        Ok(())
    }

    pub fn write_to_path(&self, path: &Path) -> anyhow::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        self.write_to(File::create(path)?)
    }

    pub fn write_to_stdout(&self) -> anyhow::Result<()> {
        self.write_to(io::stdout())
    }

    fn comparable_records(&self) -> Vec<&ScenarioRecord> {
        self.records
            .iter()
            .filter(|record| record.compare)
            .collect()
    }
}

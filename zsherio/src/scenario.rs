use std::{
    fs::File,
    io::{self, Write},
    path::Path,
    sync::Arc,
    thread,
    time::Duration,
};

use termharness::session::Session;

pub type StepAction = Arc<dyn Fn(&mut Session) -> anyhow::Result<()> + Send + Sync>;

#[derive(Clone)]
pub struct Scenario {
    pub name: String,
    pub steps: Vec<ScenarioStep>,
}

#[derive(Clone)]
pub struct ScenarioStep {
    pub label: String,
    pub settle: Duration,
    pub action: StepAction,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScenarioRecord {
    pub label: String,
    pub screen: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScenarioRun {
    pub scenario_name: String,
    pub target_name: String,
    pub records: Vec<ScenarioRecord>,
}

impl Scenario {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            steps: Vec::new(),
        }
    }

    pub fn step<F, S>(mut self, label: S, settle: Duration, action: F) -> Self
    where
        F: Fn(&mut Session) -> anyhow::Result<()> + Send + Sync + 'static,
        S: Into<String>,
    {
        self.steps.push(ScenarioStep::new(label, settle, action));
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
                label: step.label.clone(),
                screen: format_screen(&screen, screen.len()),
            });
        }

        Ok(ScenarioRun {
            scenario_name: self.name.clone(),
            target_name: target_name.into(),
            records,
        })
    }
}

impl ScenarioStep {
    pub fn new<F, S>(label: S, settle: Duration, action: F) -> Self
    where
        F: Fn(&mut Session) -> anyhow::Result<()> + Send + Sync + 'static,
        S: Into<String>,
    {
        Self {
            label: label.into(),
            settle,
            action: Arc::new(action),
        }
    }
}

impl ScenarioRun {
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
}

/// Format a single line of the screen, replacing spaces with a visible character and marking missing lines.
fn format_screen_line(line: Option<&String>) -> String {
    match line {
        Some(line) => format!("|{}|", line.replace(' ', "·")),
        None => "<missing>".to_string(),
    }
}

/// Format an entire screen, prefixing each line with its row number and marking differences.
fn format_screen(lines: &[String], total_rows: usize) -> Vec<String> {
    (0..total_rows)
        .map(|row| format!("  r{row:02} {}", format_screen_line(lines.get(row))))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    mod format_screen_line {
        use super::*;

        #[test]
        fn replaces_spaces() {
            assert_eq!(format_screen_line(Some(&"a b c".to_string())), "|a·b·c|");
        }

        #[test]
        fn handles_empty_line() {
            assert_eq!(format_screen_line(Some(&"".to_string())), "||");
        }

        #[test]
        fn handles_missing_line() {
            assert_eq!(format_screen_line(None), "<missing>");
        }
    }

    mod format_screen {
        use super::*;

        #[test]
        fn formats_multiple_lines() {
            let lines = vec![
                "line 1".to_string(),
                "line 2".to_string(),
                "line 3".to_string(),
            ];
            let formatted = format_screen(&lines, 5);
            assert_eq!(
                formatted,
                vec![
                    "  r00 |line·1|".to_string(),
                    "  r01 |line·2|".to_string(),
                    "  r02 |line·3|".to_string(),
                    "  r03 <missing>".to_string(),
                    "  r04 <missing>".to_string(),
                ]
            );
        }
    }

    mod scenario_run {
        use super::*;

        mod write_to {
            use super::*;

            #[test]
            fn write_to_matches_print_screen_style() {
                let run = ScenarioRun {
                    scenario_name: "middle_insert_wrap".to_string(),
                    target_name: "zsh".to_string(),
                    records: vec![
                        ScenarioRecord {
                            label: "type text".to_string(),
                            screen: vec!["  r00 |hello|".to_string(), "  r01 |world|".to_string()],
                        },
                        ScenarioRecord {
                            label: "insert text".to_string(),
                            screen: vec!["  r00 |hello again|".to_string()],
                        },
                    ],
                };

                let mut output = Vec::new();
                run.write_to(&mut output).unwrap();

                assert_eq!(
                    String::from_utf8(output).unwrap(),
                    "== type text ==\n  r00 |hello|\n  r01 |world|\n\n== insert text ==\n  r00 |hello again|\n"
                );
            }
        }
    }
}

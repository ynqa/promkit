use std::{
    fs::File,
    io::{self, Read},
    path::PathBuf,
};

use clap::Parser;
use promkit::{
    Prompt,
    core::crossterm::{event, execute, terminal},
    preset::yaml::Yaml,
    widgets::{
        serde_yaml::{Deserializer, Value},
        yaml_tree::{YamlTree, config::OverflowMode},
    },
};
use serde::Deserialize;

/// Interactive YAML viewer powered by promkit.
#[derive(Debug, Parser)]
#[command(name = "yaml", version)]
struct Args {
    /// Optional path to a YAML file. Reads from stdin when omitted or when "-" is specified.
    input: Option<PathBuf>,
}

/// Read YAML input from a file or stdin based on the provided arguments.
fn parse_input(args: &Args) -> anyhow::Result<String> {
    let mut input = String::new();

    match &args.input {
        None => {
            io::stdin().read_to_string(&mut input)?;
        }
        Some(path) if path == &PathBuf::from("-") => {
            io::stdin().read_to_string(&mut input)?;
        }
        Some(path) => {
            File::open(path)?.read_to_string(&mut input)?;
        }
    }

    Ok(input)
}

/// Parse a YAML string into a vector of serde_yaml::Value,
/// allowing for multiple YAML documents in the input.
fn parse_yaml_values(input: &str) -> anyhow::Result<Vec<Value>> {
    Deserializer::from_str(input)
        .map(Value::deserialize)
        .collect::<Result<Vec<_>, _>>()
        .map_err(anyhow::Error::from)
}

/// Ensure the terminal is restored to its original state when dropped.
struct TerminalGuard;

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = execute!(
            io::stdout(),
            terminal::LeaveAlternateScreen,
            event::DisableMouseCapture
        );
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let input = parse_input(&args)?;
    let values = parse_yaml_values(&input)?;

    execute!(
        io::stdout(),
        terminal::EnterAlternateScreen,
        event::EnableMouseCapture
    )?;
    let _terminal_guard = TerminalGuard;

    let tree = YamlTree::new(values.iter());
    Yaml::new(tree)
        .title("YAML Viewer")
        .overflow_mode(OverflowMode::Wrap)
        .run()
        .await
}

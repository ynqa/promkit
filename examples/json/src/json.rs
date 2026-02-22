use std::{
    fs::File,
    io::{self, Read},
    path::PathBuf,
};

use clap::Parser;
use promkit::{
    preset::json::Json,
    widgets::{
        jsonstream::JsonStream,
        serde_json::{self, Deserializer, Value},
    },
    Prompt,
};

/// Interactive JSON viewer powered by promkit.
#[derive(Debug, Parser)]
#[command(name = "json", version)]
struct Args {
    /// Optional path to a JSON file. Reads from stdin when omitted or when "-" is specified.
    input: Option<PathBuf>,

    /// Title shown in the JSON viewer.
    #[arg(short, long, default_value = "JSON viewer")]
    title: String,
}

/// Reads JSON input from a file or stdin based on the provided arguments.
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

/// Parses a JSON string into a vector of serde_json::Value,
/// allowing for multiple JSON objects in the input.
fn parse_json_stream(input: &str) -> anyhow::Result<Vec<Value>> {
    let stream: serde_json::StreamDeserializer<'_, serde_json::de::StrRead<'_>, Value> =
        Deserializer::from_str(input).into_iter::<Value>();
    stream
        .collect::<Result<Vec<_>, _>>()
        .map_err(anyhow::Error::from)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let input = parse_input(&args)?;
    let values = parse_json_stream(&input)?;

    let stream = JsonStream::new(values.iter());
    Json::new(stream).title(args.title).run().await
}

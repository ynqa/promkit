use std::{env, io};

use promkit::core::crossterm::{self, cursor, terminal};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let (_, rows) = terminal::size()?;
    crossterm::execute!(io::stdout(), cursor::MoveTo(0, rows.saturating_sub(1)))?;
    zsh_pretend::run().await
}

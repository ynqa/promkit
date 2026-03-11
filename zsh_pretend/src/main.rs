use promkit::{
    core::crossterm::{cursor, terminal},
    preset::readline::Readline,
    Prompt,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    loop {
        match Readline::default().run().await {
            Ok(command) => {
                // Keep the prompt line intact when the cursor is already on the last row.
                let (_, y) = cursor::position()?;
                let (_, h) = terminal::size()?;
                if y >= h.saturating_sub(1) {
                    println!();
                }
                println!(
                    "zsh: command not found: {}",
                    strip_outer_quotes(command.trim())
                );
            }
            Err(error) => {
                println!("error: {error}");
                break;
            }
        }
    }

    Ok(())
}

/// Strip outer quotes from a command string, if present.
/// e.g. `"ls -la"` becomes `ls -la`
fn strip_outer_quotes(command: &str) -> &str {
    if command.len() >= 2 {
        if let Some(unquoted) = command
            .strip_prefix('"')
            .and_then(|inner| inner.strip_suffix('"'))
        {
            return unquoted;
        }

        if let Some(unquoted) = command
            .strip_prefix('\'')
            .and_then(|inner| inner.strip_suffix('\''))
        {
            return unquoted;
        }
    }

    command
}

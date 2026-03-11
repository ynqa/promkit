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
                println!("zsh: command not found: \"{command}\"");
            }
            Err(error) => {
                println!("error: {error}");
                break;
            }
        }
    }

    Ok(())
}

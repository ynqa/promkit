use promkit::{
    Prompt,
    core::crossterm::{cursor, terminal},
    preset::readline::Readline,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    loop {
        match Readline::default().run().await {
            Ok(cmd) => {
                // If the prompt is finalized on the last line, print one line-feed
                // first so the result does not overwrite the prompt line.
                let (_, y) = cursor::position()?;
                let (_, h) = terminal::size()?;
                if y >= h.saturating_sub(1) {
                    println!();
                }
                println!("result: {:?}", cmd);
            }
            Err(e) => {
                println!("error: {}", e);
                break;
            }
        }
    }
    Ok(())
}

use promkit::{Prompt, TerminalModes, TerminalSession};
use readline_example::Readline;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("Build completed successfully.");

    let mut prompt = Readline::default();
    prompt.title.text = "Hi!".into();
    let modes =
        TerminalModes::RAW_MODE | TerminalModes::HIDDEN_CURSOR | TerminalModes::MOUSE_CAPTURE;
    let _terminal_session = TerminalSession::try_new(modes)?;
    prompt.run().await?;
    Ok(())
}

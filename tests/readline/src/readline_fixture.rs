use promkit::{
    core::crossterm::{cursor, terminal},
    Prompt, TerminalModes, TerminalSession,
};
use readline_example::Readline;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    loop {
        let result = {
            let modes = TerminalModes::RAW_MODE
                | TerminalModes::HIDDEN_CURSOR
                | TerminalModes::MOUSE_CAPTURE;
            let _terminal_session = TerminalSession::try_new(modes)?;
            Readline::default().run().await
        };

        match result {
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

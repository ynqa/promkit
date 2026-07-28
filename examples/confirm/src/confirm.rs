use promkit::{validate::ValidatorManager, Prompt, TerminalModes, TerminalSession};
use readline_example::Readline;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut prompt = Readline::default();
    prompt.readline.config.prefix = "Do you have a pet? (y/n) ".into();
    prompt.validator = Some(ValidatorManager::new(
        |text| ["yes", "no", "y", "n", "Y", "N"].contains(&text),
        |_| "Please type 'y' or 'n' as an answer".into(),
    ));

    let ret = {
        let modes =
            TerminalModes::RAW_MODE | TerminalModes::HIDDEN_CURSOR | TerminalModes::MOUSE_CAPTURE;
        let _terminal_session = TerminalSession::try_new(modes)?;
        prompt.run().await?
    };
    println!("result: {:?}", ret);
    Ok(())
}

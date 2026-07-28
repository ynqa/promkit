use promkit::{
    validate::ValidatorManager, widgets::text::Text, Prompt, TerminalModes, TerminalSession,
};
use readline_example::Readline;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut prompt = Readline::default();
    prompt.title.text = Text::from("Put your password");
    prompt.readline.config.mask = Some('*');
    prompt.validator = Some(ValidatorManager::new(
        |text| 4 < text.len() && text.len() < 10,
        |text| format!("Length must be over 4 and within 10 but got {}", text.len()),
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

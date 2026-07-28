use promkit::{
    validate::ValidatorManager,
    widgets::{prefix_search::PrefixSearch, text::Text},
    Prompt, TerminalModes, TerminalSession,
};
use readline::Readline;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut prompt = Readline::default();
    prompt.title.text = Text::from("Hi!");
    prompt.suggestions.prefix_search =
        PrefixSearch::from_iter(["apple", "applet", "application", "banana"]);
    prompt.validator = Some(ValidatorManager::new(
        |text| text.len() > 10,
        |text| format!("Length must be over 10 but got {}", text.len()),
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

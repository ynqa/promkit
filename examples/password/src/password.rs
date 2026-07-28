use promkit::{validate::ValidatorManager, widgets::text::Text, Prompt};
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

    let ret = prompt.run().await?;
    println!("result: {:?}", ret);
    Ok(())
}

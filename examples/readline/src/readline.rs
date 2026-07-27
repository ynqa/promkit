use promkit::{suggest::Suggest, validate::ValidatorManager, widgets::text::Text, Prompt};
use readline::Readline;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut prompt = Readline::default();
    prompt.title.text = Text::from("Hi!");
    prompt.suggest = Some(Suggest::from_iter([
        "apple",
        "applet",
        "application",
        "banana",
    ]));
    prompt.validator = Some(ValidatorManager::new(
        |text| text.len() > 10,
        |text| format!("Length must be over 10 but got {}", text.len()),
    ));

    let ret = prompt.run().await?;
    println!("result: {:?}", ret);
    Ok(())
}

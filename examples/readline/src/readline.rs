use promkit::{preset::readline::Readline, suggest::Suggest, Prompt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let ret = Readline::default()
        .title("Hi!")
        .enable_suggest(Suggest::from_iter([
            "apple",
            "applet",
            "application",
            "banana",
        ]))
        .validator(
            |text| text.len() > 10,
            |text| format!("Length must be over 10 but got {}", text.len()),
        )
        .run()
        .await?;
    println!("result: {:?}", ret);
    Ok(())
}

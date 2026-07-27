use promkit::{validate::ValidatorManager, Prompt};
use readline_example::Readline;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut prompt = Readline::default();
    prompt.readline.config.prefix = "Do you have a pet? (y/n) ".into();
    prompt.validator = Some(ValidatorManager::new(
        |text| ["yes", "no", "y", "n", "Y", "N"].contains(&text),
        |_| "Please type 'y' or 'n' as an answer".into(),
    ));

    let ret = prompt.run().await?;
    println!("result: {:?}", ret);
    Ok(())
}

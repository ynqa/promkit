use promkit::Prompt;
use readline_example::Readline;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let ret = Readline::default()
        .prefix("Do you have a pet? (y/n) ")
        .validator(
            |text| ["yes", "no", "y", "n", "Y", "N"].contains(&text),
            |_| "Please type 'y' or 'n' as an answer".into(),
        )
        .run()
        .await?;
    println!("result: {:?}", ret);
    Ok(())
}

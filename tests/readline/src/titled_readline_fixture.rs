use promkit::Prompt;
use readline_example::Readline;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("Build completed successfully.");

    let mut prompt = Readline::default();
    prompt.title.text = "Hi!".into();
    prompt.run().await?;
    Ok(())
}

use promkit::{preset::text::Text, Prompt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    Text::try_default(std::fs::read_to_string("Cargo.toml")?)
        .await?
        .run()
        .await?;
    Ok(())
}

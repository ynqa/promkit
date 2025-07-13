use promkit::{preset::text::Text, Prompt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    Text::new(std::fs::read_to_string("Cargo.toml")?)
        .run()
        .await?;
    Ok(())
}

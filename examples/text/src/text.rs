use promkit::{preset::text::Text, Prompt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    Text::new_with_text(std::fs::read_to_string("Cargo.toml")?)
        .run()
        .await?;
    Ok(())
}

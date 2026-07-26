use promkit::{preset::text_editor::TextEditor, Prompt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    TextEditor::default()
        .prefix("❯❯❯ ")
        .continuation_prefix("... ")
        .lines(4)
        .run()
        .await?;
    Ok(())
}

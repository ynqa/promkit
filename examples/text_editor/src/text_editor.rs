use promkit::{preset::text_editor::TextEditor, Prompt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let text = TextEditor::default()
        .title("Enter text (Ctrl+D to submit)")
        .lines(8)
        .run()
        .await?;

    println!("result:\n{text}");
    Ok(())
}

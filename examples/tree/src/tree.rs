use std::path::Path;

use promkit::{preset::tree::Tree, widgets::structured::tree::Document, Prompt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../promkit/src");
    let document = Document::from_path(&root)?;
    let ret = Tree::new(document)
        .title("Select a directory or file")
        .show_line_numbers(true)
        .tree_lines(10)
        .run()
        .await?;
    println!("result: {:?}", ret);
    Ok(())
}

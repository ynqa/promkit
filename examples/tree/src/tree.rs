use std::path::Path;

use promkit::{preset::tree::Tree, widgets::tree::node::Node, Prompt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../promkit/src");
    let ret = Tree::new(Node::try_from(&root)?)
        .title("Select a directory or file")
        .tree_lines(10)
        .run()
        .await?;
    println!("result: {:?}", ret);
    Ok(())
}

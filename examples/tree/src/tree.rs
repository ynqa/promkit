use promkit::{preset::tree::Tree, widgets::tree::node::Node, Prompt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let ret = Tree::new(Node::try_from(&std::env::current_dir()?.join("src"))?)
        .title("Select a directory or file")
        .tree_lines(10)
        .run()
        .await?;
    println!("result: {:?}", ret);
    Ok(())
}

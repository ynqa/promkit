use promkit::{preset::tree::Tree, promkit_widgets::tree::node::Node};

fn main() -> anyhow::Result<()> {
    let mut p = Tree::new(Node::try_from(&std::env::current_dir()?.join("src"))?)
        .title("Select a directory or file")
        .tree_lines(10)
        .prompt()?;
    println!("result: {:?}", p.run()?);
    Ok(())
}

use promkit::{error::Result, preset::Tree, tree::Node};

fn main() -> Result {
    let mut p = Tree::new(Node::new("/").add_children([
        Node::new("foo").add_children([Node::new("test1.txt"), Node::new("test2.txt")]),
        Node::new("bar"),
        Node::new("baz"),
    ]))
    .title("Select a directory or file")
    .tree_lines(10)
    .prompt()?;
    println!("result: {:?}", p.run()?);
    Ok(())
}

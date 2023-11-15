use promkit::{error::Result, preset::Tree, tree::Node};

fn main() -> Result {
    let mut p = Tree::new(Node::new("/").add_children([
        Node::new("a").add_children([Node::new("aa"), Node::new("ab")]),
        Node::new("b"),
        Node::new("c"),
    ]))
    .title("What number do you like?")
    .lines(10)
    .prompt()?;
    println!("result: {:?}", p.run()?);
    Ok(())
}

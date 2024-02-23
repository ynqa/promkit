use promkit::{error::Result, preset::Tree, tree::Node};

fn main() -> Result {
    let mut p = Tree::new(Node::NonLeaf {
        id: "root".into(),
        children: vec![
            Node::NonLeaf {
                id: "a".into(),
                children: vec![Node::Leaf("aa".into()), Node::Leaf("ab".into())],
                children_visible: true,
            },
            Node::Leaf("b".into()),
            Node::Leaf("c".into()),
        ],
        children_visible: true,
    })
    .title("Select a directory or file")
    .tree_lines(10)
    .prompt()?;
    println!("result: {:?}", p.run()?);
    Ok(())
}

/// A `Node` struct that represents a single node in a tree structure.
/// It contains data as a `String`,
/// a list of child nodes, and a visibility flag for its children.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Node {
    data: String,
    children: Vec<Node>,
    children_visible: bool,
}

/// A `NodeWithDepth` struct that represents a node
/// along with its depth information.
/// It is used for flattening a tree structure into a linear representation.
#[derive(Clone, Debug, PartialEq)]
pub struct NodeWithDepth {
    pub data: String,
    pub data_from_root: Vec<String>,
    pub children_visible: bool,
    pub is_leaf: bool,
}

impl Node {
    /// Constructs a new `Node` with the given data.
    pub fn new<T: AsRef<str>>(string: T) -> Self {
        Self {
            data: string.as_ref().to_string(),
            children: Vec::new(),
            children_visible: false,
        }
    }

    /// Adds a list of child nodes to the current node.
    pub fn add_children<I: IntoIterator<Item = Node>>(mut self, nodes: I) -> Self {
        for node in nodes {
            self.children.push(node);
        }
        self
    }

    /// Flattens the tree structure starting from this node into a vector of `NodeWithDepth`.
    pub fn flatten(&self) -> Vec<NodeWithDepth> {
        self.flatten_with_depth(vec![])
    }

    fn flatten_with_depth(&self, mut data_from_root: Vec<String>) -> Vec<NodeWithDepth> {
        let mut res = vec![];
        res.push(NodeWithDepth {
            data: self.data.clone(),
            data_from_root: data_from_root.clone(),
            children_visible: self.children_visible,
            is_leaf: self.children.is_empty(),
        });

        data_from_root.push(self.data.clone());
        if self.children_visible && !self.children.is_empty() {
            for child in self.children.iter() {
                res.extend(child.flatten_with_depth(data_from_root.clone()));
            }
        }
        res
    }

    fn traversal(&mut self, preorder: usize) -> Option<&mut Node> {
        if preorder == 0 {
            return Some(self);
        }

        if let Some(child) = self.children.iter_mut().next() {
            return child.traversal(preorder - 1);
        }
        None
    }

    /// Toggles the visibility of the children of the node found at the given preorder position.
    pub fn toggle(&mut self, preorder: usize) {
        if let Some(node) = self.traversal(preorder) {
            node.children_visible = !node.children_visible;
        }
    }
}

#[cfg(test)]
mod test {
    use super::Node;

    fn create_test_node() -> Node {
        Node::new("/").add_children([
            Node::new("a").add_children([Node::new("aa"), Node::new("ab")]),
            Node::new("b"),
            Node::new("c"),
        ])
    }

    mod flatten {
        use super::super::*;
        use super::create_test_node;

        #[test]
        fn test() {
            assert_eq!(
                vec![NodeWithDepth {
                    data: String::from("/"),
                    data_from_root: Vec::new(),
                    children_visible: false,
                    is_leaf: false,
                }],
                create_test_node().flatten()
            );
        }
    }

    mod traversal {
        use super::super::*;
        use super::create_test_node;

        #[test]
        fn test() {
            assert_eq!(
                Some(&mut Node::new("a").add_children([Node::new("aa"), Node::new("ab")])),
                create_test_node().traversal(1),
            );
        }
    }

    mod toggle {
        use super::super::*;
        use super::create_test_node;

        #[test]
        fn test() {
            let mut root = create_test_node();
            root.toggle(0);
            let expect = vec![
                NodeWithDepth {
                    data: String::from("/"),
                    data_from_root: Vec::new(),
                    children_visible: true,
                    is_leaf: false,
                },
                NodeWithDepth {
                    data: String::from("a"),
                    data_from_root: vec![String::from("/")],
                    children_visible: false,
                    is_leaf: false,
                },
                NodeWithDepth {
                    data: String::from("b"),
                    data_from_root: vec![String::from("/")],
                    children_visible: false,
                    is_leaf: true,
                },
                NodeWithDepth {
                    data: String::from("c"),
                    data_from_root: vec![String::from("/")],
                    children_visible: false,
                    is_leaf: true,
                },
            ];
            assert_eq!(expect, root.flatten());
        }
    }
}

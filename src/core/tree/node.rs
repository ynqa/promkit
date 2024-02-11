#[derive(Clone, Debug, Default, PartialEq)]
pub struct Node {
    data: String,
    children: Vec<Node>,
    children_visible: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct NodeWithDepth {
    pub data: String,
    pub depth: usize,
    pub children_visible: bool,
}

impl Node {
    pub fn new<T: AsRef<str>>(string: T) -> Self {
        Self {
            data: string.as_ref().to_string(),
            children: Vec::new(),
            children_visible: false,
        }
    }

    pub fn add_children<I: IntoIterator<Item = Node>>(mut self, nodes: I) -> Self {
        for node in nodes {
            self.children.push(node);
        }
        self
    }

    pub fn flatten(&self) -> Vec<NodeWithDepth> {
        self.flatten_with_depth(0)
    }

    fn flatten_with_depth(&self, depth: usize) -> Vec<NodeWithDepth> {
        let mut res = vec![];
        res.push(NodeWithDepth {
            data: self.data.clone(),
            depth,
            children_visible: self.children_visible,
        });

        if self.children_visible && !self.children.is_empty() {
            for child in self.children.iter() {
                res.extend(child.flatten_with_depth(depth + 1));
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
                    depth: 0,
                    children_visible: false,
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
                    depth: 0,
                    children_visible: true,
                },
                NodeWithDepth {
                    data: String::from("a"),
                    depth: 1,
                    children_visible: false,
                },
                NodeWithDepth {
                    data: String::from("b"),
                    depth: 1,
                    children_visible: false,
                },
                NodeWithDepth {
                    data: String::from("c"),
                    depth: 1,
                    children_visible: false,
                },
            ];
            assert_eq!(expect, root.flatten());
        }
    }
}

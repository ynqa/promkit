/// Represents the kind of a node in a tree structure.
///
/// This enum is used to distinguish between nodes that are currently
/// visible in their "folded" state and those that are "unfolded" to reveal
/// their children, if any.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Kind {
    /// Represents a node that is folded (i.e., its children are not currently visible).
    /// - `id`: A unique identifier for the node.
    /// - `path`: The path from the root to this node, represented as a sequence of indices.
    Folded { id: String, path: Path },

    /// Represents a node that is unfolded (i.e., its children are currently visible).
    /// - `id`: A unique identifier for the node.
    /// - `path`: The path from the root to this node, represented as a sequence of indices.
    Unfolded { id: String, path: Path },
}

/// A type alias for a path in the tree, represented as a sequence of indices.
pub type Path = Vec<usize>;

/// Represents a node within a tree structure.
///
/// A node can either be a `NonLeaf`, containing children and a visibility flag,
/// or a `Leaf`, representing an end node without children.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Node {
    /// Represents a non-leaf node, which can contain child nodes.
    /// - `id`: A unique identifier for the node.
    /// - `children`: A vector of child nodes.
    /// - `children_visible`: A boolean indicating if the children of this node are visible.
    NonLeaf {
        id: String,
        children: Vec<Node>,
        children_visible: bool,
    },

    /// Represents a leaf node, which does not contain any child nodes.
    /// - `id`: A unique identifier for the leaf node.
    Leaf(String),
}

impl Node {
    /// Flattens the tree structure into a vector of `Kind`, including only visible nodes.
    ///
    /// This method performs a depth-first search (DFS) to traverse the tree and collect
    /// nodes into a vector. Each node is represented as either `Kind::Folded` or `Kind::Unfolded`
    /// based on its visibility and whether it has children.
    ///
    /// Returns:
    /// - Vec<Kind>: A vector of `Kind` representing the visible nodes in the tree.
    pub fn flatten_visibles(&self) -> Vec<Kind> {
        fn dfs(node: &Node, path: Path, ret: &mut Vec<Kind>) {
            match node {
                Node::NonLeaf {
                    id,
                    children,
                    children_visible,
                } => {
                    if *children_visible {
                        ret.push(Kind::Unfolded {
                            id: id.clone(),
                            path: path.clone(),
                        });
                        for (index, child) in children.iter().enumerate() {
                            let mut new_path = path.clone();
                            new_path.push(index);
                            dfs(child, new_path, ret);
                        }
                    } else {
                        ret.push(Kind::Folded {
                            id: id.clone(),
                            path: path.clone(),
                        });
                    }
                }
                Node::Leaf(item) => {
                    ret.push(Kind::Folded {
                        id: item.clone(),
                        path: path.clone(),
                    });
                }
            }
        }

        let mut ret = Vec::new();
        dfs(self, Vec::new(), &mut ret);
        ret
    }

    /// Toggles the visibility of the children of the node specified by the given path.
    ///
    /// Parameters:
    /// - path: &Path - A reference to a vector of usize, representing the path to the target node.
    ///
    /// This method modifies the tree in-place. If the target node is found and is a `NonLeaf`,
    /// its `children_visible` field is toggled.
    pub fn toggle(&mut self, path: &Path) {
        if let Some(Node::NonLeaf {
            children_visible, ..
        }) = self.get_mut(path)
        {
            *children_visible = !*children_visible;
        }
    }

    /// Retrieves the IDs of all nodes along the path to a specified node.
    ///
    /// Parameters:
    /// - path: &Path - A reference to a vector of usize, representing the path to the target node.
    ///
    /// Returns:
    /// - Vec<String>: A vector of String IDs representing the nodes along the path to the target node.
    pub fn get_waypoints(&self, path: &Path) -> Vec<String> {
        let mut ids = Vec::new();
        let mut node = self;
        for &index in path {
            match node {
                Node::NonLeaf { id, children, .. } => {
                    ids.push(id.clone());
                    if let Some(child) = children.get(index) {
                        node = child;
                    } else {
                        break;
                    }
                }
                Node::Leaf(id) => {
                    ids.push(id.clone());
                    break;
                }
            }
        }
        ids
    }

    /// Retrieves a reference to the node specified by the given path.
    ///
    /// Parameters:
    /// - path: &Path - A reference to a vector of usize, representing the path to the target node.
    ///
    /// Returns:
    /// - Option<&Node>: An option containing a reference to the target node if found, or None otherwise.
    pub fn get(&self, path: &Path) -> Option<&Node> {
        let mut node = self;
        for seg in path {
            match node {
                Node::NonLeaf {
                    id: _,
                    children,
                    children_visible: _,
                } => {
                    if let Some(next_node) = children.get(*seg) {
                        node = next_node;
                    } else {
                        return None;
                    }
                }
                Node::Leaf(_) => {
                    return None;
                }
            }
        }
        Some(node)
    }

    /// Retrieves a mutable reference to the node specified by the given path.
    ///
    /// Parameters:
    /// - path: &Path - A reference to a vector of usize, representing the path to the target node.
    ///
    /// Returns:
    /// - Option<&mut Node>: An option containing a mutable reference to the target node if found, or None otherwise.
    pub fn get_mut(&mut self, path: &Path) -> Option<&mut Node> {
        let mut node = self;
        for seg in path {
            match node {
                Node::NonLeaf {
                    id: _,
                    children,
                    children_visible: _,
                } => {
                    if let Some(next_node) = children.get_mut(*seg) {
                        node = next_node;
                    } else {
                        return None;
                    }
                }
                Node::Leaf(_) => {
                    return None;
                }
            }
        }
        Some(node)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn create_test_node() -> Node {
        Node::NonLeaf {
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
        }
    }

    fn as_nonleaf(node: &Node) -> Option<(&String, &Vec<Node>, bool)> {
        match node {
            Node::NonLeaf {
                id,
                children,
                children_visible,
            } => Some((id, children, *children_visible)),
            _ => None,
        }
    }

    mod toggle {
        use super::*;

        #[test]
        fn test() {
            let mut node = create_test_node();
            node.toggle(&vec![]);
            assert!(!as_nonleaf(node.get(&vec![]).unwrap()).unwrap().2);
        }
    }

    mod flatten_visibles {
        use super::*;

        #[test]
        fn test() {
            let node = create_test_node();
            assert_eq!(
                vec![
                    Kind::Unfolded {
                        id: "root".into(),
                        path: vec![],
                    },
                    Kind::Unfolded {
                        id: "a".into(),
                        path: vec![0],
                    },
                    Kind::Folded {
                        id: "aa".into(),
                        path: vec![0, 0],
                    },
                    Kind::Folded {
                        id: "ab".into(),
                        path: vec![0, 1],
                    },
                    Kind::Folded {
                        id: "b".into(),
                        path: vec![1],
                    },
                    Kind::Folded {
                        id: "c".into(),
                        path: vec![2],
                    },
                ],
                node.flatten_visibles(),
            );
        }

        #[test]
        fn test_after_toggle() {
            let mut node = create_test_node();
            node.toggle(&vec![]);
            assert_eq!(
                vec![Kind::Folded {
                    id: "root".into(),
                    path: vec![],
                },],
                node.flatten_visibles(),
            );
        }
    }
}

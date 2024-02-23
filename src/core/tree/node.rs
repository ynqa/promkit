#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Kind {
    Folded { id: String, path: Path },
    Unfolded { id: String, path: Path },
}

pub type Path = Vec<usize>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Node {
    NonLeaf {
        id: String,
        children: Vec<Node>,
        children_visible: bool,
    },
    Leaf(String),
}

impl Node {
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

    pub fn toggle(&mut self, path: &Path) {
        if let Some(Node::NonLeaf {
            children_visible, ..
        }) = self.get_mut(path)
        {
            *children_visible = !*children_visible;
        }
    }

    pub fn path_to_ids(&self, path: &Path) -> Vec<String> {
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

    pub fn get_mut(&mut self, path: &Path) -> Option<&mut Node> {
        let mut node = self;
        for seg in path {
            match node {
                Node::NonLeaf {
                    id: _,
                    children,
                    children_visible: _,
                } => {
                    node = children.get_mut(*seg).unwrap();
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

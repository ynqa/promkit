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
        fn dfs(node: &Node, path: Path, ret: &mut Vec<Kind>, visible: bool) {
            match node {
                Node::NonLeaf {
                    id,
                    children,
                    children_visible,
                } => {
                    if visible {
                        ret.push(Kind::Unfolded {
                            id: id.clone(),
                            path: path.clone(),
                        });
                    } else {
                        ret.push(Kind::Folded {
                            id: id.clone(),
                            path: path.clone(),
                        });
                    }
                    if *children_visible {
                        for (index, child) in children.iter().enumerate() {
                            let mut new_path = path.clone();
                            new_path.push(index);
                            dfs(child, new_path, ret, true);
                        }
                    }
                }
                Node::Leaf(item) => {
                    if visible {
                        ret.push(Kind::Unfolded {
                            id: item.clone(),
                            path: path.clone(),
                        });
                    }
                }
            }
        }

        let mut ret = Vec::new();
        dfs(self, Vec::new(), &mut ret, true); // Root is always visible
        ret
    }

    pub fn toggle(&mut self, path: &Path) {
        if let Some(node) = self.get_mut(path) {
            match node {
                Node::NonLeaf {
                    children_visible, ..
                } => {
                    *children_visible = !*children_visible;
                }
                _ => {}
            }
        }
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
}

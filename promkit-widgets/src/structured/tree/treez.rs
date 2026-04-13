use crate::structured::RowOperation;

/// A single visible-or-collapsible row in a tree view.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Row {
    /// Depth from the root node. Used for indentation when rendering.
    pub depth: usize,
    /// Display label of the current node.
    pub id: String,
    /// Breadcrumb-like labels from the root to this node.
    pub path: Vec<String>,
    /// Whether this row has child nodes.
    pub has_children: bool,
    /// Whether the children of this row are currently collapsed.
    pub collapsed: bool,
}

/// Adapts an arbitrary tree-shaped data source into rows.
pub trait Adapter {
    /// Input node type used by the adapted tree source.
    type Node;
    /// Error returned while reading node metadata or children.
    type Error;

    /// Returns the display label for the given node.
    fn id_of(&self, node: &Self::Node) -> Result<String, Self::Error>;
    /// Returns the direct children of the given node.
    fn children_of(&self, node: &Self::Node) -> Result<Vec<Self::Node>, Self::Error>;
}

fn collect_rows_with<T, E, A>(
    input: &T,
    depth: usize,
    current_path: &mut Vec<String>,
    rows: &mut Vec<Row>,
    adapter: &A,
) -> Result<(), E>
where
    A: Adapter<Node = T, Error = E>,
{
    let id = adapter.id_of(input)?;
    let children = adapter.children_of(input)?;
    let has_children = !children.is_empty();

    current_path.push(id.clone());
    rows.push(Row {
        depth,
        id,
        path: current_path.clone(),
        has_children,
        collapsed: has_children,
    });

    if has_children {
        for child in &children {
            collect_rows_with(child, depth + 1, current_path, rows, adapter)?;
        }
    }

    current_path.pop();
    Ok(())
}

/// Creates tree rows from an arbitrary tree source via an [`Adapter`].
///
/// Parent rows are emitted before their descendants, and rows with children
/// start in the collapsed state by default.
pub fn create_rows<T, E, A>(root: &T, adapter: &A) -> Result<Vec<Row>, E>
where
    A: Adapter<Node = T, Error = E>,
{
    let mut rows = Vec::new();
    let mut current_path = Vec::new();
    collect_rows_with(root, 0, &mut current_path, &mut rows, adapter)?;
    Ok(rows)
}

fn is_visible(rows: &[Row], index: usize) -> bool {
    if index >= rows.len() {
        return false;
    }

    let mut ancestor_depth = rows[index].depth;
    for row in rows[..index].iter().rev() {
        if row.depth < ancestor_depth {
            if row.has_children && row.collapsed {
                return false;
            }
            ancestor_depth = row.depth;
            if ancestor_depth == 0 {
                break;
            }
        }
    }
    true
}

impl RowOperation for Vec<Row> {
    type Row = Row;

    fn up(&self, current: usize) -> usize {
        if self.is_empty() || current == 0 {
            return 0;
        }

        let mut prev = current - 1;
        loop {
            if is_visible(self, prev) {
                return prev;
            }
            if prev == 0 {
                return current;
            }
            prev -= 1;
        }
    }

    fn head(&self) -> usize {
        self.iter()
            .enumerate()
            .find_map(|(index, _)| is_visible(self, index).then_some(index))
            .unwrap_or(0)
    }

    fn down(&self, current: usize) -> usize {
        if self.is_empty() || current >= self.len().saturating_sub(1) {
            return current;
        }

        let mut next = current + 1;
        while next < self.len() {
            if is_visible(self, next) {
                return next;
            }
            next += 1;
        }
        current
    }

    fn tail(&self) -> usize {
        self.iter()
            .enumerate()
            .rev()
            .find_map(|(index, _)| is_visible(self, index).then_some(index))
            .unwrap_or(0)
    }

    fn toggle(&mut self, current: usize) -> usize {
        let Some(row) = self.get_mut(current) else {
            return current;
        };
        if row.has_children {
            row.collapsed = !row.collapsed;
        }
        current
    }

    fn set_rows_visibility(&mut self, collapsed: bool) {
        for row in self.iter_mut() {
            if row.has_children {
                row.collapsed = collapsed;
            }
        }
    }

    fn extract(&self, current: usize, n: usize) -> Vec<Row> {
        let mut result = Vec::new();
        let mut index = current;

        while index < self.len() && result.len() < n {
            if is_visible(self, index) {
                result.push(self[index].clone());
            }
            index += 1;
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone)]
    struct TestNode {
        id: &'static str,
        children: Vec<TestNode>,
    }

    struct TestAdapter;

    impl Adapter for TestAdapter {
        type Node = TestNode;
        type Error = std::convert::Infallible;

        fn id_of(&self, node: &Self::Node) -> Result<String, Self::Error> {
            Ok(node.id.to_string())
        }

        fn children_of(&self, node: &Self::Node) -> Result<Vec<Self::Node>, Self::Error> {
            Ok(node.children.clone())
        }
    }

    fn create_test_rows() -> Vec<Row> {
        vec![
            Row {
                depth: 0,
                id: "root".into(),
                path: vec!["root".into()],
                has_children: true,
                collapsed: false,
            },
            Row {
                depth: 1,
                id: "a".into(),
                path: vec!["root".into(), "a".into()],
                has_children: true,
                collapsed: false,
            },
            Row {
                depth: 2,
                id: "aa".into(),
                path: vec!["root".into(), "a".into(), "aa".into()],
                has_children: false,
                collapsed: false,
            },
            Row {
                depth: 2,
                id: "ab".into(),
                path: vec!["root".into(), "a".into(), "ab".into()],
                has_children: false,
                collapsed: false,
            },
            Row {
                depth: 1,
                id: "b".into(),
                path: vec!["root".into(), "b".into()],
                has_children: false,
                collapsed: false,
            },
        ]
    }

    #[test]
    fn extract_skips_hidden_descendants() {
        let mut rows = create_test_rows();
        rows[0].collapsed = true;

        assert_eq!(
            rows.extract(0, 5),
            vec![Row {
                depth: 0,
                id: "root".into(),
                path: vec!["root".into()],
                has_children: true,
                collapsed: true,
            }]
        );
    }

    #[test]
    fn down_skips_hidden_descendants() {
        let mut rows = create_test_rows();
        rows[1].collapsed = true;

        assert_eq!(rows.down(1), 4);
    }

    #[test]
    fn create_rows_is_generic() {
        let root = TestNode {
            id: "root",
            children: vec![
                TestNode {
                    id: "a",
                    children: vec![
                        TestNode {
                            id: "aa",
                            children: vec![],
                        },
                        TestNode {
                            id: "ab",
                            children: vec![],
                        },
                    ],
                },
                TestNode {
                    id: "b",
                    children: vec![],
                },
            ],
        };

        let rows = create_rows(&root, &TestAdapter).unwrap();

        assert_eq!(
            rows,
            vec![
                Row {
                    depth: 0,
                    id: "root".into(),
                    path: vec!["root".into()],
                    has_children: true,
                    collapsed: true,
                },
                Row {
                    depth: 1,
                    id: "a".into(),
                    path: vec!["root".into(), "a".into()],
                    has_children: true,
                    collapsed: true,
                },
                Row {
                    depth: 2,
                    id: "aa".into(),
                    path: vec!["root".into(), "a".into(), "aa".into()],
                    has_children: false,
                    collapsed: false,
                },
                Row {
                    depth: 2,
                    id: "ab".into(),
                    path: vec!["root".into(), "a".into(), "ab".into()],
                    has_children: false,
                    collapsed: false,
                },
                Row {
                    depth: 1,
                    id: "b".into(),
                    path: vec!["root".into(), "b".into()],
                    has_children: false,
                    collapsed: false,
                },
            ]
        );
    }
}

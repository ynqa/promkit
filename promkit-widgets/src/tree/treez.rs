use std::{fs, path};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Row {
    pub depth: usize,
    pub id: String,
    pub path: Vec<String>,
    pub has_children: bool,
    pub collapsed: bool,
}

pub trait RowOperation {
    type Row;

    fn up(&self, current: usize) -> usize;
    fn head(&self) -> usize;
    fn down(&self, current: usize) -> usize;
    fn tail(&self) -> usize;
    fn toggle(&mut self, current: usize) -> usize;
    fn set_rows_visibility(&mut self, collapsed: bool);
    fn extract(&self, current: usize, n: usize) -> Vec<Self::Row>;
}

fn collect_rows_with<T, E, FId, FChildren>(
    input: &T,
    depth: usize,
    current_path: &mut Vec<String>,
    rows: &mut Vec<Row>,
    id_of: &FId,
    children_of: &FChildren,
) -> Result<(), E>
where
    FId: Fn(&T) -> Result<String, E>,
    FChildren: Fn(&T) -> Result<Vec<T>, E>,
{
    let id = id_of(input)?;
    let children = children_of(input)?;
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
            collect_rows_with(child, depth + 1, current_path, rows, id_of, children_of)?;
        }
    }

    current_path.pop();
    Ok(())
}

pub fn create_rows<T, E, FId, FChildren>(
    root: &T,
    id_of: FId,
    children_of: FChildren,
) -> Result<Vec<Row>, E>
where
    FId: Fn(&T) -> Result<String, E>,
    FChildren: Fn(&T) -> Result<Vec<T>, E>,
{
    let mut rows = Vec::new();
    let mut current_path = Vec::new();
    collect_rows_with(root, 0, &mut current_path, &mut rows, &id_of, &children_of)?;
    Ok(rows)
}

fn path_id(input: &path::Path) -> anyhow::Result<String> {
    input
        .file_name()
        .and_then(|name| name.to_str())
        .map(ToOwned::to_owned)
        .or_else(|| {
            let rendered = input.display().to_string();
            (!rendered.is_empty()).then_some(rendered)
        })
        .ok_or_else(|| anyhow::anyhow!("Failed to convert path to string"))
}

pub fn create_rows_from_path(input: &path::Path) -> anyhow::Result<Vec<Row>> {
    create_rows(
        &input.to_path_buf(),
        |path: &path::PathBuf| path_id(path),
        |path| {
            if !path.is_dir() {
                return Ok(Vec::new());
            }

            let mut directories = Vec::new();
            let mut files = Vec::new();

            for entry in fs::read_dir(path)? {
                let path = entry?.path();
                if path.is_dir() {
                    directories.push(path);
                } else if path.is_file() {
                    files.push(path);
                }
            }

            directories.sort_by(|a, b| a.file_name().cmp(&b.file_name()));
            files.sort_by(|a, b| a.file_name().cmp(&b.file_name()));
            directories.extend(files);

            Ok(directories)
        },
    )
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

        let rows = create_rows(
            &root,
            |node| Ok::<_, std::convert::Infallible>(node.id.to_string()),
            |node| Ok::<_, std::convert::Infallible>(node.children.clone()),
        )
        .unwrap();

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

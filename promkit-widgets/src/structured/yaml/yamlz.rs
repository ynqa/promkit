use rayon::prelude::*;

use crate::structured::path::{append_bracket, append_string_key};

pub use crate::structured::{ContainerNode, ContainerType, RowOperation};

#[derive(Clone, Debug, PartialEq)]
pub enum YamlNode {
    Tagged { tag: String, node: Box<YamlNode> },
    DocumentSeparator,
    Null,
    Boolean(bool),
    Number(serde_yaml::Number),
    String(String),
    Container(ContainerNode),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Row {
    pub depth: usize,
    pub key: Option<String>,
    pub node: YamlNode,
    pub is_sequence_item: bool,
}

/// YAML tags can wrap container nodes (`Tagged(Container(...))`).
/// This helper centralizes "unwrap/rewrap while preserving tag" behavior.
pub(super) struct TagAwareContainer;

impl TagAwareContainer {
    pub(super) fn get(node: &YamlNode) -> Option<&ContainerNode> {
        match node {
            YamlNode::Container(container) => Some(container),
            YamlNode::Tagged { node, .. } => Self::get(node),
            _ => None,
        }
    }

    pub(super) fn replace(node: &YamlNode, new_container: ContainerNode) -> Option<YamlNode> {
        match node {
            YamlNode::Container(_) => Some(YamlNode::Container(new_container)),
            YamlNode::Tagged { tag, node } => Some(YamlNode::Tagged {
                tag: tag.clone(),
                node: Box::new(Self::replace(node, new_container)?),
            }),
            _ => None,
        }
    }
}

fn renders_as_sequence_mapping_line(row: &Row, next_row: &Row) -> bool {
    matches!(
        row.node,
        YamlNode::Container(ContainerNode::Open {
            typ: ContainerType::Object,
            collapsed: false,
            ..
        })
    ) && row.is_sequence_item
        && next_row.depth == row.depth + 1
        && !next_row.is_sequence_item
        && next_row.key.is_some()
}

fn sequence_mapping_line_start(rows: &[Row], index: usize) -> Option<usize> {
    let previous = index.checked_sub(1)?;
    renders_as_sequence_mapping_line(&rows[previous], &rows[index]).then_some(previous)
}

pub(super) fn sequence_mapping_line_start_for_path(rows: &[Row], index: usize) -> Option<usize> {
    let previous = index.checked_sub(1)?;
    let row = &rows[previous];
    let next_row = &rows[index];
    (matches!(
        TagAwareContainer::get(&row.node),
        Some(ContainerNode::Open {
            typ: ContainerType::Object,
            ..
        })
    ) && row.is_sequence_item
        && next_row.depth == row.depth + 1
        && !next_row.is_sequence_item
        && next_row.key.is_some())
    .then_some(previous)
}

fn sequence_mapping_inline_row(rows: &[Row], index: usize) -> Option<usize> {
    let next = index + 1;
    let next_row = rows.get(next)?;
    renders_as_sequence_mapping_line(&rows[index], next_row).then_some(next)
}

fn sequence_mapping_inline_container(rows: &[Row], index: usize) -> Option<usize> {
    let inline = sequence_mapping_inline_row(rows, index)?;
    matches!(
        TagAwareContainer::get(&rows[inline].node),
        Some(ContainerNode::Open { .. })
    )
    .then_some(inline)
}

pub(super) fn is_invisible_root_container(row: &Row) -> bool {
    row.depth == 0
        && row.key.is_none()
        && !row.is_sequence_item
        && matches!(
            row.node,
            YamlNode::Container(ContainerNode::Open {
                collapsed: false,
                ..
            })
        )
}

fn next_index_after(rows: &[Row], index: usize) -> usize {
    match TagAwareContainer::get(&rows[index].node) {
        Some(ContainerNode::Open {
            collapsed: true,
            close_index,
            ..
        }) => close_index + 1,
        _ => index + 1,
    }
}

impl RowOperation for Vec<Row> {
    type Row = Row;

    fn up(&self, current: usize) -> usize {
        if self.is_empty() || current == 0 {
            return 0;
        }

        let mut prev = current - 1;
        loop {
            match TagAwareContainer::get(&self[prev].node) {
                Some(ContainerNode::Close {
                    collapsed: true,
                    open_index,
                    ..
                }) => prev = *open_index,
                Some(ContainerNode::Close { .. }) if prev > 0 => {
                    prev -= 1;
                    continue;
                }
                Some(ContainerNode::Close { .. }) => return current,
                _ => {}
            }

            if is_invisible_root_container(&self[prev]) {
                if prev == 0 {
                    return current;
                }
                prev -= 1;
                continue;
            }

            return sequence_mapping_line_start(self, prev).unwrap_or(prev);
        }
    }

    fn head(&self) -> usize {
        self.iter()
            .enumerate()
            .position(|(index, row)| {
                !matches!(
                    TagAwareContainer::get(&row.node),
                    Some(ContainerNode::Close { .. })
                ) && !is_invisible_root_container(row)
                    && sequence_mapping_line_start(self, index).is_none()
            })
            .unwrap_or(0)
    }

    fn down(&self, current: usize) -> usize {
        if self.is_empty() || current >= self.len().saturating_sub(1) {
            return current;
        }

        let mut next = next_index_after(self, current);

        while next < self.len() {
            if matches!(
                TagAwareContainer::get(&self[next].node),
                Some(ContainerNode::Close { .. })
            ) {
                next += 1;
                continue;
            }

            if is_invisible_root_container(&self[next]) {
                next = next_index_after(self, next);
                continue;
            }

            // The first mapping key is rendered on its sequence item's line,
            // so it is not an independent cursor stop.
            if sequence_mapping_line_start(self, next).is_some() {
                next = next_index_after(self, next);
                continue;
            }

            return next;
        }

        current
    }

    fn tail(&self) -> usize {
        if self.is_empty() {
            return 0;
        }

        let mut current = self.head();
        loop {
            let next = self.down(current);
            if next == current {
                return current;
            }
            current = next;
        }
    }

    fn toggle(&mut self, current: usize) -> usize {
        let cursor = sequence_mapping_line_start(self, current).unwrap_or(current);
        let inline_row = sequence_mapping_inline_row(self, cursor);
        let inline_target = sequence_mapping_inline_container(self, cursor);
        let target = match inline_row {
            Some(_) => inline_target,
            None => TagAwareContainer::get(&self[cursor].node)
                .is_some()
                .then_some(cursor),
        };
        let Some(target) = target else {
            return cursor;
        };
        let container = TagAwareContainer::get(&self[target].node).cloned();

        match container {
            Some(ContainerNode::Open {
                typ,
                collapsed,
                close_index,
            }) => {
                let new_collapsed = !collapsed;

                self[target].node = TagAwareContainer::replace(
                    &self[target].node,
                    ContainerNode::Open {
                        typ: typ.clone(),
                        collapsed: new_collapsed,
                        close_index,
                    },
                )
                .expect("container open node must be present");

                self[close_index].node = TagAwareContainer::replace(
                    &self[close_index].node,
                    ContainerNode::Close {
                        typ,
                        collapsed: new_collapsed,
                        open_index: target,
                    },
                )
                .expect("container close node must be present");

                if inline_target.is_some() {
                    cursor
                } else if !new_collapsed && is_invisible_root_container(&self[target]) {
                    self.down(target)
                } else {
                    target
                }
            }
            Some(ContainerNode::Close {
                typ,
                collapsed,
                open_index,
            }) => {
                let new_collapsed = !collapsed;

                self[target].node = TagAwareContainer::replace(
                    &self[target].node,
                    ContainerNode::Close {
                        typ: typ.clone(),
                        collapsed: new_collapsed,
                        open_index,
                    },
                )
                .expect("container close node must be present");

                self[open_index].node = TagAwareContainer::replace(
                    &self[open_index].node,
                    ContainerNode::Open {
                        typ,
                        collapsed: new_collapsed,
                        close_index: target,
                    },
                )
                .expect("container open node must be present");

                open_index
            }
            _ => cursor,
        }
    }

    fn set_rows_visibility(&mut self, collapsed: bool) {
        self.par_iter_mut().for_each(|row| {
            let container = TagAwareContainer::get(&row.node).cloned();
            match container {
                Some(ContainerNode::Open {
                    typ, close_index, ..
                }) => {
                    row.node = TagAwareContainer::replace(
                        &row.node,
                        ContainerNode::Open {
                            typ,
                            collapsed,
                            close_index,
                        },
                    )
                    .expect("container open node must be present");
                }
                Some(ContainerNode::Close {
                    typ, open_index, ..
                }) => {
                    row.node = TagAwareContainer::replace(
                        &row.node,
                        ContainerNode::Close {
                            typ,
                            collapsed,
                            open_index,
                        },
                    )
                    .expect("container close node must be present");
                }
                _ => {}
            }
        });
    }

    fn extract(&self, current: usize, n: usize) -> Vec<Row> {
        let mut result = Vec::new();
        let mut i = current;
        let mut rendered_rows = 0;

        while i < self.len()
            && (matches!(
                TagAwareContainer::get(&self[i].node),
                Some(ContainerNode::Close { .. })
            ) || is_invisible_root_container(&self[i]))
        {
            i += 1;
        }

        while i < self.len() && rendered_rows < n {
            let row = &self[i];
            if matches!(
                TagAwareContainer::get(&row.node),
                Some(ContainerNode::Close { .. })
            ) {
                i += 1;
                continue;
            }
            if is_invisible_root_container(row) {
                i = next_index_after(self, i);
                continue;
            }

            result.push(row.clone());

            let next_index = next_index_after(self, i);
            // The renderer combines these two source rows into one YAML line.
            // Include both while counting them as one requested visible row.
            if let Some(next_row) = self.get(next_index)
                && renders_as_sequence_mapping_line(row, next_row)
            {
                result.push(next_row.clone());
                i = next_index_after(self, next_index);
            } else {
                i = next_index;
            }

            rendered_rows += 1;
        }

        result
    }
}

pub(super) fn normalize_mapping_key_for_display(mapping_key: &serde_yaml::Value) -> Option<String> {
    match mapping_key {
        serde_yaml::Value::String(s) => Some(s.clone()),
        serde_yaml::Value::Number(n) => Some(n.to_string()),
        serde_yaml::Value::Bool(b) => Some(b.to_string()),
        serde_yaml::Value::Null => Some("null".to_string()),
        other => {
            let rendered = serde_yaml::to_string(other).ok()?;
            let compact = rendered
                .lines()
                .map(str::trim)
                .filter(|line| !line.is_empty() && *line != "---" && *line != "...")
                .collect::<Vec<_>>()
                .join(" ");
            (!compact.is_empty()).then_some(compact)
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) enum PathKeyKind {
    #[default]
    None,
    String,
    Number,
    Bool,
    Null,
    Unsupported,
}

impl PathKeyKind {
    pub(super) fn from_mapping_key(key: &serde_yaml::Value) -> Self {
        match key {
            serde_yaml::Value::String(_) => Self::String,
            serde_yaml::Value::Number(_) => Self::Number,
            serde_yaml::Value::Bool(_) => Self::Bool,
            serde_yaml::Value::Null => Self::Null,
            serde_yaml::Value::Tagged(_)
            | serde_yaml::Value::Sequence(_)
            | serde_yaml::Value::Mapping(_) => Self::Unsupported,
        }
    }
}

pub(super) struct IndexedRows {
    pub(super) rows: Vec<Row>,
    pub(super) path_key_kinds: Vec<PathKeyKind>,
}

fn push_indexed_row(
    rows: &mut Vec<Row>,
    path_key_kinds: &mut Vec<PathKeyKind>,
    row: Row,
    path_key_kind: PathKeyKind,
) -> usize {
    rows.push(row);
    path_key_kinds.push(path_key_kind);
    debug_assert_eq!(rows.len(), path_key_kinds.len());
    rows.len() - 1
}

fn process_value(
    value: &serde_yaml::Value,
    rows: &mut Vec<Row>,
    path_key_kinds: &mut Vec<PathKeyKind>,
    depth: usize,
    key: Option<String>,
    path_key_kind: PathKeyKind,
    is_sequence_item: bool,
) -> usize {
    match value {
        serde_yaml::Value::Tagged(tagged) => {
            let index = process_value(
                &tagged.value,
                rows,
                path_key_kinds,
                depth,
                key,
                path_key_kind,
                is_sequence_item,
            );
            rows[index].node = YamlNode::Tagged {
                tag: tagged.tag.to_string(),
                node: Box::new(rows[index].node.clone()),
            };
            index
        }
        serde_yaml::Value::Null => push_indexed_row(
            rows,
            path_key_kinds,
            Row {
                depth,
                key,
                is_sequence_item,
                node: YamlNode::Null,
            },
            path_key_kind,
        ),
        serde_yaml::Value::Bool(b) => push_indexed_row(
            rows,
            path_key_kinds,
            Row {
                depth,
                key,
                is_sequence_item,
                node: YamlNode::Boolean(*b),
            },
            path_key_kind,
        ),
        serde_yaml::Value::Number(n) => push_indexed_row(
            rows,
            path_key_kinds,
            Row {
                depth,
                key,
                is_sequence_item,
                node: YamlNode::Number(n.clone()),
            },
            path_key_kind,
        ),
        serde_yaml::Value::String(s) => push_indexed_row(
            rows,
            path_key_kinds,
            Row {
                depth,
                key,
                is_sequence_item,
                node: YamlNode::String(s.clone()),
            },
            path_key_kind,
        ),
        serde_yaml::Value::Sequence(seq) => {
            if seq.is_empty() {
                return push_indexed_row(
                    rows,
                    path_key_kinds,
                    Row {
                        depth,
                        key,
                        is_sequence_item,
                        node: YamlNode::Container(ContainerNode::Empty {
                            typ: ContainerType::Array,
                        }),
                    },
                    path_key_kind,
                );
            }

            let open_index = push_indexed_row(
                rows,
                path_key_kinds,
                Row {
                    depth,
                    key,
                    is_sequence_item,
                    node: YamlNode::Container(ContainerNode::Open {
                        typ: ContainerType::Array,
                        collapsed: false,
                        close_index: 0,
                    }),
                },
                path_key_kind,
            );

            for item in seq {
                process_value(
                    item,
                    rows,
                    path_key_kinds,
                    depth + 1,
                    None,
                    PathKeyKind::None,
                    true,
                );
            }

            let close_index = push_indexed_row(
                rows,
                path_key_kinds,
                Row {
                    depth,
                    key: None,
                    is_sequence_item: false,
                    node: YamlNode::Container(ContainerNode::Close {
                        typ: ContainerType::Array,
                        collapsed: false,
                        open_index,
                    }),
                },
                PathKeyKind::None,
            );

            rows[open_index].node = YamlNode::Container(ContainerNode::Open {
                typ: ContainerType::Array,
                collapsed: false,
                close_index,
            });

            open_index
        }
        serde_yaml::Value::Mapping(map) => {
            if map.is_empty() {
                return push_indexed_row(
                    rows,
                    path_key_kinds,
                    Row {
                        depth,
                        key,
                        is_sequence_item,
                        node: YamlNode::Container(ContainerNode::Empty {
                            typ: ContainerType::Object,
                        }),
                    },
                    path_key_kind,
                );
            }

            let open_index = push_indexed_row(
                rows,
                path_key_kinds,
                Row {
                    depth,
                    key,
                    is_sequence_item,
                    node: YamlNode::Container(ContainerNode::Open {
                        typ: ContainerType::Object,
                        collapsed: false,
                        close_index: 0,
                    }),
                },
                path_key_kind,
            );

            for (mapping_key, map_value) in map {
                let key = normalize_mapping_key_for_display(mapping_key);
                let path_key_kind = PathKeyKind::from_mapping_key(mapping_key);
                process_value(
                    map_value,
                    rows,
                    path_key_kinds,
                    depth + 1,
                    key,
                    path_key_kind,
                    false,
                );
            }

            let close_index = push_indexed_row(
                rows,
                path_key_kinds,
                Row {
                    depth,
                    key: None,
                    is_sequence_item: false,
                    node: YamlNode::Container(ContainerNode::Close {
                        typ: ContainerType::Object,
                        collapsed: false,
                        open_index,
                    }),
                },
                PathKeyKind::None,
            );

            rows[open_index].node = YamlNode::Container(ContainerNode::Open {
                typ: ContainerType::Object,
                collapsed: false,
                close_index,
            });

            open_index
        }
    }
}

pub fn create_rows<'a, T: IntoIterator<Item = &'a serde_yaml::Value>>(iter: T) -> Vec<Row> {
    create_indexed_rows(iter).rows
}

pub(super) fn create_indexed_rows<'a, T: IntoIterator<Item = &'a serde_yaml::Value>>(
    iter: T,
) -> IndexedRows {
    let mut rows = Vec::new();
    let mut path_key_kinds = Vec::new();
    for (index, value) in iter.into_iter().enumerate() {
        if index > 0 {
            push_indexed_row(
                &mut rows,
                &mut path_key_kinds,
                Row {
                    depth: 0,
                    key: None,
                    node: YamlNode::DocumentSeparator,
                    is_sequence_item: false,
                },
                PathKeyKind::None,
            );
        }
        process_value(
            value,
            &mut rows,
            &mut path_key_kinds,
            0,
            None,
            PathKeyKind::None,
            false,
        );
    }
    IndexedRows {
        rows,
        path_key_kinds,
    }
}

#[derive(Debug)]
pub struct PathIterator<'a> {
    stack: Vec<(String, &'a serde_yaml::Value)>,
}

impl Iterator for PathIterator<'_> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some((current_path, value)) = self.stack.pop() {
            match value {
                serde_yaml::Value::Tagged(tagged) => {
                    self.stack.push((current_path.clone(), &tagged.value));
                }
                serde_yaml::Value::Mapping(map) => {
                    for (key, val) in map {
                        match key {
                            serde_yaml::Value::String(key) => {
                                let new_path = append_string_key(&current_path, key);
                                self.stack.push((new_path, val));
                            }
                            serde_yaml::Value::Number(n) => {
                                self.stack
                                    .push((append_bracket(&current_path, &n.to_string()), val));
                            }
                            serde_yaml::Value::Bool(b) => {
                                self.stack
                                    .push((append_bracket(&current_path, &b.to_string()), val));
                            }
                            serde_yaml::Value::Null => {
                                self.stack
                                    .push((append_bracket(&current_path, "null"), val));
                            }
                            _ => {}
                        }
                    }
                }
                serde_yaml::Value::Sequence(seq) => {
                    for (i, val) in seq.iter().enumerate() {
                        self.stack
                            .push((append_bracket(&current_path, &i.to_string()), val));
                    }
                }
                _ => {}
            }
            Some(current_path)
        } else {
            None
        }
    }
}

pub fn get_all_paths<'a, T: IntoIterator<Item = &'a serde_yaml::Value>>(
    iter: T,
) -> impl Iterator<Item = String> + 'a {
    let mut stack = Vec::new();
    for value in iter {
        stack.push((".".to_string(), value));
    }
    PathIterator { stack }
}

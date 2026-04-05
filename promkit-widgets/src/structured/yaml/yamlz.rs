use rayon::prelude::*;

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
struct TagAwareContainer;

impl TagAwareContainer {
    fn get(node: &YamlNode) -> Option<&ContainerNode> {
        match node {
            YamlNode::Container(container) => Some(container),
            YamlNode::Tagged { node, .. } => Self::get(node),
            _ => None,
        }
    }

    fn replace(node: &YamlNode, new_container: ContainerNode) -> Option<YamlNode> {
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

impl RowOperation for Vec<Row> {
    type Row = Row;

    fn up(&self, current: usize) -> usize {
        if self.is_empty() || current == 0 {
            return 0;
        }

        let mut prev = current - 1;
        while prev > 0
            && matches!(
                TagAwareContainer::get(&self[prev].node),
                Some(ContainerNode::Close { .. })
            )
        {
            prev -= 1;
        }

        if matches!(
            TagAwareContainer::get(&self[prev].node),
            Some(ContainerNode::Close { .. })
        ) {
            current
        } else {
            prev
        }
    }

    fn head(&self) -> usize {
        self.iter()
            .position(|row| {
                !matches!(
                    TagAwareContainer::get(&row.node),
                    Some(ContainerNode::Close { .. })
                )
            })
            .unwrap_or(0)
    }

    fn down(&self, current: usize) -> usize {
        if self.is_empty() || current >= self.len().saturating_sub(1) {
            return current;
        }

        let mut next = match TagAwareContainer::get(&self[current].node) {
            Some(ContainerNode::Open {
                collapsed: true,
                close_index,
                ..
            }) => close_index + 1,
            _ => current + 1,
        };

        while next < self.len()
            && matches!(
                TagAwareContainer::get(&self[next].node),
                Some(ContainerNode::Close { .. })
            )
        {
            next += 1;
        }

        if next >= self.len() { current } else { next }
    }

    fn tail(&self) -> usize {
        if self.is_empty() {
            return 0;
        }

        self.iter()
            .rposition(|row| {
                !matches!(
                    TagAwareContainer::get(&row.node),
                    Some(ContainerNode::Close { .. })
                )
            })
            .unwrap_or(0)
    }

    fn toggle(&mut self, current: usize) -> usize {
        let container = TagAwareContainer::get(&self[current].node).cloned();
        match container {
            Some(ContainerNode::Open {
                typ,
                collapsed,
                close_index,
            }) => {
                let new_collapsed = !collapsed;

                self[current].node = TagAwareContainer::replace(
                    &self[current].node,
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
                        open_index: current,
                    },
                )
                .expect("container close node must be present");

                current
            }
            Some(ContainerNode::Close {
                typ,
                collapsed,
                open_index,
            }) => {
                let new_collapsed = !collapsed;

                self[current].node = TagAwareContainer::replace(
                    &self[current].node,
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
                        close_index: current,
                    },
                )
                .expect("container open node must be present");

                open_index
            }
            _ => current,
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

        while i < self.len()
            && matches!(
                TagAwareContainer::get(&self[i].node),
                Some(ContainerNode::Close { .. })
            )
        {
            i += 1;
        }

        while i < self.len() && result.len() < n {
            let row = &self[i];
            if matches!(
                TagAwareContainer::get(&row.node),
                Some(ContainerNode::Close { .. })
            ) {
                i += 1;
                continue;
            }

            result.push(row.clone());
            match TagAwareContainer::get(&row.node) {
                Some(ContainerNode::Open {
                    collapsed: true,
                    close_index,
                    ..
                }) => i = close_index + 1,
                _ => i += 1,
            }
        }

        result
    }
}

fn normalize_mapping_key_for_display(mapping_key: &serde_yaml::Value) -> Option<String> {
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

fn process_value(
    value: &serde_yaml::Value,
    rows: &mut Vec<Row>,
    depth: usize,
    key: Option<String>,
    is_sequence_item: bool,
) -> usize {
    match value {
        serde_yaml::Value::Tagged(tagged) => {
            let index = process_value(&tagged.value, rows, depth, key, is_sequence_item);
            rows[index].node = YamlNode::Tagged {
                tag: tagged.tag.to_string(),
                node: Box::new(rows[index].node.clone()),
            };
            index
        }
        serde_yaml::Value::Null => {
            rows.push(Row {
                depth,
                key,
                is_sequence_item,
                node: YamlNode::Null,
            });
            rows.len() - 1
        }
        serde_yaml::Value::Bool(b) => {
            rows.push(Row {
                depth,
                key,
                is_sequence_item,
                node: YamlNode::Boolean(*b),
            });
            rows.len() - 1
        }
        serde_yaml::Value::Number(n) => {
            rows.push(Row {
                depth,
                key,
                is_sequence_item,
                node: YamlNode::Number(n.clone()),
            });
            rows.len() - 1
        }
        serde_yaml::Value::String(s) => {
            rows.push(Row {
                depth,
                key,
                is_sequence_item,
                node: YamlNode::String(s.clone()),
            });
            rows.len() - 1
        }
        serde_yaml::Value::Sequence(seq) => {
            if seq.is_empty() {
                rows.push(Row {
                    depth,
                    key,
                    is_sequence_item,
                    node: YamlNode::Container(ContainerNode::Empty {
                        typ: ContainerType::Array,
                    }),
                });
                return rows.len() - 1;
            }

            let open_index = rows.len();
            rows.push(Row {
                depth,
                key,
                is_sequence_item,
                node: YamlNode::Container(ContainerNode::Open {
                    typ: ContainerType::Array,
                    collapsed: false,
                    close_index: 0,
                }),
            });

            for item in seq {
                process_value(item, rows, depth + 1, None, true);
            }

            let close_index = rows.len();
            rows.push(Row {
                depth,
                key: None,
                is_sequence_item: false,
                node: YamlNode::Container(ContainerNode::Close {
                    typ: ContainerType::Array,
                    collapsed: false,
                    open_index,
                }),
            });

            rows[open_index].node = YamlNode::Container(ContainerNode::Open {
                typ: ContainerType::Array,
                collapsed: false,
                close_index,
            });

            open_index
        }
        serde_yaml::Value::Mapping(map) => {
            if map.is_empty() {
                rows.push(Row {
                    depth,
                    key,
                    is_sequence_item,
                    node: YamlNode::Container(ContainerNode::Empty {
                        typ: ContainerType::Object,
                    }),
                });
                return rows.len() - 1;
            }

            let open_index = rows.len();
            rows.push(Row {
                depth,
                key,
                is_sequence_item,
                node: YamlNode::Container(ContainerNode::Open {
                    typ: ContainerType::Object,
                    collapsed: false,
                    close_index: 0,
                }),
            });

            for (mapping_key, map_value) in map {
                let key = normalize_mapping_key_for_display(mapping_key);
                process_value(map_value, rows, depth + 1, key, false);
            }

            let close_index = rows.len();
            rows.push(Row {
                depth,
                key: None,
                is_sequence_item: false,
                node: YamlNode::Container(ContainerNode::Close {
                    typ: ContainerType::Object,
                    collapsed: false,
                    open_index,
                }),
            });

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
    let mut rows = Vec::new();
    for (index, value) in iter.into_iter().enumerate() {
        if index > 0 {
            rows.push(Row {
                depth: 0,
                key: None,
                node: YamlNode::DocumentSeparator,
                is_sequence_item: false,
            });
        }
        process_value(value, &mut rows, 0, None, false);
    }
    rows
}

#[derive(Debug)]
pub struct PathIterator<'a> {
    stack: Vec<(String, &'a serde_yaml::Value)>,
}

impl PathIterator<'_> {
    fn escape_path_key(key: &str) -> String {
        if key.contains('.') || key.contains('-') || key.contains('@') {
            format!("\"{}\"", key)
        } else {
            key.to_string()
        }
    }
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
                                let escaped = Self::escape_path_key(key);
                                let new_path = if current_path == "." {
                                    format!(".{}", escaped)
                                } else {
                                    format!("{}.{}", current_path, escaped)
                                };
                                self.stack.push((new_path, val));
                            }
                            serde_yaml::Value::Number(n) => {
                                self.stack.push((format!("{}[{}]", current_path, n), val));
                            }
                            serde_yaml::Value::Bool(b) => {
                                self.stack.push((format!("{}[{}]", current_path, b), val));
                            }
                            serde_yaml::Value::Null => {
                                self.stack.push((format!("{}[null]", current_path), val));
                            }
                            _ => {}
                        }
                    }
                }
                serde_yaml::Value::Sequence(seq) => {
                    for (i, val) in seq.iter().enumerate() {
                        self.stack.push((format!("{}[{}]", current_path, i), val));
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

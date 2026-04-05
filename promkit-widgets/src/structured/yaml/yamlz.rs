use rayon::prelude::*;

pub use crate::structured::{ContainerNode, ContainerType, RowOperation};

#[derive(Clone, Debug, PartialEq)]
pub enum YamlNode {
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
    pub is_sequence_item: bool,
    pub tag: Option<String>,
    pub node: YamlNode,
}

impl Row {
    fn is_close_container(&self) -> bool {
        matches!(self.node, YamlNode::Container(ContainerNode::Close { .. }))
    }
}

impl RowOperation for Vec<Row> {
    type Row = Row;

    fn up(&self, current: usize) -> usize {
        if self.is_empty() || current == 0 {
            return 0;
        }

        let mut prev = current - 1;
        while prev > 0 && self[prev].is_close_container() {
            prev -= 1;
        }

        if self[prev].is_close_container() {
            current
        } else {
            prev
        }
    }

    fn head(&self) -> usize {
        self.iter()
            .position(|row| !row.is_close_container())
            .unwrap_or(0)
    }

    fn down(&self, current: usize) -> usize {
        if self.is_empty() || current >= self.len().saturating_sub(1) {
            return current;
        }

        let mut next = match &self[current].node {
            YamlNode::Container(ContainerNode::Open {
                collapsed: true,
                close_index,
                ..
            }) => close_index + 1,
            _ => current + 1,
        };

        while next < self.len() && self[next].is_close_container() {
            next += 1;
        }

        if next >= self.len() { current } else { next }
    }

    fn tail(&self) -> usize {
        if self.is_empty() {
            return 0;
        }

        self.iter()
            .rposition(|row| !row.is_close_container())
            .unwrap_or(0)
    }

    fn toggle(&mut self, current: usize) -> usize {
        match &self[current].node {
            YamlNode::Container(ContainerNode::Open {
                typ,
                collapsed,
                close_index,
            }) => {
                let new_collapsed = !collapsed;
                let close_idx = *close_index;
                let typ_clone = typ.clone();

                self[current].node = YamlNode::Container(ContainerNode::Open {
                    typ: typ_clone.clone(),
                    collapsed: new_collapsed,
                    close_index: close_idx,
                });

                self[close_idx].node = YamlNode::Container(ContainerNode::Close {
                    typ: typ_clone,
                    collapsed: new_collapsed,
                    open_index: current,
                });

                current
            }
            YamlNode::Container(ContainerNode::Close {
                typ,
                collapsed,
                open_index,
            }) => {
                let new_collapsed = !collapsed;
                let open_idx = *open_index;
                let typ_clone = typ.clone();

                self[current].node = YamlNode::Container(ContainerNode::Close {
                    typ: typ_clone.clone(),
                    collapsed: new_collapsed,
                    open_index: open_idx,
                });

                self[open_idx].node = YamlNode::Container(ContainerNode::Open {
                    typ: typ_clone,
                    collapsed: new_collapsed,
                    close_index: current,
                });

                open_idx
            }
            _ => current,
        }
    }

    fn set_rows_visibility(&mut self, collapsed: bool) {
        self.par_iter_mut().for_each(|row| {
            if let YamlNode::Container(ContainerNode::Open {
                typ, close_index, ..
            }) = &row.node
            {
                row.node = YamlNode::Container(ContainerNode::Open {
                    typ: typ.clone(),
                    collapsed,
                    close_index: *close_index,
                });
            } else if let YamlNode::Container(ContainerNode::Close {
                typ, open_index, ..
            }) = &row.node
            {
                row.node = YamlNode::Container(ContainerNode::Close {
                    typ: typ.clone(),
                    collapsed,
                    open_index: *open_index,
                });
            }
        });
    }

    fn extract(&self, current: usize, n: usize) -> Vec<Row> {
        let mut result = Vec::new();
        let mut i = current;

        while i < self.len() && self[i].is_close_container() {
            i += 1;
        }

        while i < self.len() && result.len() < n {
            let row = &self[i];
            if row.is_close_container() {
                i += 1;
                continue;
            }

            result.push(row.clone());
            match &row.node {
                YamlNode::Container(ContainerNode::Open {
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
    tag: Option<String>,
) -> usize {
    match value {
        serde_yaml::Value::Tagged(tagged) => process_value(
            &tagged.value,
            rows,
            depth,
            key,
            is_sequence_item,
            Some(tagged.tag.to_string()),
        ),
        serde_yaml::Value::Null => {
            rows.push(Row {
                depth,
                key,
                is_sequence_item,
                tag,
                node: YamlNode::Null,
            });
            rows.len() - 1
        }
        serde_yaml::Value::Bool(b) => {
            rows.push(Row {
                depth,
                key,
                is_sequence_item,
                tag,
                node: YamlNode::Boolean(*b),
            });
            rows.len() - 1
        }
        serde_yaml::Value::Number(n) => {
            rows.push(Row {
                depth,
                key,
                is_sequence_item,
                tag,
                node: YamlNode::Number(n.clone()),
            });
            rows.len() - 1
        }
        serde_yaml::Value::String(s) => {
            rows.push(Row {
                depth,
                key,
                is_sequence_item,
                tag,
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
                    tag,
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
                tag,
                node: YamlNode::Container(ContainerNode::Open {
                    typ: ContainerType::Array,
                    collapsed: false,
                    close_index: 0,
                }),
            });

            for item in seq {
                process_value(item, rows, depth + 1, None, true, None);
            }

            let close_index = rows.len();
            rows.push(Row {
                depth,
                key: None,
                is_sequence_item: false,
                tag: None,
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
                    tag,
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
                tag,
                node: YamlNode::Container(ContainerNode::Open {
                    typ: ContainerType::Object,
                    collapsed: false,
                    close_index: 0,
                }),
            });

            for (mapping_key, map_value) in map {
                let key = normalize_mapping_key_for_display(mapping_key);
                process_value(map_value, rows, depth + 1, key, false, None);
            }

            let close_index = rows.len();
            rows.push(Row {
                depth,
                key: None,
                is_sequence_item: false,
                tag: None,
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
    for value in iter {
        process_value(value, &mut rows, 0, None, false, None);
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

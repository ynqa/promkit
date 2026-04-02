#[derive(Clone, Debug, PartialEq)]
pub enum CollectionKind {
    Mapping,
    Sequence,
}

impl CollectionKind {
    pub fn empty_str(&self) -> &'static str {
        match self {
            CollectionKind::Mapping => "{}",
            CollectionKind::Sequence => "[]",
        }
    }

    pub fn collapsed_preview(&self) -> &'static str {
        match self {
            CollectionKind::Mapping => "{...}",
            CollectionKind::Sequence => "[...]",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum YamlNode {
    Null,
    Boolean(bool),
    Number(serde_yaml::Number),
    String(String),
    Empty {
        kind: CollectionKind,
    },
    Start {
        kind: CollectionKind,
        collapsed: bool,
        close_index: usize,
    },
    End {
        kind: CollectionKind,
        collapsed: bool,
        open_index: usize,
    },
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
    fn is_end(&self) -> bool {
        matches!(self.node, YamlNode::End { .. })
    }
}

pub trait RowOperation {
    fn up(&self, current: usize) -> usize;
    fn head(&self) -> usize;
    fn down(&self, current: usize) -> usize;
    fn tail(&self) -> usize;
    fn toggle(&mut self, current: usize) -> usize;
    fn set_nodes_visibility(&mut self, collapsed: bool);
    fn extract(&self, current: usize, n: usize) -> Vec<Row>;
}

impl RowOperation for Vec<Row> {
    fn up(&self, current: usize) -> usize {
        if self.is_empty() || current == 0 {
            return 0;
        }

        let mut prev = current - 1;
        while prev > 0 && self[prev].is_end() {
            prev -= 1;
        }
        if self[prev].is_end() {
            current
        } else {
            prev
        }
    }

    fn head(&self) -> usize {
        self.iter().position(|r| !r.is_end()).unwrap_or(0)
    }

    fn down(&self, current: usize) -> usize {
        if self.is_empty() || current >= self.len().saturating_sub(1) {
            return current;
        }

        let mut next = match &self[current].node {
            YamlNode::Start {
                collapsed: true,
                close_index,
                ..
            } => close_index + 1,
            _ => current + 1,
        };

        while next < self.len() && self[next].is_end() {
            next += 1;
        }

        if next >= self.len() { current } else { next }
    }

    fn tail(&self) -> usize {
        if self.is_empty() {
            return 0;
        }
        self.iter().rposition(|r| !r.is_end()).unwrap_or(0)
    }

    fn toggle(&mut self, current: usize) -> usize {
        match self[current].node.clone() {
            YamlNode::Start {
                kind,
                collapsed,
                close_index,
            } => {
                let new_collapsed = !collapsed;
                self[current].node = YamlNode::Start {
                    kind: kind.clone(),
                    collapsed: new_collapsed,
                    close_index,
                };
                self[close_index].node = YamlNode::End {
                    kind,
                    collapsed: new_collapsed,
                    open_index: current,
                };
                current
            }
            YamlNode::End {
                kind,
                collapsed,
                open_index,
            } => {
                let new_collapsed = !collapsed;
                self[current].node = YamlNode::End {
                    kind: kind.clone(),
                    collapsed: new_collapsed,
                    open_index,
                };
                self[open_index].node = YamlNode::Start {
                    kind,
                    collapsed: new_collapsed,
                    close_index: current,
                };
                open_index
            }
            _ => current,
        }
    }

    fn set_nodes_visibility(&mut self, collapsed: bool) {
        for row in self {
            match &mut row.node {
                YamlNode::Start { collapsed: c, .. } | YamlNode::End { collapsed: c, .. } => {
                    *c = collapsed;
                }
                _ => {}
            }
        }
    }

    fn extract(&self, current: usize, n: usize) -> Vec<Row> {
        let mut result = Vec::new();
        let mut i = current;
        while i < self.len() && self[i].is_end() {
            i += 1;
        }

        while i < self.len() && result.len() < n {
            let row = &self[i];
            if row.is_end() {
                i += 1;
                continue;
            }

            result.push(row.clone());
            match &row.node {
                YamlNode::Start {
                    collapsed: true,
                    close_index,
                    ..
                } => i = close_index + 1,
                _ => i += 1,
            }
        }

        result
    }
}

fn normalize_key_for_display(key: &serde_yaml::Value) -> Option<String> {
    match key {
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
                    node: YamlNode::Empty {
                        kind: CollectionKind::Sequence,
                    },
                });
                return rows.len() - 1;
            }

            let open_index = rows.len();
            rows.push(Row {
                depth,
                key,
                is_sequence_item,
                tag,
                node: YamlNode::Start {
                    kind: CollectionKind::Sequence,
                    collapsed: false,
                    close_index: 0,
                },
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
                node: YamlNode::End {
                    kind: CollectionKind::Sequence,
                    collapsed: false,
                    open_index,
                },
            });

            rows[open_index].node = YamlNode::Start {
                kind: CollectionKind::Sequence,
                collapsed: false,
                close_index,
            };

            open_index
        }
        serde_yaml::Value::Mapping(map) => {
            if map.is_empty() {
                rows.push(Row {
                    depth,
                    key,
                    is_sequence_item,
                    tag,
                    node: YamlNode::Empty {
                        kind: CollectionKind::Mapping,
                    },
                });
                return rows.len() - 1;
            }

            let open_index = rows.len();
            rows.push(Row {
                depth,
                key,
                is_sequence_item,
                tag,
                node: YamlNode::Start {
                    kind: CollectionKind::Mapping,
                    collapsed: false,
                    close_index: 0,
                },
            });

            for (map_key, map_value) in map {
                let key = normalize_key_for_display(map_key);
                process_value(map_value, rows, depth + 1, key, false, None);
            }

            let close_index = rows.len();
            rows.push(Row {
                depth,
                key: None,
                is_sequence_item: false,
                tag: None,
                node: YamlNode::End {
                    kind: CollectionKind::Mapping,
                    collapsed: false,
                    open_index,
                },
            });

            rows[open_index].node = YamlNode::Start {
                kind: CollectionKind::Mapping,
                collapsed: false,
                close_index,
            };

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

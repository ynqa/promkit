use rayon::prelude::*;

pub use crate::structured::{ContainerNode, ContainerType, PrettyRender, RowOperation};

#[derive(Clone, Debug, PartialEq)]
pub enum JsonNode {
    Null,
    Boolean(bool),
    Number(serde_json::Number),
    String(String),
    Container(ContainerNode),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Row {
    pub depth: usize,
    pub k: Option<String>,
    pub v: JsonNode,
}

impl PrettyRender for [Row] {
    fn render_pretty(&self, indent: usize) -> String {
        let mut result = String::new();
        let mut first_in_container = true;

        for (i, row) in self.iter().enumerate() {
            if !matches!(&row.v, JsonNode::Container(ContainerNode::Close { .. })) {
                if !result.is_empty() {
                    result.push('\n');
                }
                result.push_str(&" ".repeat(indent * row.depth));
            }

            if let Some(key) = &row.k {
                result.push('"');
                result.push_str(key);
                result.push_str("\": ");
            }

            match &row.v {
                JsonNode::Null => result.push_str("null"),
                JsonNode::Boolean(b) => result.push_str(&b.to_string()),
                JsonNode::Number(n) => result.push_str(&n.to_string()),
                JsonNode::String(s) => {
                    result.push('"');
                    result.push_str(&s.replace('\n', "\\n"));
                    result.push('"');
                }
                JsonNode::Container(node) => match node {
                    ContainerNode::Empty { typ } => {
                        result.push_str(typ.empty_str());
                    }
                    ContainerNode::Open { typ, .. } => {
                        result.push_str(typ.open_str());
                    }
                    ContainerNode::Close { typ, .. } => {
                        if !first_in_container {
                            result.push('\n');
                            result.push_str(&" ".repeat(indent * row.depth));
                        }
                        result.push_str(typ.close_str());
                    }
                },
            }

            if i + 1 < self.len() {
                if matches!(
                    &self[i + 1].v,
                    JsonNode::Container(ContainerNode::Close { .. })
                ) {
                } else if matches!(&row.v, JsonNode::Container(ContainerNode::Open { .. })) {
                } else {
                    result.push(',');
                }
            }

            if matches!(&row.v, JsonNode::Container(ContainerNode::Open { .. })) {
                first_in_container = true;
            } else {
                first_in_container = false;
            }
        }

        result
    }
}

impl RowOperation for Vec<Row> {
    type Row = Row;

    fn up(&self, current: usize) -> usize {
        if current == 0 {
            return 0;
        }

        let prev = current - 1;
        match &self[prev].v {
            JsonNode::Container(ContainerNode::Close {
                collapsed,
                open_index,
                ..
            }) if *collapsed => *open_index,
            _ => prev,
        }
    }

    fn head(&self) -> usize {
        0
    }

    fn down(&self, current: usize) -> usize {
        if current >= self.len() - 1 {
            return current;
        }

        let next = current + 1;
        match &self[current].v {
            JsonNode::Container(ContainerNode::Open {
                collapsed,
                close_index,
                ..
            }) if *collapsed => {
                let next_pos = close_index + 1;
                if next_pos >= self.len() {
                    current
                } else {
                    next_pos
                }
            }
            _ => next,
        }
    }

    fn tail(&self) -> usize {
        if self.is_empty() {
            return 0;
        }

        let mut last = self.len() - 1;
        match &self[last].v {
            JsonNode::Container(ContainerNode::Close {
                collapsed,
                open_index,
                ..
            }) if *collapsed => {
                last = *open_index;
                last
            }
            _ => last,
        }
    }

    fn toggle(&mut self, current: usize) -> usize {
        match &self[current].v {
            JsonNode::Container(ContainerNode::Open {
                typ,
                collapsed,
                close_index,
            }) => {
                let new_collapsed = !collapsed;
                let close_idx = *close_index;
                let typ_clone = typ.clone();

                self[current].v = JsonNode::Container(ContainerNode::Open {
                    typ: typ_clone.clone(),
                    collapsed: new_collapsed,
                    close_index: close_idx,
                });

                self[close_idx].v = JsonNode::Container(ContainerNode::Close {
                    typ: typ_clone,
                    collapsed: new_collapsed,
                    open_index: current,
                });

                current
            }
            JsonNode::Container(ContainerNode::Close {
                typ,
                collapsed,
                open_index,
            }) => {
                let new_collapsed = !collapsed;
                let open_idx = *open_index;
                let typ_clone = typ.clone();

                self[current].v = JsonNode::Container(ContainerNode::Close {
                    typ: typ_clone.clone(),
                    collapsed: new_collapsed,
                    open_index: open_idx,
                });

                self[open_idx].v = JsonNode::Container(ContainerNode::Open {
                    typ: typ_clone,
                    collapsed: new_collapsed,
                    close_index: current,
                });

                if new_collapsed { open_idx } else { current }
            }
            _ => current,
        }
    }

    fn set_rows_visibility(&mut self, collapsed: bool) {
        self.par_iter_mut().for_each(|row| {
            if let JsonNode::Container(ContainerNode::Open {
                typ, close_index, ..
            }) = &row.v
            {
                row.v = JsonNode::Container(ContainerNode::Open {
                    typ: typ.clone(),
                    collapsed,
                    close_index: *close_index,
                });
            } else if let JsonNode::Container(ContainerNode::Close {
                typ, open_index, ..
            }) = &row.v
            {
                row.v = JsonNode::Container(ContainerNode::Close {
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
        let mut remaining = n;

        while i < self.len() && remaining > 0 {
            result.push(self[i].clone());
            remaining -= 1;

            match &self[i].v {
                JsonNode::Container(ContainerNode::Open {
                    collapsed: true,
                    close_index,
                    ..
                }) => {
                    i = *close_index + 1;
                }
                _ => {
                    i += 1;
                }
            }
        }

        result
    }
}

fn process_value(
    value: &serde_json::Value,
    rows: &mut Vec<Row>,
    depth: usize,
    key: Option<String>,
) -> usize {
    match value {
        serde_json::Value::Null => {
            rows.push(Row {
                depth,
                k: key,
                v: JsonNode::Null,
            });
            rows.len() - 1
        }
        serde_json::Value::Bool(b) => {
            rows.push(Row {
                depth,
                k: key,
                v: JsonNode::Boolean(*b),
            });
            rows.len() - 1
        }
        serde_json::Value::Number(n) => {
            rows.push(Row {
                depth,
                k: key,
                v: JsonNode::Number(n.clone()),
            });
            rows.len() - 1
        }
        serde_json::Value::String(s) => {
            rows.push(Row {
                depth,
                k: key,
                v: JsonNode::String(s.clone()),
            });
            rows.len() - 1
        }
        serde_json::Value::Array(arr) => {
            if arr.is_empty() {
                rows.push(Row {
                    depth,
                    k: key,
                    v: JsonNode::Container(ContainerNode::Empty {
                        typ: ContainerType::Array,
                    }),
                });
                return rows.len() - 1;
            }

            let open_index = rows.len();

            rows.push(Row {
                depth,
                k: key,
                v: JsonNode::Container(ContainerNode::Open {
                    typ: ContainerType::Array,
                    collapsed: false,
                    close_index: 0,
                }),
            });

            for value in arr {
                process_value(value, rows, depth + 1, None);
            }

            let close_index = rows.len();
            rows.push(Row {
                depth,
                k: None,
                v: JsonNode::Container(ContainerNode::Close {
                    typ: ContainerType::Array,
                    collapsed: false,
                    open_index,
                }),
            });

            rows[open_index].v = JsonNode::Container(ContainerNode::Open {
                typ: ContainerType::Array,
                collapsed: false,
                close_index,
            });

            open_index
        }
        serde_json::Value::Object(obj) => {
            if obj.is_empty() {
                rows.push(Row {
                    depth,
                    k: key,
                    v: JsonNode::Container(ContainerNode::Empty {
                        typ: ContainerType::Object,
                    }),
                });
                return rows.len() - 1;
            }

            let open_index = rows.len();

            rows.push(Row {
                depth,
                k: key,
                v: JsonNode::Container(ContainerNode::Open {
                    typ: ContainerType::Object,
                    collapsed: false,
                    close_index: 0,
                }),
            });

            for (key, value) in obj {
                process_value(value, rows, depth + 1, Some(key.clone()));
            }

            let close_index = rows.len();
            rows.push(Row {
                depth,
                k: None,
                v: JsonNode::Container(ContainerNode::Close {
                    typ: ContainerType::Object,
                    collapsed: false,
                    open_index,
                }),
            });

            rows[open_index].v = JsonNode::Container(ContainerNode::Open {
                typ: ContainerType::Object,
                collapsed: false,
                close_index,
            });

            open_index
        }
    }
}

pub fn create_rows<'a, T: IntoIterator<Item = &'a serde_json::Value>>(iter: T) -> Vec<Row> {
    let mut rows = Vec::new();
    for value in iter {
        process_value(value, &mut rows, 0, None);
    }
    rows
}

#[derive(Debug)]
pub struct PathIterator<'a> {
    stack: Vec<(String, &'a serde_json::Value)>,
}

impl PathIterator<'_> {
    fn escape_json_path_key(key: &str) -> String {
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
                serde_json::Value::Object(obj) => {
                    for (key, val) in obj.iter() {
                        let escaped = Self::escape_json_path_key(key);
                        let new_path = if current_path == "." {
                            format!(".{}", escaped)
                        } else {
                            format!("{}.{}", current_path, escaped)
                        };
                        self.stack.push((new_path, val));
                    }
                }
                serde_json::Value::Array(arr) => {
                    for (i, val) in arr.iter().enumerate() {
                        let new_path = format!("{}[{}]", current_path, i);
                        self.stack.push((new_path, val));
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

pub fn get_all_paths<'a, T: IntoIterator<Item = &'a serde_json::Value>>(
    iter: T,
) -> impl Iterator<Item = String> + 'a {
    let mut stack = Vec::new();
    for value in iter {
        stack.push((".".to_string(), value));
    }
    PathIterator { stack }
}

use rayon::prelude::*;

pub mod format;

#[derive(Clone, Debug, PartialEq)]
pub enum ContainerType {
    Object,
    Array,
}

impl ContainerType {
    pub fn open_str(&self) -> &'static str {
        match self {
            ContainerType::Object => "{",
            ContainerType::Array => "[",
        }
    }

    pub fn close_str(&self) -> &'static str {
        match self {
            ContainerType::Object => "}",
            ContainerType::Array => "]",
        }
    }

    pub fn empty_str(&self) -> &'static str {
        match self {
            ContainerType::Object => "{}",
            ContainerType::Array => "[]",
        }
    }

    pub fn collapsed_preview(&self) -> &'static str {
        match self {
            ContainerType::Object => "{…}",
            ContainerType::Array => "[…]",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Null,
    Boolean(bool),
    Number(serde_json::Number),
    String(String),
    Empty {
        typ: ContainerType,
    },
    Open {
        typ: ContainerType,
        collapsed: bool,
        close_index: usize,
    },
    Close {
        typ: ContainerType,
        collapsed: bool,
        open_index: usize,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct Row {
    pub depth: usize,
    pub k: Option<String>,
    pub v: Value,
}

pub trait RowOperation {
    fn up(&self, current: usize) -> usize;
    fn head(&self) -> usize;
    fn down(&self, current: usize) -> usize;
    fn tail(&self) -> usize;
    fn toggle(&mut self, current: usize) -> usize;
    fn set_rows_visibility(&mut self, collapsed: bool);
    fn extract(&self, current: usize, n: usize) -> Vec<Row>;
}

impl RowOperation for Vec<Row> {
    fn up(&self, current: usize) -> usize {
        if current == 0 {
            return 0;
        }

        let prev = current - 1;
        match &self[prev].v {
            Value::Close {
                collapsed,
                open_index,
                ..
            } if *collapsed => *open_index,
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
            Value::Open {
                collapsed,
                close_index,
                ..
            } if *collapsed => {
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
            Value::Close {
                collapsed,
                open_index,
                ..
            } if *collapsed => {
                last = *open_index;
                last
            }
            _ => last,
        }
    }

    fn toggle(&mut self, current: usize) -> usize {
        match &self[current].v {
            Value::Open {
                typ,
                collapsed,
                close_index,
            } => {
                let new_collapsed = !collapsed;
                let close_idx = *close_index;
                let typ_clone = typ.clone();

                self[current].v = Value::Open {
                    typ: typ_clone.clone(),
                    collapsed: new_collapsed,
                    close_index: close_idx,
                };

                self[close_idx].v = Value::Close {
                    typ: typ_clone,
                    collapsed: new_collapsed,
                    open_index: current,
                };

                current
            }
            Value::Close {
                typ,
                collapsed,
                open_index,
            } => {
                let new_collapsed = !collapsed;
                let open_idx = *open_index;
                let typ_clone = typ.clone();

                self[current].v = Value::Close {
                    typ: typ_clone.clone(),
                    collapsed: new_collapsed,
                    open_index: open_idx,
                };

                self[open_idx].v = Value::Open {
                    typ: typ_clone,
                    collapsed: new_collapsed,
                    close_index: current,
                };

                if new_collapsed {
                    open_idx
                } else {
                    current
                }
            }
            _ => current,
        }
    }

    fn set_rows_visibility(&mut self, collapsed: bool) {
        self.par_iter_mut().for_each(|row| {
            if let Value::Open {
                typ, close_index, ..
            } = &row.v
            {
                row.v = Value::Open {
                    typ: typ.clone(),
                    collapsed,
                    close_index: *close_index,
                };
            } else if let Value::Close {
                typ, open_index, ..
            } = &row.v
            {
                row.v = Value::Close {
                    typ: typ.clone(),
                    collapsed,
                    open_index: *open_index,
                };
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
                Value::Open {
                    collapsed: true,
                    close_index,
                    ..
                } => {
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
                v: Value::Null,
            });
            rows.len() - 1
        }
        serde_json::Value::Bool(b) => {
            rows.push(Row {
                depth,
                k: key,
                v: Value::Boolean(*b),
            });
            rows.len() - 1
        }
        serde_json::Value::Number(n) => {
            rows.push(Row {
                depth,
                k: key,
                v: Value::Number(n.clone()),
            });
            rows.len() - 1
        }
        serde_json::Value::String(s) => {
            rows.push(Row {
                depth,
                k: key,
                v: Value::String(s.clone()),
            });
            rows.len() - 1
        }
        serde_json::Value::Array(arr) => {
            if arr.is_empty() {
                rows.push(Row {
                    depth,
                    k: key,
                    v: Value::Empty {
                        typ: ContainerType::Array,
                    },
                });
                return rows.len() - 1;
            }

            let open_index = rows.len();

            rows.push(Row {
                depth,
                k: key,
                v: Value::Open {
                    typ: ContainerType::Array,
                    collapsed: false,
                    close_index: 0,
                },
            });

            for value in arr {
                process_value(value, rows, depth + 1, None);
            }

            let close_index = rows.len();
            rows.push(Row {
                depth,
                k: None,
                v: Value::Close {
                    typ: ContainerType::Array,
                    collapsed: false,
                    open_index,
                },
            });

            rows[open_index].v = Value::Open {
                typ: ContainerType::Array,
                collapsed: false,
                close_index,
            };

            open_index
        }
        serde_json::Value::Object(obj) => {
            if obj.is_empty() {
                rows.push(Row {
                    depth,
                    k: key,
                    v: Value::Empty {
                        typ: ContainerType::Object,
                    },
                });
                return rows.len() - 1;
            }

            let open_index = rows.len();

            rows.push(Row {
                depth,
                k: key,
                v: Value::Open {
                    typ: ContainerType::Object,
                    collapsed: false,
                    close_index: 0,
                },
            });

            for (key, value) in obj {
                process_value(value, rows, depth + 1, Some(key.clone()));
            }

            let close_index = rows.len();
            rows.push(Row {
                depth,
                k: None,
                v: Value::Close {
                    typ: ContainerType::Object,
                    collapsed: false,
                    open_index,
                },
            });

            rows[open_index].v = Value::Open {
                typ: ContainerType::Object,
                collapsed: false,
                close_index,
            };

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

impl<'a> Iterator for PathIterator<'a> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some((current_path, value)) = self.stack.pop() {
            match value {
                serde_json::Value::Object(obj) => {
                    for (key, val) in obj.iter() {
                        let new_path = if current_path == "." {
                            format!(".{}", key)
                        } else {
                            format!("{}.{}", current_path, key)
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

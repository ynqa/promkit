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
    fn up(&mut self, current: usize) -> usize;
}

impl RowOperation for Vec<Row> {
    fn up(&mut self, current: usize) -> usize {
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

pub fn create_rows<T: IntoIterator<Item = serde_json::Value>>(iter: T) -> Vec<Row> {
    let mut rows = Vec::new();
    for value in iter {
        process_value(&value, &mut rows, 0, None);
    }
    rows
}

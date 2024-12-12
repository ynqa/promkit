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

pub fn create_rows<T: IntoIterator<Item = serde_json::Value>>(iter: T) -> Vec<Row> {
    let mut rows = vec![];
    rows
}

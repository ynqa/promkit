use winnow::Parser;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Boolean,
    Number,
    String,
    EmptyArray,
    EmptyObject,
    OpenContainer {
        container_type: ContainerType,
        collapsed: bool,
        first_child: usize,
        close_index: usize,
    },
    CloseContainer {
        container_type: ContainerType,
        collapsed: bool,
        last_child: usize,
        open_index: usize,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ContainerType {
    Array,
    Object,
}

#[derive(Clone, Debug)]
pub struct Row {
    pub parent: Option<usize>,
    pub depth: usize,
    pub value: Value,
    pub prev_sibling: Option<usize>,
    pub next_sibling: Option<usize>,
}

pub struct JsonParser<'a> {
    input: &'a str,
    parents: Vec<usize>,
    rows: Vec<Row>,
}

impl<'a> JsonParser<'a> {}

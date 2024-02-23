use indexmap::IndexMap;
use serde_json::{self, Result, Value};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Kind {
    /// `Value` is one of the following:
    /// Null, Bool(bool), Number(Number), String(String)
    /// All `bool` represents whether there is comma or not.

    /// e.g. { or "map": {
    MapStart { key: Option<String>, path: JsonPath },
    /// e.g. }
    MapEnd { is_last: bool },
    /// e.g. "map": { ... }
    MapFolded {
        key: Option<String>,
        path: JsonPath,
        is_last: bool,
    },
    /// e.g. "number": 1
    MapEntry {
        kv: (String, Value),
        path: JsonPath,
        is_last: bool,
    },

    /// e.g. [ or "list": [
    ArrayStart { key: Option<String>, path: JsonPath },
    /// e.g. ]
    ArrayEnd { is_last: bool },
    /// e.g. "list": [ ... ]
    ArrayFolded {
        key: Option<String>,
        path: JsonPath,
        is_last: bool,
    },
    /// e.g. "abc"
    ArrayEntry {
        v: Value,
        path: JsonPath,
        is_last: bool,
    },
}

pub type JsonPath = Vec<JsonPathSegment>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum JsonPathSegment {
    Key(String),
    Index(usize),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Node {
    Object {
        children: IndexMap<String, Node>,
        children_visible: bool,
    },
    Array {
        children: Vec<Node>,
        children_visible: bool,
    },
    /// Null, Bool(bool), Number(Number), String(String)
    Leaf(Value),
}

impl Node {
    pub fn new(value: Value) -> Self {
        match value {
            Value::Object(map) => {
                let children = map.into_iter().map(|(k, v)| (k, Node::new(v))).collect();
                Node::Object {
                    children,
                    children_visible: true,
                }
            }
            Value::Array(vec) => {
                let children = vec.into_iter().map(Node::new).collect();
                Node::Array {
                    children,
                    children_visible: true,
                }
            }
            _ => Node::Leaf(value),
        }
    }

    pub fn new_from_str(json_str: &str) -> Result<Self> {
        let value: Value = serde_json::from_str(json_str)?;
        Ok(Node::new(value))
    }

    pub fn get(&self, path: &JsonPath) -> Option<&Node> {
        let mut node = self;
        for seg in path {
            match seg {
                JsonPathSegment::Key(s) => {
                    if let Node::Object { children, .. } = node {
                        node = children.get(s).unwrap();
                    }
                }
                JsonPathSegment::Index(n) => {
                    if let Node::Array { children, .. } = node {
                        node = children.get(*n).unwrap();
                    }
                }
            }
        }
        Some(node)
    }

    pub fn get_mut(&mut self, path: &JsonPath) -> Option<&mut Node> {
        let mut node = self;
        for seg in path {
            match seg {
                JsonPathSegment::Key(s) => {
                    if let Node::Object { children, .. } = node {
                        node = children.get_mut(s).unwrap();
                    }
                }
                JsonPathSegment::Index(n) => {
                    if let Node::Array { children, .. } = node {
                        node = children.get_mut(*n).unwrap();
                    }
                }
            }
        }
        Some(node)
    }

    pub fn toggle(&mut self, path: &JsonPath) {
        if let Some(node) = self.get_mut(path) {
            match node {
                Node::Object {
                    children_visible, ..
                } => *children_visible = !*children_visible,
                Node::Array {
                    children_visible, ..
                } => *children_visible = !*children_visible,
                _ => {}
            }
        }
    }

    pub fn flatten_visibles(&self) -> Vec<Kind> {
        fn dfs(
            node: &Node,
            path: JsonPath,
            ret: &mut Vec<Kind>,
            parent_visible: bool,
            is_last: bool,
        ) {
            match node {
                Node::Object {
                    children,
                    children_visible,
                } => {
                    if *children_visible && parent_visible {
                        let start_kind = Kind::MapStart {
                            key: path.last().and_then(|index| match index {
                                JsonPathSegment::Key(s) => Some(s.clone()),
                                _ => None,
                            }),
                            path: path.clone(),
                        };
                        ret.push(start_kind);

                        let keys = children.keys().collect::<Vec<_>>();
                        for (i, key) in keys.iter().enumerate() {
                            let child = children.get(*key).unwrap();
                            let mut branch = path.clone();
                            branch.push(JsonPathSegment::Key(key.to_string()));
                            let child_is_last = i == keys.len() - 1;
                            dfs(child, branch, ret, *children_visible, child_is_last);
                        }

                        ret.push(Kind::MapEnd { is_last });
                    } else {
                        ret.push(Kind::MapFolded {
                            key: path.last().and_then(|index| match index {
                                JsonPathSegment::Key(s) => Some(s.clone()),
                                _ => None,
                            }),
                            path: path.clone(),
                            is_last,
                        });
                    }
                }
                Node::Array {
                    children,
                    children_visible,
                } => {
                    if *children_visible && parent_visible {
                        let start_kind = Kind::ArrayStart {
                            key: path.last().and_then(|index| match index {
                                JsonPathSegment::Key(s) => Some(s.clone()),
                                _ => None,
                            }),
                            path: path.clone(),
                        };
                        ret.push(start_kind);

                        for (i, child) in children.iter().enumerate() {
                            let mut branch = path.clone();
                            branch.push(JsonPathSegment::Index(i));
                            let child_is_last = i == children.len() - 1;
                            dfs(child, branch, ret, true, child_is_last);
                        }

                        ret.push(Kind::ArrayEnd { is_last });
                    } else {
                        ret.push(Kind::ArrayFolded {
                            key: path.last().and_then(|index| match index {
                                JsonPathSegment::Key(s) => Some(s.clone()),
                                _ => None,
                            }),
                            path: path.clone(),
                            is_last,
                        });
                    }
                }
                Node::Leaf(value) => {
                    if let Some(JsonPathSegment::Key(key)) = path.last() {
                        ret.push(Kind::MapEntry {
                            kv: (key.clone(), value.clone()),
                            path: path.clone(),
                            is_last,
                        });
                    } else {
                        // This case might not be necessary
                        // if leaves are always under a key,
                        // but it's here for completeness.
                        ret.push(Kind::ArrayEntry {
                            v: value.clone(),
                            path: path.clone(),
                            is_last,
                        });
                    }
                }
            }
        }

        let mut ret = Vec::new();
        dfs(self, Vec::new(), &mut ret, true, true); // Start with the root node being visible and is_last true
        ret
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const JSON_STR: &str = r#"
    {
        "number": 1,
        "map": {
          "string1": "aaa",
          "string2": "bbb"
        },
        "list": [
          "abc",
          "def"
        ],
        "map_in_map": {
          "nested": {
            "leaf": "eof"
          }
        },
        "map_in_list": [
          {
            "map1": 1
          },
          {
            "map2": 2
          }
        ]
    }"#;

    fn as_object(node: &Node) -> Option<(&IndexMap<String, Node>, bool)> {
        if let Node::Object {
            children,
            children_visible,
        } = node
        {
            Some((children, *children_visible))
        } else {
            None
        }
    }

    mod flatten_visibles {

        use super::super::*;
        use super::JSON_STR;

        #[test]
        fn test_after_toggle() {
            let mut node = Node::new_from_str(JSON_STR).unwrap();
            node.toggle(&vec![]);
            assert_eq!(
                vec![Kind::MapFolded {
                    key: None,
                    path: vec![],
                    is_last: true,
                }],
                node.flatten_visibles(),
            );
        }

        #[test]
        fn test_string() {
            let mut node = Node::new_from_str("\"makoto\"").unwrap();
            node.toggle(&vec![]);
            assert_eq!(
                vec![Kind::ArrayEntry {
                    v: Value::String("makoto".to_string()),
                    path: vec![],
                    is_last: true
                },],
                node.flatten_visibles(),
            );
        }

        #[test]
        fn test() {
            let node = Node::new_from_str(JSON_STR).unwrap();
            assert_eq!(
                vec![
                    // {
                    Kind::MapStart {
                        key: None,
                        path: vec![]
                    },
                    // "number": 1,
                    Kind::MapEntry {
                        kv: (
                            "number".to_string(),
                            Value::Number(serde_json::Number::from(1))
                        ),
                        path: vec![JsonPathSegment::Key("number".to_string())],
                        is_last: false,
                    },
                    // "map": {
                    Kind::MapStart {
                        key: Some("map".to_string()),
                        path: vec![JsonPathSegment::Key("map".to_string())],
                    },
                    // "string1": "aaa",
                    Kind::MapEntry {
                        kv: ("string1".to_string(), Value::String("aaa".to_string())),
                        path: vec![
                            JsonPathSegment::Key("map".to_string()),
                            JsonPathSegment::Key("string1".to_string())
                        ],
                        is_last: false,
                    },
                    // "string2": "bbb"
                    Kind::MapEntry {
                        kv: ("string2".to_string(), Value::String("bbb".to_string())),
                        path: vec![
                            JsonPathSegment::Key("map".to_string()),
                            JsonPathSegment::Key("string2".to_string())
                        ],
                        is_last: true,
                    },
                    // },
                    Kind::MapEnd { is_last: false },
                    // "list": [
                    Kind::ArrayStart {
                        key: Some("list".to_string()),
                        path: vec![JsonPathSegment::Key("list".to_string())],
                    },
                    // "abc",
                    Kind::ArrayEntry {
                        v: Value::String("abc".to_string()),
                        path: vec![
                            JsonPathSegment::Key("list".to_string()),
                            JsonPathSegment::Index(0)
                        ],
                        is_last: false,
                    },
                    // "def"
                    Kind::ArrayEntry {
                        v: Value::String("def".to_string()),
                        path: vec![
                            JsonPathSegment::Key("list".to_string()),
                            JsonPathSegment::Index(1)
                        ],
                        is_last: true,
                    },
                    // ],
                    Kind::ArrayEnd { is_last: false },
                    // "map_in_map": {
                    Kind::MapStart {
                        key: Some("map_in_map".to_string()),
                        path: vec![JsonPathSegment::Key("map_in_map".to_string())],
                    },
                    // "nested": {
                    Kind::MapStart {
                        key: Some("nested".to_string()),
                        path: vec![
                            JsonPathSegment::Key("map_in_map".to_string()),
                            JsonPathSegment::Key("nested".to_string())
                        ],
                    },
                    // "leaf": "eof"
                    Kind::MapEntry {
                        kv: ("leaf".to_string(), Value::String("eof".to_string())),
                        path: vec![
                            JsonPathSegment::Key("map_in_map".to_string()),
                            JsonPathSegment::Key("nested".to_string()),
                            JsonPathSegment::Key("leaf".to_string())
                        ],
                        is_last: true,
                    },
                    // }
                    Kind::MapEnd { is_last: true },
                    // },
                    Kind::MapEnd { is_last: false },
                    // "map_in_list": [
                    Kind::ArrayStart {
                        key: Some("map_in_list".to_string()),
                        path: vec![JsonPathSegment::Key("map_in_list".to_string())],
                    },
                    // {
                    Kind::MapStart {
                        key: None,
                        path: vec![
                            JsonPathSegment::Key("map_in_list".to_string()),
                            JsonPathSegment::Index(0)
                        ],
                    },
                    // "map1": 1
                    Kind::MapEntry {
                        kv: (
                            "map1".to_string(),
                            Value::Number(serde_json::Number::from(1))
                        ),
                        path: vec![
                            JsonPathSegment::Key("map_in_list".to_string()),
                            JsonPathSegment::Index(0),
                            JsonPathSegment::Key("map1".to_string())
                        ],
                        is_last: true,
                    },
                    // },
                    Kind::MapEnd { is_last: false },
                    // {
                    Kind::MapStart {
                        key: None,
                        path: vec![
                            JsonPathSegment::Key("map_in_list".to_string()),
                            JsonPathSegment::Index(1)
                        ],
                    },
                    // "map2": 2
                    Kind::MapEntry {
                        kv: (
                            "map2".to_string(),
                            Value::Number(serde_json::Number::from(2))
                        ),
                        path: vec![
                            JsonPathSegment::Key("map_in_list".to_string()),
                            JsonPathSegment::Index(1),
                            JsonPathSegment::Key("map2".to_string())
                        ],
                        is_last: true,
                    },
                    // }
                    Kind::MapEnd { is_last: true },
                    // ]
                    Kind::ArrayEnd { is_last: true },
                    // }
                    Kind::MapEnd { is_last: true },
                ],
                node.flatten_visibles(),
            );
        }
    }

    mod toggle {
        use super::super::*;
        use super::{as_object, JSON_STR};

        #[test]
        fn test() {
            let mut node = Node::new_from_str(JSON_STR).unwrap();
            node.toggle(&vec![JsonPathSegment::Key("map".to_string())]);
            assert!(
                !as_object(
                    node.get(&vec![JsonPathSegment::Key("map".to_string())])
                        .unwrap()
                )
                .unwrap()
                .1
            );
        }
    }

    mod get {
        use super::super::*;
        use super::JSON_STR;

        #[test]
        fn test() {
            let node = Node::new_from_str(JSON_STR).unwrap();
            assert_eq!(&node, node.get(&vec![]).unwrap());
        }
    }

    mod from_str {
        use serde_json::Number;

        use super::super::*;
        use super::JSON_STR;

        #[test]
        fn test() {
            assert_eq!(
                Node::Object {
                    children: IndexMap::from_iter(vec![
                        (
                            String::from("number"),
                            Node::Leaf(Value::Number(Number::from(1)))
                        ),
                        (
                            String::from("map"),
                            Node::Object {
                                children: IndexMap::from_iter(vec![
                                    (
                                        String::from("string1"),
                                        Node::Leaf(Value::String(String::from("aaa")))
                                    ),
                                    (
                                        String::from("string2"),
                                        Node::Leaf(Value::String(String::from("bbb")))
                                    ),
                                ]),
                                children_visible: true,
                            }
                        ),
                        (
                            String::from("list"),
                            Node::Array {
                                children: vec![
                                    Node::Leaf(Value::String(String::from("abc"))),
                                    Node::Leaf(Value::String(String::from("def"))),
                                ],
                                children_visible: true,
                            }
                        ),
                        (
                            String::from("map_in_map"),
                            Node::Object {
                                children: IndexMap::from_iter(vec![(
                                    String::from("nested"),
                                    Node::Object {
                                        children: IndexMap::from_iter(vec![(
                                            String::from("leaf"),
                                            Node::Leaf(Value::String(String::from("eof")))
                                        ),]),
                                        children_visible: true,
                                    }
                                ),]),
                                children_visible: true,
                            }
                        ),
                        (
                            String::from("map_in_list"),
                            Node::Array {
                                children: vec![
                                    Node::Object {
                                        children: IndexMap::from_iter(vec![(
                                            String::from("map1"),
                                            Node::Leaf(Value::Number(Number::from(1)))
                                        ),]),
                                        children_visible: true,
                                    },
                                    Node::Object {
                                        children: IndexMap::from_iter(vec![(
                                            String::from("map2"),
                                            Node::Leaf(Value::Number(Number::from(2)))
                                        ),]),
                                        children_visible: true,
                                    },
                                ],
                                children_visible: true,
                            }
                        ),
                    ]),
                    children_visible: true,
                },
                Node::new_from_str(JSON_STR).unwrap(),
            );
        }
    }
}

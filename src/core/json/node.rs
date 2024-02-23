use indexmap::IndexMap;
use serde_json;

use crate::{Error, Result};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum JsonSyntaxKind {
    /// `Value` is one of the following:
    /// Null, Bool(bool), Number(Number), String(String)
    /// All `bool` represents whether there is comma or not.

    /// e.g. { or "map": {
    MapStart {
        key: Option<String>,
        path: JsonPath,
        indent: usize,
    },
    /// e.g. }
    MapEnd { is_last: bool, indent: usize },
    /// e.g. "map": { ... }
    MapFolded {
        key: Option<String>,
        path: JsonPath,
        is_last: bool,
        indent: usize,
    },
    /// e.g. "number": 1
    MapEntry {
        kv: (String, serde_json::Value),
        path: JsonPath,
        is_last: bool,
        indent: usize,
    },

    /// e.g. [ or "list": [
    ArrayStart {
        key: Option<String>,
        path: JsonPath,
        indent: usize,
    },
    /// e.g. ]
    ArrayEnd { is_last: bool, indent: usize },
    /// e.g. "list": [ ... ]
    ArrayFolded {
        key: Option<String>,
        path: JsonPath,
        is_last: bool,
        indent: usize,
    },
    /// e.g. "abc"
    ArrayEntry {
        v: serde_json::Value,
        path: JsonPath,
        is_last: bool,
        indent: usize,
    },
}

pub type JsonPath = Vec<JsonPathSegment>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum JsonPathSegment {
    Key(String),
    Index(usize),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum JsonNode {
    Object {
        children: IndexMap<String, JsonNode>,
        children_visible: bool,
    },
    Array {
        children: Vec<JsonNode>,
        children_visible: bool,
    },
    /// Null, Bool(bool), Number(Number), String(String)
    Leaf(serde_json::Value),
}

impl TryFrom<&str> for JsonNode {
    type Error = Error;

    fn try_from(json_str: &str) -> Result<Self> {
        let value: serde_json::Value = serde_json::from_str(json_str)?;
        Ok(JsonNode::from(value))
    }
}

impl From<serde_json::Value> for JsonNode {
    fn from(value: serde_json::Value) -> Self {
        match value {
            serde_json::Value::Object(map) => {
                let children = map
                    .into_iter()
                    .map(|(k, v)| (k, JsonNode::from(v)))
                    .collect();
                JsonNode::Object {
                    children,
                    children_visible: true,
                }
            }
            serde_json::Value::Array(vec) => {
                let children = vec.into_iter().map(JsonNode::from).collect();
                JsonNode::Array {
                    children,
                    children_visible: true,
                }
            }
            _ => JsonNode::Leaf(value),
        }
    }
}

impl JsonNode {
    pub fn get(&self, path: &JsonPath) -> Option<&JsonNode> {
        let mut node = self;
        for seg in path {
            node = match seg {
                JsonPathSegment::Key(s) => {
                    if let JsonNode::Object { children, .. } = node {
                        children.get(s)?
                    } else {
                        return None;
                    }
                }
                JsonPathSegment::Index(n) => {
                    if let JsonNode::Array { children, .. } = node {
                        children.get(*n)?
                    } else {
                        return None;
                    }
                }
            };
        }
        Some(node)
    }

    pub fn get_mut(&mut self, path: &JsonPath) -> Option<&mut JsonNode> {
        let mut node = self;
        for seg in path {
            node = match seg {
                JsonPathSegment::Key(s) => {
                    if let JsonNode::Object { children, .. } = node {
                        children.get_mut(s)?
                    } else {
                        return None;
                    }
                }
                JsonPathSegment::Index(n) => {
                    if let JsonNode::Array { children, .. } = node {
                        children.get_mut(*n)?
                    } else {
                        return None;
                    }
                }
            };
        }
        Some(node)
    }

    pub fn toggle(&mut self, path: &JsonPath) {
        if let Some(node) = self.get_mut(path) {
            match node {
                JsonNode::Object {
                    children_visible, ..
                } => *children_visible = !*children_visible,
                JsonNode::Array {
                    children_visible, ..
                } => *children_visible = !*children_visible,
                _ => {}
            }
        }
    }

    pub fn flatten_visibles(&self) -> Vec<JsonSyntaxKind> {
        fn dfs(
            node: &JsonNode,
            path: JsonPath,
            ret: &mut Vec<JsonSyntaxKind>,
            is_last: bool,
            indent: usize,
        ) {
            match node {
                JsonNode::Object {
                    children,
                    children_visible,
                } => {
                    if *children_visible {
                        let start_kind = JsonSyntaxKind::MapStart {
                            key: path.last().and_then(|index| match index {
                                JsonPathSegment::Key(s) => Some(s.clone()),
                                _ => None,
                            }),
                            path: path.clone(),
                            indent,
                        };
                        ret.push(start_kind);

                        let keys = children.keys().collect::<Vec<_>>();
                        for (i, key) in keys.iter().enumerate() {
                            let child = children.get(*key).unwrap();
                            let mut branch = path.clone();
                            branch.push(JsonPathSegment::Key(key.to_string()));
                            let child_is_last = i == keys.len() - 1;
                            dfs(child, branch, ret, child_is_last, indent + 1);
                        }

                        ret.push(JsonSyntaxKind::MapEnd { is_last, indent });
                    } else {
                        ret.push(JsonSyntaxKind::MapFolded {
                            key: path.last().and_then(|index| match index {
                                JsonPathSegment::Key(s) => Some(s.clone()),
                                _ => None,
                            }),
                            path: path.clone(),
                            is_last,
                            indent,
                        });
                    }
                }
                JsonNode::Array {
                    children,
                    children_visible,
                } => {
                    if *children_visible {
                        let start_kind = JsonSyntaxKind::ArrayStart {
                            key: path.last().and_then(|index| match index {
                                JsonPathSegment::Key(s) => Some(s.clone()),
                                _ => None,
                            }),
                            path: path.clone(),
                            indent,
                        };
                        ret.push(start_kind);

                        for (i, child) in children.iter().enumerate() {
                            let mut branch = path.clone();
                            branch.push(JsonPathSegment::Index(i));
                            let child_is_last = i == children.len() - 1;
                            dfs(child, branch, ret, child_is_last, indent + 1);
                        }

                        ret.push(JsonSyntaxKind::ArrayEnd { is_last, indent });
                    } else {
                        ret.push(JsonSyntaxKind::ArrayFolded {
                            key: path.last().and_then(|index| match index {
                                JsonPathSegment::Key(s) => Some(s.clone()),
                                _ => None,
                            }),
                            path: path.clone(),
                            is_last,
                            indent,
                        });
                    }
                }
                JsonNode::Leaf(value) => {
                    if let Some(JsonPathSegment::Key(key)) = path.last() {
                        ret.push(JsonSyntaxKind::MapEntry {
                            kv: (key.clone(), value.clone()),
                            path: path.clone(),
                            is_last,
                            indent,
                        });
                    } else {
                        ret.push(JsonSyntaxKind::ArrayEntry {
                            v: value.clone(),
                            path: path.clone(),
                            is_last,
                            indent,
                        });
                    }
                }
            }
        }

        let mut ret = Vec::new();
        dfs(self, Vec::new(), &mut ret, true, 0); // Start with the root node being visible and is_last true, and indent 0
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

    fn as_object(node: &JsonNode) -> Option<(&IndexMap<String, JsonNode>, bool)> {
        if let JsonNode::Object {
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
        use super::*;

        #[test]
        fn test_after_toggle() {
            let mut node = JsonNode::try_from(JSON_STR).unwrap();
            node.toggle(&vec![]);
            assert_eq!(
                vec![JsonSyntaxKind::MapFolded {
                    key: None,
                    path: vec![],
                    is_last: true,
                    indent: 0,
                }],
                node.flatten_visibles(),
            );
        }

        #[test]
        fn test_string() {
            let mut node = JsonNode::try_from("\"string\"").unwrap();
            node.toggle(&vec![]);
            assert_eq!(
                vec![JsonSyntaxKind::ArrayEntry {
                    v: serde_json::Value::String("string".to_string()),
                    path: vec![],
                    is_last: true,
                    indent: 0,
                },],
                node.flatten_visibles(),
            );
        }

        #[test]
        fn test() {
            let node = JsonNode::try_from(JSON_STR).unwrap();
            assert_eq!(
                vec![
                    // {
                    JsonSyntaxKind::MapStart {
                        key: None,
                        path: vec![],
                        indent: 0,
                    },
                    // "number": 1,
                    JsonSyntaxKind::MapEntry {
                        kv: (
                            "number".to_string(),
                            serde_json::Value::Number(serde_json::Number::from(1))
                        ),
                        path: vec![JsonPathSegment::Key("number".to_string())],
                        is_last: false,
                        indent: 1,
                    },
                    // "map": {
                    JsonSyntaxKind::MapStart {
                        key: Some("map".to_string()),
                        path: vec![JsonPathSegment::Key("map".to_string())],
                        indent: 1,
                    },
                    // "string1": "aaa",
                    JsonSyntaxKind::MapEntry {
                        kv: (
                            "string1".to_string(),
                            serde_json::Value::String("aaa".to_string())
                        ),
                        path: vec![
                            JsonPathSegment::Key("map".to_string()),
                            JsonPathSegment::Key("string1".to_string())
                        ],
                        is_last: false,
                        indent: 2,
                    },
                    // "string2": "bbb"
                    JsonSyntaxKind::MapEntry {
                        kv: (
                            "string2".to_string(),
                            serde_json::Value::String("bbb".to_string())
                        ),
                        path: vec![
                            JsonPathSegment::Key("map".to_string()),
                            JsonPathSegment::Key("string2".to_string())
                        ],
                        is_last: true,
                        indent: 2,
                    },
                    // },
                    JsonSyntaxKind::MapEnd {
                        is_last: false,
                        indent: 1
                    },
                    // "list": [
                    JsonSyntaxKind::ArrayStart {
                        key: Some("list".to_string()),
                        path: vec![JsonPathSegment::Key("list".to_string())],
                        indent: 1,
                    },
                    // "abc",
                    JsonSyntaxKind::ArrayEntry {
                        v: serde_json::Value::String("abc".to_string()),
                        path: vec![
                            JsonPathSegment::Key("list".to_string()),
                            JsonPathSegment::Index(0)
                        ],
                        is_last: false,
                        indent: 2,
                    },
                    // "def"
                    JsonSyntaxKind::ArrayEntry {
                        v: serde_json::Value::String("def".to_string()),
                        path: vec![
                            JsonPathSegment::Key("list".to_string()),
                            JsonPathSegment::Index(1)
                        ],
                        is_last: true,
                        indent: 2,
                    },
                    // ],
                    JsonSyntaxKind::ArrayEnd {
                        is_last: false,
                        indent: 1
                    },
                    // "map_in_map": {
                    JsonSyntaxKind::MapStart {
                        key: Some("map_in_map".to_string()),
                        path: vec![JsonPathSegment::Key("map_in_map".to_string())],
                        indent: 1,
                    },
                    // "nested": {
                    JsonSyntaxKind::MapStart {
                        key: Some("nested".to_string()),
                        path: vec![
                            JsonPathSegment::Key("map_in_map".to_string()),
                            JsonPathSegment::Key("nested".to_string())
                        ],
                        indent: 2,
                    },
                    // "leaf": "eof"
                    JsonSyntaxKind::MapEntry {
                        kv: (
                            "leaf".to_string(),
                            serde_json::Value::String("eof".to_string())
                        ),
                        path: vec![
                            JsonPathSegment::Key("map_in_map".to_string()),
                            JsonPathSegment::Key("nested".to_string()),
                            JsonPathSegment::Key("leaf".to_string())
                        ],
                        is_last: true,
                        indent: 3,
                    },
                    // }
                    JsonSyntaxKind::MapEnd {
                        is_last: true,
                        indent: 2
                    },
                    // },
                    JsonSyntaxKind::MapEnd {
                        is_last: false,
                        indent: 1
                    },
                    // "map_in_list": [
                    JsonSyntaxKind::ArrayStart {
                        key: Some("map_in_list".to_string()),
                        path: vec![JsonPathSegment::Key("map_in_list".to_string())],
                        indent: 1,
                    },
                    // {
                    JsonSyntaxKind::MapStart {
                        key: None,
                        path: vec![
                            JsonPathSegment::Key("map_in_list".to_string()),
                            JsonPathSegment::Index(0)
                        ],
                        indent: 2,
                    },
                    // "map1": 1
                    JsonSyntaxKind::MapEntry {
                        kv: (
                            "map1".to_string(),
                            serde_json::Value::Number(serde_json::Number::from(1))
                        ),
                        path: vec![
                            JsonPathSegment::Key("map_in_list".to_string()),
                            JsonPathSegment::Index(0),
                            JsonPathSegment::Key("map1".to_string())
                        ],
                        is_last: true,
                        indent: 3,
                    },
                    // },
                    JsonSyntaxKind::MapEnd {
                        is_last: false,
                        indent: 2
                    },
                    // {
                    JsonSyntaxKind::MapStart {
                        key: None,
                        path: vec![
                            JsonPathSegment::Key("map_in_list".to_string()),
                            JsonPathSegment::Index(1)
                        ],
                        indent: 2,
                    },
                    // "map2": 2
                    JsonSyntaxKind::MapEntry {
                        kv: (
                            "map2".to_string(),
                            serde_json::Value::Number(serde_json::Number::from(2))
                        ),
                        path: vec![
                            JsonPathSegment::Key("map_in_list".to_string()),
                            JsonPathSegment::Index(1),
                            JsonPathSegment::Key("map2".to_string())
                        ],
                        is_last: true,
                        indent: 3,
                    },
                    // }
                    JsonSyntaxKind::MapEnd {
                        is_last: true,
                        indent: 2
                    },
                    // ]
                    JsonSyntaxKind::ArrayEnd {
                        is_last: true,
                        indent: 1
                    },
                    // }
                    JsonSyntaxKind::MapEnd {
                        is_last: true,
                        indent: 0
                    },
                ],
                node.flatten_visibles(),
            );
        }
    }

    mod toggle {
        use super::*;

        #[test]
        fn test() {
            let mut node = JsonNode::try_from(JSON_STR).unwrap();
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
        use super::*;

        #[test]
        fn test() {
            let node = JsonNode::try_from(JSON_STR).unwrap();
            assert_eq!(Some(&node.clone()), node.get(&vec![]));
        }

        #[test]
        fn test_with_invalid_path() {
            let node = JsonNode::try_from(JSON_STR).unwrap();
            assert_eq!(
                None,
                node.get(&vec![
                    JsonPathSegment::Key("map".to_string()),
                    JsonPathSegment::Key("invalid_segment".to_string()),
                ],)
            );
        }
    }

    mod get_mut {
        use super::*;

        #[test]
        fn test() {
            let mut node = JsonNode::try_from(JSON_STR).unwrap();
            assert_eq!(Some(&mut node.clone()), node.get_mut(&vec![]));
        }

        #[test]
        fn test_with_invalid_path() {
            let mut node = JsonNode::try_from(JSON_STR).unwrap();
            assert_eq!(
                None,
                node.get_mut(&vec![
                    JsonPathSegment::Key("map".to_string()),
                    JsonPathSegment::Key("invalid_segment".to_string()),
                ],)
            );
        }
    }

    mod from_str {
        use super::*;
        use serde_json::Number;

        #[test]
        fn test() {
            assert_eq!(
                JsonNode::Object {
                    children: IndexMap::from_iter(vec![
                        (
                            String::from("number"),
                            JsonNode::Leaf(serde_json::Value::Number(Number::from(1)))
                        ),
                        (
                            String::from("map"),
                            JsonNode::Object {
                                children: IndexMap::from_iter(vec![
                                    (
                                        String::from("string1"),
                                        JsonNode::Leaf(serde_json::Value::String(String::from(
                                            "aaa"
                                        )))
                                    ),
                                    (
                                        String::from("string2"),
                                        JsonNode::Leaf(serde_json::Value::String(String::from(
                                            "bbb"
                                        )))
                                    ),
                                ]),
                                children_visible: true,
                            }
                        ),
                        (
                            String::from("list"),
                            JsonNode::Array {
                                children: vec![
                                    JsonNode::Leaf(serde_json::Value::String(String::from("abc"))),
                                    JsonNode::Leaf(serde_json::Value::String(String::from("def"))),
                                ],
                                children_visible: true,
                            }
                        ),
                        (
                            String::from("map_in_map"),
                            JsonNode::Object {
                                children: IndexMap::from_iter(vec![(
                                    String::from("nested"),
                                    JsonNode::Object {
                                        children: IndexMap::from_iter(vec![(
                                            String::from("leaf"),
                                            JsonNode::Leaf(serde_json::Value::String(
                                                String::from("eof")
                                            ))
                                        ),]),
                                        children_visible: true,
                                    }
                                ),]),
                                children_visible: true,
                            }
                        ),
                        (
                            String::from("map_in_list"),
                            JsonNode::Array {
                                children: vec![
                                    JsonNode::Object {
                                        children: IndexMap::from_iter(vec![(
                                            String::from("map1"),
                                            JsonNode::Leaf(serde_json::Value::Number(
                                                Number::from(1)
                                            ))
                                        ),]),
                                        children_visible: true,
                                    },
                                    JsonNode::Object {
                                        children: IndexMap::from_iter(vec![(
                                            String::from("map2"),
                                            JsonNode::Leaf(serde_json::Value::Number(
                                                Number::from(2)
                                            ))
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
                JsonNode::try_from(JSON_STR).unwrap(),
            );
        }
    }
}

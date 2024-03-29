use indexmap::IndexMap;

use crate::serde_json;

/// Represents the various kinds of syntax elements found in a JSON document.
/// This includes the start and end of maps and arrays, entries within maps and arrays,
/// and folded representations of maps and arrays for compact display.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum JsonSyntaxKind {
    /// Represents the start of a map. Optionally contains the key if this map is an entry in another map,
    /// the path to this map in the JSON document, and the indentation level for formatting.
    MapStart {
        key: Option<String>,
        path: JsonPath,
        indent: usize,
    },
    /// Represents the end of a map. Contains a flag indicating if this is the last element in its parent
    /// and the indentation level for formatting.
    MapEnd { is_last: bool, indent: usize },
    /// Represents a map that is folded (i.e., its contents are not displayed). Contains the same information as `MapStart`
    /// plus a flag indicating if this is the last element in its parent.
    MapFolded {
        key: Option<String>,
        path: JsonPath,
        is_last: bool,
        indent: usize,
    },
    /// Represents an entry in a map, containing the key-value pair, the path to this entry,
    /// a flag indicating if this is the last element in its parent, and the indentation level for formatting.
    MapEntry {
        kv: (String, serde_json::Value),
        path: JsonPath,
        is_last: bool,
        indent: usize,
    },
    /// Represents the start of an array. Optionally contains the key if this array is an entry in a map,
    /// the path to this array in the JSON document, and the indentation level for formatting.
    ArrayStart {
        key: Option<String>,
        path: JsonPath,
        indent: usize,
    },
    /// Represents the end of an array. Contains a flag indicating if this is the last element in its parent
    /// and the indentation level for formatting.
    ArrayEnd { is_last: bool, indent: usize },
    /// Represents an array that is folded (i.e., its contents are not displayed). Contains the same information as `ArrayStart`
    /// plus a flag indicating if this is the last element in its parent.
    ArrayFolded {
        key: Option<String>,
        path: JsonPath,
        is_last: bool,
        indent: usize,
    },
    /// Represents an entry in an array, containing the value, the path to this entry,
    /// a flag indicating if this is the last element in its parent, and the indentation level for formatting.
    ArrayEntry {
        v: serde_json::Value,
        path: JsonPath,
        is_last: bool,
        indent: usize,
    },
}

impl JsonSyntaxKind {
    /// Retrieves the path of the `JsonSyntaxKind` if available.
    ///
    /// # Returns
    /// An `Option<&JsonPath>` representing the path of the syntax kind, or `None` if not applicable.
    pub fn path(&self) -> Option<&JsonPath> {
        match self {
            JsonSyntaxKind::MapStart { path, .. }
            | JsonSyntaxKind::MapFolded { path, .. }
            | JsonSyntaxKind::MapEntry { path, .. }
            | JsonSyntaxKind::ArrayStart { path, .. }
            | JsonSyntaxKind::ArrayFolded { path, .. }
            | JsonSyntaxKind::ArrayEntry { path, .. } => Some(path),
            _ => None,
        }
    }
}

pub type JsonPath = Vec<JsonPathSegment>;

/// Represents a segment of a path in a JSON document, which can be either a key in an object
/// or an index in an array.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum JsonPathSegment {
    /// Represents a key in a JSON object.
    Key(String),
    /// Represents an index in a JSON array.
    Index(usize),
}

/// Represents a node in a JSON structure, which can be an object, an array, or a leaf value.
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

impl JsonNode {
    /// Creates a `JsonNode` from a `serde_json::Value` with visibility for all children set to true if `depth` is `None`,
    /// or up to a specified depth if `depth` is `Some(usize)`.
    ///
    /// # Arguments
    /// * `value` - The `serde_json::Value` to convert into a `JsonNode`.
    /// * `depth` - An `Option<usize>` representing the depth up to which child nodes should be visible.
    ///   A depth of `Some(0)` means only the root is visible. `None` means all children are visible.
    ///
    /// # Returns
    /// A `JsonNode` with children visibility set according to the specified depth.
    pub fn new(value: serde_json::Value, depth: Option<usize>) -> Self {
        match value {
            serde_json::Value::Object(map) => {
                let children = map
                    .into_iter()
                    .map(|(k, v)| (k, JsonNode::new(v, depth.map(|d| d.saturating_sub(1)))))
                    .collect();
                JsonNode::Object {
                    children,
                    children_visible: depth.map_or(true, |d| d > 0),
                }
            }
            serde_json::Value::Array(vec) => {
                let children = vec
                    .into_iter()
                    .map(|v| JsonNode::new(v, depth.map(|d| d.saturating_sub(1))))
                    .collect();
                JsonNode::Array {
                    children,
                    children_visible: depth.map_or(true, |d| d > 0),
                }
            }
            _ => JsonNode::Leaf(value),
        }
    }

    /// Retrieves a reference to a `JsonNode` at a specified JSON path.
    ///
    /// # Arguments
    /// * `path` - A vector of `JsonPathSegment` indicating the path to the node.
    ///
    /// # Returns
    /// An `Option` containing a reference to the found node, or `None` if not found.
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

    /// Retrieves a mutable reference to a `JsonNode` at a specified JSON path.
    ///
    /// # Arguments
    /// * `path` - A vector of `JsonPathSegment` indicating the path to the node.
    ///
    /// # Returns
    /// An `Option` containing a mutable reference to the found node, or `None` if not found.
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

    /// Toggles the visibility of children for a `JsonNode` at a specified JSON path.
    ///
    /// # Arguments
    /// * `path` - A vector of `JsonPathSegment` indicating the path to the node.
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

    /// Flattens the visible parts of the JSON structure into a vector of `JsonSyntaxKind`.
    ///
    /// # Returns
    /// A vector of `JsonSyntaxKind` representing the visible parts of the JSON structure.
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
    use crate::{json::JsonStream, serde_json::Deserializer};

    use super::*;

    const JSON_STR: &str = r#"{
        "number": 1,
        "map": {
          "string1": "aaa"
        },
        "list": [
          "abc"
        ]
    }"#;

    fn test_json_node() -> JsonNode {
        let stream = Deserializer::from_str(JSON_STR)
            .into_iter::<serde_json::Value>()
            .filter_map(Result::ok);
        JsonStream::new(stream, None).get_root(0).unwrap().clone()
    }

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

    mod new {
        use super::*;
        use crate::serde_json::json;

        #[test]
        fn test_one_level_depth() {
            let value: serde_json::Value = serde_json::from_str(JSON_STR).unwrap();
            let node = JsonNode::new(value, Some(1));
            let expected = JsonNode::Object {
                children: IndexMap::from_iter(vec![
                    ("number".to_string(), JsonNode::Leaf(json!(1))),
                    (
                        "map".to_string(),
                        JsonNode::Object {
                            children: IndexMap::from_iter(vec![(
                                "string1".to_string(),
                                JsonNode::Leaf(json!("aaa")),
                            )]),
                            children_visible: false,
                        },
                    ),
                    (
                        "list".to_string(),
                        JsonNode::Array {
                            children: vec![JsonNode::Leaf(json!("abc"))],
                            children_visible: false,
                        },
                    ),
                ]),
                children_visible: true,
            };
            assert_eq!(node, expected);
        }
    }

    mod flatten_visibles {
        use super::*;

        #[test]
        fn test() {
            let node = test_json_node();
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
                        is_last: true,
                        indent: 2,
                    },
                    // ],
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

        #[test]
        fn test_after_toggle() {
            let mut node = test_json_node();
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
    }

    mod toggle {
        use super::*;

        #[test]
        fn test() {
            let mut node = test_json_node();
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
            let node = test_json_node();
            assert_eq!(Some(&node.clone()), node.get(&vec![]));
        }

        #[test]
        fn test_with_invalid_path() {
            let node = test_json_node();
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
            let mut node = test_json_node();
            assert_eq!(Some(&mut node.clone()), node.get_mut(&vec![]));
        }

        #[test]
        fn test_with_invalid_path() {
            let mut node = test_json_node();
            assert_eq!(
                None,
                node.get_mut(&vec![
                    JsonPathSegment::Key("map".to_string()),
                    JsonPathSegment::Key("invalid_segment".to_string()),
                ],)
            );
        }
    }

    mod try_from {
        use super::*;
        use crate::serde_json::Number;

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
                                children: IndexMap::from_iter(vec![(
                                    String::from("string1"),
                                    JsonNode::Leaf(serde_json::Value::String(String::from("aaa")))
                                ),]),
                                children_visible: true,
                            }
                        ),
                        (
                            String::from("list"),
                            JsonNode::Array {
                                children: vec![JsonNode::Leaf(serde_json::Value::String(
                                    String::from("abc")
                                )),],
                                children_visible: true,
                            }
                        ),
                    ]),
                    children_visible: true,
                },
                test_json_node(),
            );
        }
    }
}

use indexmap::IndexMap;
use serde_json::{self, Result, Value};

#[derive(Debug, PartialEq, Eq)]
pub enum Node {
    Object {
        children: IndexMap<String, Node>,
        children_visible: bool,
    },
    Array {
        children: Vec<Node>,
        children_visible: bool,
    },
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

    pub fn try_new(json_str: &str) -> Result<Self> {
        let value: Value = serde_json::from_str(json_str)?;
        Ok(Node::new(value))
    }
}

#[cfg(test)]
mod test {
    use super::Node;

    const json_str: &str = r#"
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

    mod try_new {
        use serde_json::Number;

        use super::super::*;
        use super::json_str;

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
                Node::try_new(json_str).unwrap(),
            );
        }
    }
}

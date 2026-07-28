use std::str::FromStr;

use serde_json::Deserializer;

use promkit_widgets::json::jsonz::*;

#[test]
fn creates_rows_for_empty_containers() {
    let values: Vec<_> = Deserializer::from_str(
        r#"
            {}
            []
        "#,
    )
    .into_iter::<serde_json::Value>()
    .filter_map(serde_json::Result::ok)
    .collect();

    let rows = create_rows(values.iter());
    assert_eq!(rows.len(), 2);

    assert_eq!(
        rows[0],
        Row {
            depth: 0,
            key: None,
            node: JsonNode::Container(ContainerNode::Empty {
                typ: ContainerType::Object
            }),
        }
    );

    assert_eq!(
        rows[1],
        Row {
            depth: 0,
            key: None,
            node: JsonNode::Container(ContainerNode::Empty {
                typ: ContainerType::Array
            }),
        }
    );
}

#[test]
fn creates_rows_for_a_nested_object() {
    let input = serde_json::Value::from_str(
        r#"
            {
                "a": {
                    "b": {
                        "c": "value"
                    }
                }
            }
        "#,
    )
    .unwrap();

    let rows = create_rows([&input]);

    assert_eq!(
        rows[0],
        Row {
            depth: 0,
            key: None,
            node: JsonNode::Container(ContainerNode::Open {
                typ: ContainerType::Object,
                collapsed: false,
                close_index: 6,
            }),
        }
    );

    assert_eq!(
        rows[1],
        Row {
            depth: 1,
            key: Some("a".to_string()),
            node: JsonNode::Container(ContainerNode::Open {
                typ: ContainerType::Object,
                collapsed: false,
                close_index: 5,
            }),
        }
    );

    assert_eq!(
        rows[2],
        Row {
            depth: 2,
            key: Some("b".to_string()),
            node: JsonNode::Container(ContainerNode::Open {
                typ: ContainerType::Object,
                collapsed: false,
                close_index: 4,
            }),
        }
    );

    assert_eq!(
        rows[3],
        Row {
            depth: 3,
            key: Some("c".to_string()),
            node: JsonNode::String("value".to_string()),
        }
    );

    assert_eq!(
        rows[4],
        Row {
            depth: 2,
            key: None,
            node: JsonNode::Container(ContainerNode::Close {
                typ: ContainerType::Object,
                collapsed: false,
                open_index: 2,
            }),
        }
    );

    assert_eq!(
        rows[5],
        Row {
            depth: 1,
            key: None,
            node: JsonNode::Container(ContainerNode::Close {
                typ: ContainerType::Object,
                collapsed: false,
                open_index: 1,
            }),
        }
    );

    assert_eq!(
        rows[6],
        Row {
            depth: 0,
            key: None,
            node: JsonNode::Container(ContainerNode::Close {
                typ: ContainerType::Object,
                collapsed: false,
                open_index: 0,
            }),
        }
    );
}

#[test]
fn creates_rows_for_a_nested_array() {
    let input = serde_json::Value::from_str(
        r#"
            [
                [
                    [
                        1,
                        2,
                        3
                    ]
                ]
            ]
        "#,
    )
    .unwrap();

    let rows = create_rows([&input]);

    assert_eq!(
        rows[0],
        Row {
            depth: 0,
            key: None,
            node: JsonNode::Container(ContainerNode::Open {
                typ: ContainerType::Array,
                collapsed: false,
                close_index: 8,
            }),
        }
    );

    assert_eq!(
        rows[1],
        Row {
            depth: 1,
            key: None,
            node: JsonNode::Container(ContainerNode::Open {
                typ: ContainerType::Array,
                collapsed: false,
                close_index: 7,
            }),
        }
    );

    assert_eq!(
        rows[2],
        Row {
            depth: 2,
            key: None,
            node: JsonNode::Container(ContainerNode::Open {
                typ: ContainerType::Array,
                collapsed: false,
                close_index: 6,
            }),
        }
    );

    for (i, num) in [1, 2, 3].iter().enumerate() {
        assert_eq!(
            rows[3 + i],
            Row {
                depth: 3,
                key: None,
                node: JsonNode::Number(serde_json::Number::from(*num)),
            }
        );
    }

    assert_eq!(
        rows[6],
        Row {
            depth: 2,
            key: None,
            node: JsonNode::Container(ContainerNode::Close {
                typ: ContainerType::Array,
                collapsed: false,
                open_index: 2,
            }),
        }
    );

    assert_eq!(
        rows[7],
        Row {
            depth: 1,
            key: None,
            node: JsonNode::Container(ContainerNode::Close {
                typ: ContainerType::Array,
                collapsed: false,
                open_index: 1,
            }),
        }
    );

    assert_eq!(
        rows[8],
        Row {
            depth: 0,
            key: None,
            node: JsonNode::Container(ContainerNode::Close {
                typ: ContainerType::Array,
                collapsed: false,
                open_index: 0,
            }),
        }
    );
}

#[test]
fn creates_rows_for_mixed_containers() {
    let input = serde_json::Value::from_str(
        r#"
            {
                "array": [
                    {
                        "key": "value"
                    },
                    [
                        1,
                        2,
                        3
                    ]
                ],
                "object": {
                    "nested": true
                }
            }
        "#,
    )
    .unwrap();

    let rows = create_rows([&input]);

    assert_eq!(
        rows[0],
        Row {
            depth: 0,
            key: None,
            node: JsonNode::Container(ContainerNode::Open {
                typ: ContainerType::Object,
                collapsed: false,
                close_index: 14,
            }),
        }
    );

    assert_eq!(
        rows[1],
        Row {
            depth: 1,
            key: Some("array".to_string()),
            node: JsonNode::Container(ContainerNode::Open {
                typ: ContainerType::Array,
                collapsed: false,
                close_index: 10,
            }),
        }
    );

    assert_eq!(
        rows[2],
        Row {
            depth: 2,
            key: None,
            node: JsonNode::Container(ContainerNode::Open {
                typ: ContainerType::Object,
                collapsed: false,
                close_index: 4,
            }),
        }
    );

    assert_eq!(
        rows[3],
        Row {
            depth: 3,
            key: Some("key".to_string()),
            node: JsonNode::String("value".to_string()),
        }
    );

    assert_eq!(
        rows[4],
        Row {
            depth: 2,
            key: None,
            node: JsonNode::Container(ContainerNode::Close {
                typ: ContainerType::Object,
                collapsed: false,
                open_index: 2,
            }),
        }
    );

    assert_eq!(
        rows[5],
        Row {
            depth: 2,
            key: None,
            node: JsonNode::Container(ContainerNode::Open {
                typ: ContainerType::Array,
                collapsed: false,
                close_index: 9,
            }),
        }
    );

    for (i, num) in [1, 2, 3].iter().enumerate() {
        assert_eq!(
            rows[6 + i],
            Row {
                depth: 3,
                key: None,
                node: JsonNode::Number(serde_json::Number::from(*num)),
            }
        );
    }

    assert_eq!(
        rows[9],
        Row {
            depth: 2,
            key: None,
            node: JsonNode::Container(ContainerNode::Close {
                typ: ContainerType::Array,
                collapsed: false,
                open_index: 5,
            }),
        }
    );

    assert_eq!(
        rows[10],
        Row {
            depth: 1,
            key: None,
            node: JsonNode::Container(ContainerNode::Close {
                typ: ContainerType::Array,
                collapsed: false,
                open_index: 1,
            }),
        }
    );

    assert_eq!(
        rows[11],
        Row {
            depth: 1,
            key: Some("object".to_string()),
            node: JsonNode::Container(ContainerNode::Open {
                typ: ContainerType::Object,
                collapsed: false,
                close_index: 13,
            }),
        }
    );

    assert_eq!(
        rows[12],
        Row {
            depth: 2,
            key: Some("nested".to_string()),
            node: JsonNode::Boolean(true),
        }
    );

    assert_eq!(
        rows[13],
        Row {
            depth: 1,
            key: None,
            node: JsonNode::Container(ContainerNode::Close {
                typ: ContainerType::Object,
                collapsed: false,
                open_index: 11,
            }),
        }
    );

    assert_eq!(
        rows[14],
        Row {
            depth: 0,
            key: None,
            node: JsonNode::Container(ContainerNode::Close {
                typ: ContainerType::Object,
                collapsed: false,
                open_index: 0,
            }),
        }
    );
}

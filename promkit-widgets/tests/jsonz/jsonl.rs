use serde_json::Deserializer;

use promkit_widgets::json::jsonz::*;

#[test]
fn test_basic_jsonl() {
    let inputs: Vec<_> = Deserializer::from_str(
        r#"
            {
                "name": "Alice",
                "age": 30
            }
            {
                "name": "Bob",
                "age": 25
            }
            {
                "name": "Charlie",
                "age": 35
            }
        "#,
    )
    .into_iter::<serde_json::Value>()
    .filter_map(serde_json::Result::ok)
    .collect();

    let mut rows = create_rows(inputs.iter());

    assert_eq!(
        rows[0],
        Row {
            depth: 0,
            key: None,
            node: JsonNode::Container(ContainerNode::Open {
                typ: ContainerType::Object,
                collapsed: false,
                close_index: 3,
            }),
        }
    );
    assert_eq!(
        rows[1],
        Row {
            depth: 1,
            key: Some("name".to_string()),
            node: JsonNode::String("Alice".to_string()),
        }
    );
    assert_eq!(
        rows[2],
        Row {
            depth: 1,
            key: Some("age".to_string()),
            node: JsonNode::Number(serde_json::Number::from(30)),
        }
    );
    assert_eq!(
        rows[3],
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

    assert_eq!(
        rows[4],
        Row {
            depth: 0,
            key: None,
            node: JsonNode::Container(ContainerNode::Open {
                typ: ContainerType::Object,
                collapsed: false,
                close_index: 7,
            }),
        }
    );
    assert_eq!(
        rows[5],
        Row {
            depth: 1,
            key: Some("name".to_string()),
            node: JsonNode::String("Bob".to_string()),
        }
    );
    assert_eq!(
        rows[6],
        Row {
            depth: 1,
            key: Some("age".to_string()),
            node: JsonNode::Number(serde_json::Number::from(25)),
        }
    );
    assert_eq!(
        rows[7],
        Row {
            depth: 0,
            key: None,
            node: JsonNode::Container(ContainerNode::Close {
                typ: ContainerType::Object,
                collapsed: false,
                open_index: 4,
            }),
        }
    );

    rows.toggle(0);
    assert_eq!(
        rows[0].node,
        JsonNode::Container(ContainerNode::Open {
            typ: ContainerType::Object,
            collapsed: true,
            close_index: 3,
        })
    );
    assert_eq!(
        rows[3].node,
        JsonNode::Container(ContainerNode::Close {
            typ: ContainerType::Object,
            collapsed: true,
            open_index: 0,
        })
    );

    assert_eq!(rows.up(4), 0);
    assert_eq!(rows.down(0), 4);
}

#[test]
fn test_mixed_jsonl() {
    let inputs: Vec<_> = Deserializer::from_str(
        r#"
            {
                "array": [
                    1,
                    2,
                    3
                ]
            }
            [
                {
                    "nested": true
                },
                {
                    "nested": false
                }
            ]
            {
                "empty": {}
            }
        "#,
    )
    .into_iter::<serde_json::Value>()
    .filter_map(serde_json::Result::ok)
    .collect();

    let mut rows = create_rows(inputs.iter());

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
            key: Some("array".to_string()),
            node: JsonNode::Container(ContainerNode::Open {
                typ: ContainerType::Array,
                collapsed: false,
                close_index: 5,
            }),
        }
    );

    for (i, num) in [1, 2, 3].iter().enumerate() {
        assert_eq!(
            rows[2 + i],
            Row {
                depth: 2,
                key: None,
                node: JsonNode::Number(serde_json::Number::from(*num)),
            }
        );
    }

    let array_start = 7;
    assert_eq!(
        rows[array_start],
        Row {
            depth: 0,
            key: None,
            node: JsonNode::Container(ContainerNode::Open {
                typ: ContainerType::Array,
                collapsed: false,
                close_index: 14,
            }),
        }
    );

    assert_eq!(
        rows[array_start + 1],
        Row {
            depth: 1,
            key: None,
            node: JsonNode::Container(ContainerNode::Open {
                typ: ContainerType::Object,
                collapsed: false,
                close_index: 10,
            }),
        }
    );

    let extracted = rows.extract(array_start + 1, 3);
    assert_eq!(extracted.len(), 3);
    assert_eq!(
        extracted[0],
        Row {
            depth: 1,
            key: None,
            node: JsonNode::Container(ContainerNode::Open {
                typ: ContainerType::Object,
                collapsed: false,
                close_index: 10,
            }),
        }
    );
    assert_eq!(
        extracted[1],
        Row {
            depth: 2,
            key: Some("nested".to_string()),
            node: JsonNode::Boolean(true),
        }
    );
    assert_eq!(
        extracted[2],
        Row {
            depth: 1,
            key: None,
            node: JsonNode::Container(ContainerNode::Close {
                typ: ContainerType::Object,
                collapsed: false,
                open_index: array_start + 1,
            }),
        }
    );

    rows.toggle(array_start);
    assert_eq!(
        rows[array_start].node,
        JsonNode::Container(ContainerNode::Open {
            typ: ContainerType::Array,
            collapsed: true,
            close_index: 14,
        })
    );
}

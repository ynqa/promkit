#[cfg(test)]
mod create_rows {
    use std::str::FromStr;

    use promkit::jsonz::*;
    use serde_json::Deserializer;

    #[test]
    fn test_empty_containers() {
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
                k: None,
                v: Value::Empty {
                    typ: ContainerType::Object
                },
            }
        );

        assert_eq!(
            rows[1],
            Row {
                depth: 0,
                k: None,
                v: Value::Empty {
                    typ: ContainerType::Array
                },
            }
        );
    }

    #[test]
    fn test_nested_object() {
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
                k: None,
                v: Value::Open {
                    typ: ContainerType::Object,
                    collapsed: false,
                    close_index: 6,
                },
            }
        );

        assert_eq!(
            rows[1],
            Row {
                depth: 1,
                k: Some("a".to_string()),
                v: Value::Open {
                    typ: ContainerType::Object,
                    collapsed: false,
                    close_index: 5,
                },
            }
        );

        assert_eq!(
            rows[2],
            Row {
                depth: 2,
                k: Some("b".to_string()),
                v: Value::Open {
                    typ: ContainerType::Object,
                    collapsed: false,
                    close_index: 4,
                },
            }
        );

        assert_eq!(
            rows[3],
            Row {
                depth: 3,
                k: Some("c".to_string()),
                v: Value::String("value".to_string()),
            }
        );

        assert_eq!(
            rows[4],
            Row {
                depth: 2,
                k: None,
                v: Value::Close {
                    typ: ContainerType::Object,
                    collapsed: false,
                    open_index: 2,
                },
            }
        );

        assert_eq!(
            rows[5],
            Row {
                depth: 1,
                k: None,
                v: Value::Close {
                    typ: ContainerType::Object,
                    collapsed: false,
                    open_index: 1,
                },
            }
        );

        assert_eq!(
            rows[6],
            Row {
                depth: 0,
                k: None,
                v: Value::Close {
                    typ: ContainerType::Object,
                    collapsed: false,
                    open_index: 0,
                },
            }
        );
    }

    #[test]
    fn test_nested_array() {
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
                k: None,
                v: Value::Open {
                    typ: ContainerType::Array,
                    collapsed: false,
                    close_index: 8,
                },
            }
        );

        assert_eq!(
            rows[1],
            Row {
                depth: 1,
                k: None,
                v: Value::Open {
                    typ: ContainerType::Array,
                    collapsed: false,
                    close_index: 7,
                },
            }
        );

        assert_eq!(
            rows[2],
            Row {
                depth: 2,
                k: None,
                v: Value::Open {
                    typ: ContainerType::Array,
                    collapsed: false,
                    close_index: 6,
                },
            }
        );

        for (i, num) in [1, 2, 3].iter().enumerate() {
            assert_eq!(
                rows[3 + i],
                Row {
                    depth: 3,
                    k: None,
                    v: Value::Number(serde_json::Number::from(*num)),
                }
            );
        }

        assert_eq!(
            rows[6],
            Row {
                depth: 2,
                k: None,
                v: Value::Close {
                    typ: ContainerType::Array,
                    collapsed: false,
                    open_index: 2,
                },
            }
        );

        assert_eq!(
            rows[7],
            Row {
                depth: 1,
                k: None,
                v: Value::Close {
                    typ: ContainerType::Array,
                    collapsed: false,
                    open_index: 1,
                },
            }
        );

        assert_eq!(
            rows[8],
            Row {
                depth: 0,
                k: None,
                v: Value::Close {
                    typ: ContainerType::Array,
                    collapsed: false,
                    open_index: 0,
                },
            }
        );
    }

    #[test]
    fn test_mixed_containers() {
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
                k: None,
                v: Value::Open {
                    typ: ContainerType::Object,
                    collapsed: false,
                    close_index: 14,
                },
            }
        );

        assert_eq!(
            rows[1],
            Row {
                depth: 1,
                k: Some("array".to_string()),
                v: Value::Open {
                    typ: ContainerType::Array,
                    collapsed: false,
                    close_index: 10,
                },
            }
        );

        assert_eq!(
            rows[2],
            Row {
                depth: 2,
                k: None,
                v: Value::Open {
                    typ: ContainerType::Object,
                    collapsed: false,
                    close_index: 4,
                },
            }
        );

        assert_eq!(
            rows[3],
            Row {
                depth: 3,
                k: Some("key".to_string()),
                v: Value::String("value".to_string()),
            }
        );

        assert_eq!(
            rows[4],
            Row {
                depth: 2,
                k: None,
                v: Value::Close {
                    typ: ContainerType::Object,
                    collapsed: false,
                    open_index: 2,
                },
            }
        );

        assert_eq!(
            rows[5],
            Row {
                depth: 2,
                k: None,
                v: Value::Open {
                    typ: ContainerType::Array,
                    collapsed: false,
                    close_index: 9,
                },
            }
        );

        for (i, num) in [1, 2, 3].iter().enumerate() {
            assert_eq!(
                rows[6 + i],
                Row {
                    depth: 3,
                    k: None,
                    v: Value::Number(serde_json::Number::from(*num)),
                }
            );
        }

        assert_eq!(
            rows[9],
            Row {
                depth: 2,
                k: None,
                v: Value::Close {
                    typ: ContainerType::Array,
                    collapsed: false,
                    open_index: 5,
                },
            }
        );

        assert_eq!(
            rows[10],
            Row {
                depth: 1,
                k: None,
                v: Value::Close {
                    typ: ContainerType::Array,
                    collapsed: false,
                    open_index: 1,
                },
            }
        );

        assert_eq!(
            rows[11],
            Row {
                depth: 1,
                k: Some("object".to_string()),
                v: Value::Open {
                    typ: ContainerType::Object,
                    collapsed: false,
                    close_index: 13,
                },
            }
        );

        assert_eq!(
            rows[12],
            Row {
                depth: 2,
                k: Some("nested".to_string()),
                v: Value::Boolean(true),
            }
        );

        assert_eq!(
            rows[13],
            Row {
                depth: 1,
                k: None,
                v: Value::Close {
                    typ: ContainerType::Object,
                    collapsed: false,
                    open_index: 11,
                },
            }
        );

        assert_eq!(
            rows[14],
            Row {
                depth: 0,
                k: None,
                v: Value::Close {
                    typ: ContainerType::Object,
                    collapsed: false,
                    open_index: 0,
                },
            }
        );
    }
}

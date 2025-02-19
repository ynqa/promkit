#[cfg(test)]
mod toggle {
    use std::str::FromStr;

    use promkit::jsonz::*;

    #[test]
    fn test_on_open() {
        let input = serde_json::Value::from_str(
            r#"
                {
                    "object": {
                        "key": "value"
                    },
                    "array": [
                        1,
                        2,
                        3
                    ]
                }
            "#,
        )
        .unwrap();

        let mut rows = create_rows([&input]);

        let index = rows.toggle(1);
        assert_eq!(index, 1);
        assert_eq!(
            rows[1].v,
            Value::Open {
                typ: ContainerType::Object,
                collapsed: true,
                close_index: 3
            }
        );
        assert_eq!(
            rows[3].v,
            Value::Close {
                typ: ContainerType::Object,
                collapsed: true,
                open_index: 1
            }
        );

        let index = rows.toggle(4);
        assert_eq!(index, 4);
        assert_eq!(
            rows[4].v,
            Value::Open {
                typ: ContainerType::Array,
                collapsed: true,
                close_index: 8
            }
        );
        assert_eq!(
            rows[8].v,
            Value::Close {
                typ: ContainerType::Array,
                collapsed: true,
                open_index: 4
            }
        );

        let index = rows.toggle(0);
        assert_eq!(index, 0);
        assert_eq!(
            rows[0].v,
            Value::Open {
                typ: ContainerType::Object,
                collapsed: true,
                close_index: 9
            }
        );
        assert_eq!(
            rows[9].v,
            Value::Close {
                typ: ContainerType::Object,
                collapsed: true,
                open_index: 0
            }
        );

        rows.toggle(0);
        assert_eq!(
            rows[0].v,
            Value::Open {
                typ: ContainerType::Object,
                collapsed: false,
                close_index: 9
            }
        );
        assert_eq!(
            rows[9].v,
            Value::Close {
                typ: ContainerType::Object,
                collapsed: false,
                open_index: 0
            }
        );

        rows.toggle(4);
        assert_eq!(
            rows[4].v,
            Value::Open {
                typ: ContainerType::Array,
                collapsed: false,
                close_index: 8
            }
        );
        assert_eq!(
            rows[8].v,
            Value::Close {
                typ: ContainerType::Array,
                collapsed: false,
                open_index: 4
            }
        );

        rows.toggle(1);
        assert_eq!(
            rows[1].v,
            Value::Open {
                typ: ContainerType::Object,
                collapsed: false,
                close_index: 3
            }
        );
        assert_eq!(
            rows[3].v,
            Value::Close {
                typ: ContainerType::Object,
                collapsed: false,
                open_index: 1
            }
        );
    }

    #[test]
    fn test_on_close() {
        let input = serde_json::Value::from_str(
            r#"
                {
                    "object": {
                        "key": "value"
                    },
                    "array": [
                        1,
                        2,
                        3
                    ]
                }
            "#,
        )
        .unwrap();

        let mut rows = create_rows([&input]);

        let index = rows.toggle(3);
        assert_eq!(index, 1);
        assert_eq!(
            rows[1].v,
            Value::Open {
                typ: ContainerType::Object,
                collapsed: true,
                close_index: 3
            }
        );
        assert_eq!(
            rows[3].v,
            Value::Close {
                typ: ContainerType::Object,
                collapsed: true,
                open_index: 1
            }
        );

        let index = rows.toggle(8);
        assert_eq!(index, 4);
        assert_eq!(
            rows[4].v,
            Value::Open {
                typ: ContainerType::Array,
                collapsed: true,
                close_index: 8
            }
        );
        assert_eq!(
            rows[8].v,
            Value::Close {
                typ: ContainerType::Array,
                collapsed: true,
                open_index: 4
            }
        );

        let index = rows.toggle(9);
        assert_eq!(index, 0);
        assert_eq!(
            rows[0].v,
            Value::Open {
                typ: ContainerType::Object,
                collapsed: true,
                close_index: 9
            }
        );
        assert_eq!(
            rows[9].v,
            Value::Close {
                typ: ContainerType::Object,
                collapsed: true,
                open_index: 0
            }
        );
    }
}

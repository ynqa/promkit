#[cfg(test)]
mod down {
    use std::str::FromStr;

    use promkit::jsonz::*;

    #[test]
    fn test_collapsed_containers() {
        let input = serde_json::Value::from_str(
            r#"
                {
                    "collapsed_object": {
                        "key": "value"
                    },
                    "collapsed_array": [
                        1,
                        2,
                        3
                    ]
                }
            "#,
        )
        .unwrap();

        let mut rows = create_rows([input]);
        rows[1].v = Value::Open {
            typ: ContainerType::Object,
            collapsed: true,
            close_index: 3,
        };
        rows[3].v = Value::Close {
            typ: ContainerType::Object,
            collapsed: true,
            open_index: 1,
        };

        rows[4].v = Value::Open {
            typ: ContainerType::Array,
            collapsed: true,
            close_index: 8,
        };
        rows[8].v = Value::Close {
            typ: ContainerType::Array,
            collapsed: true,
            open_index: 4,
        };
        assert_eq!(rows.down(0), 1);
        assert_eq!(rows.down(1), 4);
        assert_eq!(rows.down(4), 9);
        assert_eq!(rows.down(9), 9);
    }
}

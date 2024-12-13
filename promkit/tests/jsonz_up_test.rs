#[cfg(test)]
mod up {
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
        rows.toggle(1);
        rows.toggle(4);

        assert_eq!(rows.up(0), 0);
        assert_eq!(rows.up(1), 0);
        assert_eq!(rows.up(4), 1);
        assert_eq!(rows.up(9), 4);
    }
}

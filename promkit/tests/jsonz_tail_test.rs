#[cfg(test)]
mod tail {
    use std::str::FromStr;

    use promkit::jsonz::*;
    use serde_json::Deserializer;

    #[test]
    fn test() {
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
        assert_eq!(rows.tail(), 9);
        rows.toggle(9);
        assert_eq!(rows.tail(), 0);
    }

    #[test]
    fn test_for_jsonl() {
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

        assert_eq!(rows.tail(), 11);
        rows.toggle(0);
        assert_eq!(rows.tail(), 11);
        rows.toggle(8);
        assert_eq!(rows.tail(), 8);
    }
}

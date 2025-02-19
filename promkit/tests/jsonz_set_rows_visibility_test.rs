#[cfg(test)]
mod set_nodes_visibility {
    use serde_json::Deserializer;

    use promkit::jsonz::*;

    #[test]
    fn test() {
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

        rows.set_rows_visibility(true);
        for row in &rows {
            match &row.v {
                Value::Open { collapsed, .. } | Value::Close { collapsed, .. } => {
                    assert!(collapsed, "Node should be collapsed");
                }
                _ => {}
            }
        }

        rows.set_rows_visibility(false);
        for row in &rows {
            match &row.v {
                Value::Open { collapsed, .. } | Value::Close { collapsed, .. } => {
                    assert!(!collapsed, "Node should be expanded");
                }
                _ => {}
            }
        }
    }
}

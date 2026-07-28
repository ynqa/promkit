use std::str::FromStr;

use serde_json::Deserializer;

use promkit_widgets::json::jsonz::*;

#[test]
fn remains_on_the_first_row_after_toggle() {
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
    assert_eq!(rows.head(), 0);
    rows.toggle(9);
    assert_eq!(rows.head(), 0);
}

#[test]
fn remains_on_the_first_json_lines_document() {
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

    assert_eq!(rows.head(), 0);
    rows.toggle(0);
    assert_eq!(rows.head(), 0);
    rows.toggle(8);
    assert_eq!(rows.head(), 0);
}

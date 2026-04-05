use std::str::FromStr;

use promkit_widgets::json::jsonz::{create_rows, PrettyRender};

#[test]
fn render_pretty() {
    let expected = r#"
{
    "array": [
        {
            "key": "value"
        },
        [
            1,
            2,
            3
        ],
        {
            "nested": true
        }
    ],
    "object": {
        "array": [
            1,
            2,
            3
        ],
        "nested": {
            "value": "test"
        }
    }
}"#
    .trim();

    let rows = create_rows([&serde_json::Value::from_str(expected).unwrap()]);
    assert_eq!(rows.render_pretty(4), expected);
}

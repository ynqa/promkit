use std::str::FromStr;

use promkit_widgets::json::jsonz::*;

#[test]
fn skips_collapsed_container_contents() {
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

    let mut rows = create_rows([&input]);
    rows.toggle(1);
    rows.toggle(4);

    assert_eq!(rows.down(0), 1);
    assert_eq!(rows.down(1), 4);
    assert_eq!(rows.down(4), 9);
    assert_eq!(rows.down(9), 9);
}

#[test]
fn stays_on_the_last_collapsed_container() {
    let input = serde_json::Value::from_str(
        r#"
            [
                1,
                2,
                3
            ]
        "#,
    )
    .unwrap();

    let mut rows = create_rows([&input]);
    rows.toggle(0);

    assert_eq!(rows.down(0), 0);
}

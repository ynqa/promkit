use std::str::FromStr;

use promkit_widgets::jsonstream::jsonz::*;

#[test]
fn test_basic_extract() {
    let input = serde_json::Value::from_str(
        r#"
            {
                "a": 1,
                "b": 2,
                "c": 3
            }
        "#,
    )
    .unwrap();

    let rows = create_rows([&input]);

    let extracted = rows.extract(0, 2);
    assert_eq!(extracted.len(), 2);
    assert_eq!(
        extracted[0],
        Row {
            depth: 0,
            k: None,
            v: Value::Open {
                typ: ContainerType::Object,
                collapsed: false,
                close_index: 4,
            },
        }
    );
    assert_eq!(
        extracted[1],
        Row {
            depth: 1,
            k: Some("a".to_string()),
            v: Value::Number(serde_json::Number::from(1)),
        }
    );

    let extracted = rows.extract(2, 2);
    assert_eq!(extracted.len(), 2);
    assert_eq!(
        extracted[0],
        Row {
            depth: 1,
            k: Some("b".to_string()),
            v: Value::Number(serde_json::Number::from(2)),
        }
    );
    assert_eq!(
        extracted[1],
        Row {
            depth: 1,
            k: Some("c".to_string()),
            v: Value::Number(serde_json::Number::from(3)),
        }
    );
}

#[test]
fn test_extract_with_collapsed_open() {
    let input = serde_json::Value::from_str(
        r#"
            {
                "object": {
                    "a": 1,
                    "b": 2
                },
                "after": "value"
            }
        "#,
    )
    .unwrap();

    let mut rows = create_rows([&input]);

    // {
    //   "object": {...},
    //   "after": "value"
    // }
    rows.toggle(1);

    let extracted = rows.extract(1, 3);
    assert_eq!(extracted.len(), 3);
    assert_eq!(
        extracted[0],
        Row {
            depth: 1,
            k: Some("object".to_string()),
            v: Value::Open {
                typ: ContainerType::Object,
                collapsed: true,
                close_index: 4,
            },
        }
    );
    assert_eq!(
        extracted[1],
        Row {
            depth: 1,
            k: Some("after".to_string()),
            v: Value::String("value".to_string()),
        }
    );
    assert_eq!(
        extracted[2],
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
fn test_extract_nested_structure() {
    let input = serde_json::Value::from_str(
        r#"
            {
                "array": [
                    {
                        "a": 1
                    },
                    {
                        "b": 2
                    }
                ]
            }
        "#,
    )
    .unwrap();

    let rows = create_rows([&input]);

    let extracted = rows.extract(2, 3);
    assert_eq!(extracted.len(), 3);
    assert_eq!(
        extracted[0],
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
        extracted[1],
        Row {
            depth: 3,
            k: Some("a".to_string()),
            v: Value::Number(serde_json::Number::from(1)),
        }
    );
    assert_eq!(
        extracted[2],
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
}

#[test]
fn test_extract_boundary_cases() {
    let input = serde_json::Value::from_str(
        r#"
            {
                "a": 1,
                "b": 2
            }
        "#,
    )
    .unwrap();

    let rows = create_rows([&input]);

    let extracted = rows.extract(0, 10);
    assert_eq!(extracted.len(), 4);

    let extracted = rows.extract(3, 2);
    assert_eq!(extracted.len(), 1);

    let extracted = rows.extract(10, 1);
    assert_eq!(extracted.len(), 0);
}

#[test]
fn test_extract_complex_nested_collapsed() {
    let input = serde_json::Value::from_str(
        r#"
            {
                "obj1": {
                    "arr1": [
                        {
                            "a": 1,
                            "obj2": {
                                "b": 2,
                                "c": 3
                            }
                        }
                    ],
                    "arr2": [
                        {
                            "d": 4,
                            "obj3": {
                                "e": 5,
                                "f": 6
                            }
                        }
                    ]
                },
                "after": "value"
            }
        "#,
    )
    .unwrap();

    let mut rows = create_rows([&input]);

    // {
    //   "obj1": {
    //     "arr1": [
    //       {...}
    //     ],
    //     "arr2": [
    //       {
    //         "d": 4,
    //         "obj3": {...}
    //       }
    //     ]
    //   },
    //   "after": "value"
    // }
    rows.toggle(3);
    rows.toggle(14);

    let extracted = rows.extract(2, 6);
    assert_eq!(extracted.len(), 6);

    assert_eq!(
        extracted[0],
        Row {
            depth: 2,
            k: Some("arr1".to_string()),
            v: Value::Open {
                typ: ContainerType::Array,
                collapsed: false,
                close_index: 10,
            },
        }
    );
    assert_eq!(
        extracted[5],
        Row {
            depth: 4,
            k: Some("d".to_string()),
            v: Value::Number(serde_json::Number::from(4)),
        }
    );
}

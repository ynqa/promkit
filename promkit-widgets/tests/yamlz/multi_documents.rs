use promkit_widgets::yaml::yamlz::*;

fn number(raw: &str) -> serde_yaml::Number {
    match serde_yaml::from_str::<serde_yaml::Value>(raw).unwrap() {
        serde_yaml::Value::Number(n) => n,
        _ => panic!("not a number: {raw}"),
    }
}

#[test]
fn creates_rows_for_all_document_shapes() {
    let input = r#"
---
null_value: null
bool_true: true
bool_false: false
int_value: 42
float_value: 3.14
string_plain: hello
string_quoted: "line1\nline2"
tagged_scalar: !MyTag tagged
empty_map: {}
empty_seq: []
seq_scalars:
  - a
  - 1
  - false
nested_map:
  child_key: child_value
nested_seq:
  - name: alice
    age: 20
  - [1, 2, 3]
tagged_map: !MapTag { x: 1, y: 2 }
tagged_seq: !SeqTag [a, b]
complex_keys:
  "plain.key": value1
  10: value2
  true: value3
  null: value4
  ? { a: 1, b: 2 }
  : complex_key_value

---
- !ItemTag first
- { k: v }
- [x, y]
- {}
- []

---
!RootTag 999
"#;

    let inputs = input
        .split("\n---")
        .map(str::trim)
        .filter(|doc| !doc.is_empty())
        .map(|doc| serde_yaml::from_str::<serde_yaml::Value>(doc).unwrap())
        .collect::<Vec<_>>();

    let rows = create_rows(inputs.iter());

    let expected = vec![
        Row {
            depth: 0,
            key: None,
            node: YamlNode::Container(ContainerNode::Open {
                typ: ContainerType::Object,
                collapsed: false,
                close_index: 45,
            }),
            is_sequence_item: false,
        },
        Row {
            depth: 1,
            key: Some("null_value".to_string()),
            node: YamlNode::Null,
            is_sequence_item: false,
        },
        Row {
            depth: 1,
            key: Some("bool_true".to_string()),
            node: YamlNode::Boolean(true),
            is_sequence_item: false,
        },
        Row {
            depth: 1,
            key: Some("bool_false".to_string()),
            node: YamlNode::Boolean(false),
            is_sequence_item: false,
        },
        Row {
            depth: 1,
            key: Some("int_value".to_string()),
            node: YamlNode::Number(number("42")),
            is_sequence_item: false,
        },
        Row {
            depth: 1,
            key: Some("float_value".to_string()),
            node: YamlNode::Number(number("3.14")),
            is_sequence_item: false,
        },
        Row {
            depth: 1,
            key: Some("string_plain".to_string()),
            node: YamlNode::String("hello".to_string()),
            is_sequence_item: false,
        },
        Row {
            depth: 1,
            key: Some("string_quoted".to_string()),
            node: YamlNode::String("line1\nline2".to_string()),
            is_sequence_item: false,
        },
        Row {
            depth: 1,
            key: Some("tagged_scalar".to_string()),
            node: YamlNode::Tagged {
                tag: "!MyTag".to_string(),
                node: Box::new(YamlNode::String("tagged".to_string())),
            },
            is_sequence_item: false,
        },
        Row {
            depth: 1,
            key: Some("empty_map".to_string()),
            node: YamlNode::Container(ContainerNode::Empty {
                typ: ContainerType::Object,
            }),
            is_sequence_item: false,
        },
        Row {
            depth: 1,
            key: Some("empty_seq".to_string()),
            node: YamlNode::Container(ContainerNode::Empty {
                typ: ContainerType::Array,
            }),
            is_sequence_item: false,
        },
        Row {
            depth: 1,
            key: Some("seq_scalars".to_string()),
            node: YamlNode::Container(ContainerNode::Open {
                typ: ContainerType::Array,
                collapsed: false,
                close_index: 15,
            }),
            is_sequence_item: false,
        },
        Row {
            depth: 2,
            key: None,
            node: YamlNode::String("a".to_string()),
            is_sequence_item: true,
        },
        Row {
            depth: 2,
            key: None,
            node: YamlNode::Number(number("1")),
            is_sequence_item: true,
        },
        Row {
            depth: 2,
            key: None,
            node: YamlNode::Boolean(false),
            is_sequence_item: true,
        },
        Row {
            depth: 1,
            key: None,
            node: YamlNode::Container(ContainerNode::Close {
                typ: ContainerType::Array,
                collapsed: false,
                open_index: 11,
            }),
            is_sequence_item: false,
        },
        Row {
            depth: 1,
            key: Some("nested_map".to_string()),
            node: YamlNode::Container(ContainerNode::Open {
                typ: ContainerType::Object,
                collapsed: false,
                close_index: 18,
            }),
            is_sequence_item: false,
        },
        Row {
            depth: 2,
            key: Some("child_key".to_string()),
            node: YamlNode::String("child_value".to_string()),
            is_sequence_item: false,
        },
        Row {
            depth: 1,
            key: None,
            node: YamlNode::Container(ContainerNode::Close {
                typ: ContainerType::Object,
                collapsed: false,
                open_index: 16,
            }),
            is_sequence_item: false,
        },
        Row {
            depth: 1,
            key: Some("nested_seq".to_string()),
            node: YamlNode::Container(ContainerNode::Open {
                typ: ContainerType::Array,
                collapsed: false,
                close_index: 29,
            }),
            is_sequence_item: false,
        },
        Row {
            depth: 2,
            key: None,
            node: YamlNode::Container(ContainerNode::Open {
                typ: ContainerType::Object,
                collapsed: false,
                close_index: 23,
            }),
            is_sequence_item: true,
        },
        Row {
            depth: 3,
            key: Some("name".to_string()),
            node: YamlNode::String("alice".to_string()),
            is_sequence_item: false,
        },
        Row {
            depth: 3,
            key: Some("age".to_string()),
            node: YamlNode::Number(number("20")),
            is_sequence_item: false,
        },
        Row {
            depth: 2,
            key: None,
            node: YamlNode::Container(ContainerNode::Close {
                typ: ContainerType::Object,
                collapsed: false,
                open_index: 20,
            }),
            is_sequence_item: false,
        },
        Row {
            depth: 2,
            key: None,
            node: YamlNode::Container(ContainerNode::Open {
                typ: ContainerType::Array,
                collapsed: false,
                close_index: 28,
            }),
            is_sequence_item: true,
        },
        Row {
            depth: 3,
            key: None,
            node: YamlNode::Number(number("1")),
            is_sequence_item: true,
        },
        Row {
            depth: 3,
            key: None,
            node: YamlNode::Number(number("2")),
            is_sequence_item: true,
        },
        Row {
            depth: 3,
            key: None,
            node: YamlNode::Number(number("3")),
            is_sequence_item: true,
        },
        Row {
            depth: 2,
            key: None,
            node: YamlNode::Container(ContainerNode::Close {
                typ: ContainerType::Array,
                collapsed: false,
                open_index: 24,
            }),
            is_sequence_item: false,
        },
        Row {
            depth: 1,
            key: None,
            node: YamlNode::Container(ContainerNode::Close {
                typ: ContainerType::Array,
                collapsed: false,
                open_index: 19,
            }),
            is_sequence_item: false,
        },
        Row {
            depth: 1,
            key: Some("tagged_map".to_string()),
            node: YamlNode::Tagged {
                tag: "!MapTag".to_string(),
                node: Box::new(YamlNode::Container(ContainerNode::Open {
                    typ: ContainerType::Object,
                    collapsed: false,
                    close_index: 33,
                })),
            },
            is_sequence_item: false,
        },
        Row {
            depth: 2,
            key: Some("x".to_string()),
            node: YamlNode::Number(number("1")),
            is_sequence_item: false,
        },
        Row {
            depth: 2,
            key: Some("y".to_string()),
            node: YamlNode::Number(number("2")),
            is_sequence_item: false,
        },
        Row {
            depth: 1,
            key: None,
            node: YamlNode::Container(ContainerNode::Close {
                typ: ContainerType::Object,
                collapsed: false,
                open_index: 30,
            }),
            is_sequence_item: false,
        },
        Row {
            depth: 1,
            key: Some("tagged_seq".to_string()),
            node: YamlNode::Tagged {
                tag: "!SeqTag".to_string(),
                node: Box::new(YamlNode::Container(ContainerNode::Open {
                    typ: ContainerType::Array,
                    collapsed: false,
                    close_index: 37,
                })),
            },
            is_sequence_item: false,
        },
        Row {
            depth: 2,
            key: None,
            node: YamlNode::String("a".to_string()),
            is_sequence_item: true,
        },
        Row {
            depth: 2,
            key: None,
            node: YamlNode::String("b".to_string()),
            is_sequence_item: true,
        },
        Row {
            depth: 1,
            key: None,
            node: YamlNode::Container(ContainerNode::Close {
                typ: ContainerType::Array,
                collapsed: false,
                open_index: 34,
            }),
            is_sequence_item: false,
        },
        Row {
            depth: 1,
            key: Some("complex_keys".to_string()),
            node: YamlNode::Container(ContainerNode::Open {
                typ: ContainerType::Object,
                collapsed: false,
                close_index: 44,
            }),
            is_sequence_item: false,
        },
        Row {
            depth: 2,
            key: Some("plain.key".to_string()),
            node: YamlNode::String("value1".to_string()),
            is_sequence_item: false,
        },
        Row {
            depth: 2,
            key: Some("10".to_string()),
            node: YamlNode::String("value2".to_string()),
            is_sequence_item: false,
        },
        Row {
            depth: 2,
            key: Some("true".to_string()),
            node: YamlNode::String("value3".to_string()),
            is_sequence_item: false,
        },
        Row {
            depth: 2,
            key: Some("null".to_string()),
            node: YamlNode::String("value4".to_string()),
            is_sequence_item: false,
        },
        Row {
            depth: 2,
            key: Some("a: 1 b: 2".to_string()),
            node: YamlNode::String("complex_key_value".to_string()),
            is_sequence_item: false,
        },
        Row {
            depth: 1,
            key: None,
            node: YamlNode::Container(ContainerNode::Close {
                typ: ContainerType::Object,
                collapsed: false,
                open_index: 38,
            }),
            is_sequence_item: false,
        },
        Row {
            depth: 0,
            key: None,
            node: YamlNode::Container(ContainerNode::Close {
                typ: ContainerType::Object,
                collapsed: false,
                open_index: 0,
            }),
            is_sequence_item: false,
        },
        Row {
            depth: 0,
            key: None,
            node: YamlNode::DocumentSeparator,
            is_sequence_item: false,
        },
        Row {
            depth: 0,
            key: None,
            node: YamlNode::Container(ContainerNode::Open {
                typ: ContainerType::Array,
                collapsed: false,
                close_index: 58,
            }),
            is_sequence_item: false,
        },
        Row {
            depth: 1,
            key: None,
            node: YamlNode::Tagged {
                tag: "!ItemTag".to_string(),
                node: Box::new(YamlNode::String("first".to_string())),
            },
            is_sequence_item: true,
        },
        Row {
            depth: 1,
            key: None,
            node: YamlNode::Container(ContainerNode::Open {
                typ: ContainerType::Object,
                collapsed: false,
                close_index: 51,
            }),
            is_sequence_item: true,
        },
        Row {
            depth: 2,
            key: Some("k".to_string()),
            node: YamlNode::String("v".to_string()),
            is_sequence_item: false,
        },
        Row {
            depth: 1,
            key: None,
            node: YamlNode::Container(ContainerNode::Close {
                typ: ContainerType::Object,
                collapsed: false,
                open_index: 49,
            }),
            is_sequence_item: false,
        },
        Row {
            depth: 1,
            key: None,
            node: YamlNode::Container(ContainerNode::Open {
                typ: ContainerType::Array,
                collapsed: false,
                close_index: 55,
            }),
            is_sequence_item: true,
        },
        Row {
            depth: 2,
            key: None,
            node: YamlNode::String("x".to_string()),
            is_sequence_item: true,
        },
        Row {
            depth: 2,
            key: None,
            node: YamlNode::String("y".to_string()),
            is_sequence_item: true,
        },
        Row {
            depth: 1,
            key: None,
            node: YamlNode::Container(ContainerNode::Close {
                typ: ContainerType::Array,
                collapsed: false,
                open_index: 52,
            }),
            is_sequence_item: false,
        },
        Row {
            depth: 1,
            key: None,
            node: YamlNode::Container(ContainerNode::Empty {
                typ: ContainerType::Object,
            }),
            is_sequence_item: true,
        },
        Row {
            depth: 1,
            key: None,
            node: YamlNode::Container(ContainerNode::Empty {
                typ: ContainerType::Array,
            }),
            is_sequence_item: true,
        },
        Row {
            depth: 0,
            key: None,
            node: YamlNode::Container(ContainerNode::Close {
                typ: ContainerType::Array,
                collapsed: false,
                open_index: 47,
            }),
            is_sequence_item: false,
        },
        Row {
            depth: 0,
            key: None,
            node: YamlNode::DocumentSeparator,
            is_sequence_item: false,
        },
        Row {
            depth: 0,
            key: None,
            node: YamlNode::Tagged {
                tag: "!RootTag".to_string(),
                node: Box::new(YamlNode::Number(number("999"))),
            },
            is_sequence_item: false,
        },
    ];

    assert_eq!(rows.len(), expected.len());
    for (index, (actual, exp)) in rows.iter().zip(expected.iter()).enumerate() {
        assert_eq!(actual, exp, "row mismatch at index {index}");
    }
}

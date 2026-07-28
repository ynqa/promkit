use promkit_widgets::yaml::yamlz::{RowOperation, create_rows};

#[test]
fn tail_does_not_stop_on_a_merged_mapping_key() {
    let input = serde_yaml::from_str::<serde_yaml::Value>(
        r#"
- name: alice
"#,
    )
    .unwrap();
    let rows = create_rows([&input]);

    assert_eq!(rows.tail(), 1);
}

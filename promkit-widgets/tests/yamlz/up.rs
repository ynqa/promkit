use promkit_widgets::yaml::yamlz::{Row, RowOperation, create_rows};

fn sequence_mapping_rows() -> Vec<Row> {
    let input = serde_yaml::from_str::<serde_yaml::Value>(
        r#"
- name: alice
  age: 20
- tail
"#,
    )
    .unwrap();

    create_rows([&input])
}

#[test]
fn up_skips_a_mapping_key_merged_into_its_sequence_item() {
    let rows = sequence_mapping_rows();

    // Moving up from "age: 20" should select the combined
    // "- name: alice" line at row 1, not its key row at row 2.
    assert_eq!(rows.up(3), 1);
}

#[test]
fn up_from_the_first_root_mapping_key_does_not_select_the_invisible_root() {
    let input = serde_yaml::from_str::<serde_yaml::Value>("apiVersion: v1\nkind: Pod\n").unwrap();
    let rows = create_rows([&input]);

    assert_eq!(rows.head(), 1);
    assert_eq!(rows.up(1), 1);
}

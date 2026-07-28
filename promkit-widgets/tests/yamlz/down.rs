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
fn down_skips_a_mapping_key_merged_into_its_sequence_item() {
    let rows = sequence_mapping_rows();

    // Row 1 and row 2 render together as "- name: alice", so moving down
    // from that line should select "age: 20" at row 3.
    assert_eq!(rows.down(1), 3);
}

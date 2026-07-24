use promkit_widgets::yaml::{
    Config,
    config::OverflowMode,
    yamlz::{Row, RowOperation, create_rows},
};

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

#[test]
fn up_skips_a_mapping_key_merged_into_its_sequence_item() {
    let rows = sequence_mapping_rows();

    // Moving up from "age: 20" should select the combined
    // "- name: alice" line at row 1, not its key row at row 2.
    assert_eq!(rows.up(3), 1);
}

#[test]
fn toggle_collapses_and_expands_a_combined_sequence_mapping_line() {
    let mut rows = sequence_mapping_rows();
    let config = Config {
        overflow_mode: OverflowMode::Truncate,
        ..Default::default()
    };

    rows.toggle(1);
    let collapsed = config
        .render_terminal_rows(&rows.extract(1, 2), 80)
        .into_iter()
        .map(|line| line.to_string())
        .collect::<Vec<_>>();
    assert_eq!(collapsed, vec!["- {…}", "- tail"]);

    rows.toggle(1);
    let expanded = config
        .render_terminal_rows(&rows.extract(1, 2), 80)
        .into_iter()
        .map(|line| line.to_string())
        .collect::<Vec<_>>();
    assert_eq!(expanded, vec!["- name: alice", "  age: 20"]);
}

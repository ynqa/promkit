use promkit_widgets::yaml::{
    Config, Document,
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

#[test]
fn toggle_from_a_merged_mapping_key_targets_its_sequence_item() {
    let mut rows = sequence_mapping_rows();

    assert_eq!(rows.toggle(2), 1);

    let rendered = Config {
        overflow_mode: OverflowMode::Truncate,
        ..Default::default()
    }
    .render_terminal_rows(&rows.extract(1, 1), 80)
    .into_iter()
    .map(|line| line.to_string())
    .collect::<Vec<_>>();
    assert_eq!(rendered, vec!["- {…}"]);
}

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

#[test]
fn document_starts_on_the_first_visible_root_mapping_key() {
    let input = serde_yaml::from_str::<serde_yaml::Value>("apiVersion: v1\nkind: Pod\n").unwrap();
    let document = Document::new([&input]);

    let extracted = document.extract_rows_from_current(1);

    assert_eq!(extracted[0].key.as_deref(), Some("apiVersion"));
}

#[test]
fn up_from_the_first_root_mapping_key_does_not_select_the_invisible_root() {
    let input = serde_yaml::from_str::<serde_yaml::Value>("apiVersion: v1\nkind: Pod\n").unwrap();
    let rows = create_rows([&input]);

    assert_eq!(rows.head(), 1);
    assert_eq!(rows.up(1), 1);
}

#[test]
fn a_collapsed_root_container_remains_navigable() {
    let input = serde_yaml::from_str::<serde_yaml::Value>("apiVersion: v1\nkind: Pod\n").unwrap();
    let mut rows = create_rows([&input]);

    rows.toggle(0);

    assert_eq!(rows.head(), 0);
}

#[test]
fn toggle_on_the_first_root_mapping_key_collapses_and_restores_the_root() {
    let input = serde_yaml::from_str::<serde_yaml::Value>("apiVersion: v1\nkind: Pod\n").unwrap();
    let mut document = Document::new([&input]);
    let config = Config {
        overflow_mode: OverflowMode::Truncate,
        ..Default::default()
    };

    document.toggle();
    let collapsed = config
        .render_terminal_rows(&document.extract_rows_from_current(1), 80)
        .into_iter()
        .map(|line| line.to_string())
        .collect::<Vec<_>>();
    assert_eq!(collapsed, vec!["{…}"]);

    document.toggle();
    let expanded = config
        .render_terminal_rows(&document.extract_rows_from_current(1), 80)
        .into_iter()
        .map(|line| line.to_string())
        .collect::<Vec<_>>();
    assert_eq!(expanded, vec!["apiVersion: v1"]);
}

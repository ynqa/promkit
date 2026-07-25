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
fn toggle_on_a_scalar_sequence_item_key_is_a_no_op() {
    let mut rows = sequence_mapping_rows();
    let config = Config {
        overflow_mode: OverflowMode::Truncate,
        ..Default::default()
    };

    assert_eq!(rows.toggle(1), 1);
    let rendered = config
        .render_terminal_rows(&rows.extract(1, 2), 80)
        .into_iter()
        .map(|line| line.to_string())
        .collect::<Vec<_>>();
    assert_eq!(rendered, vec!["- name: alice", "  age: 20"]);
}

#[test]
fn toggle_from_a_merged_scalar_key_is_a_no_op() {
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
    assert_eq!(rendered, vec!["- name: alice"]);
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
fn toggle_on_the_first_root_scalar_key_is_a_no_op() {
    let input = serde_yaml::from_str::<serde_yaml::Value>("apiVersion: v1\nkind: Pod\n").unwrap();
    let mut document = Document::new([&input]);
    let config = Config {
        overflow_mode: OverflowMode::Truncate,
        ..Default::default()
    };

    document.toggle();
    let rendered = config
        .render_terminal_rows(&document.extract_rows_from_current(1), 80)
        .into_iter()
        .map(|line| line.to_string())
        .collect::<Vec<_>>();
    assert_eq!(rendered, vec!["apiVersion: v1"]);
}

#[test]
fn toggle_on_a_nested_scalar_key_is_a_no_op() {
    let input =
        serde_yaml::from_str::<serde_yaml::Value>("metadata:\n  name: example\nkind: Pod\n")
            .unwrap();
    let mut document = Document::new([&input]);
    let config = Config {
        overflow_mode: OverflowMode::Truncate,
        ..Default::default()
    };

    assert!(document.down());
    document.toggle();

    let rendered = config
        .render_terminal_rows(&document.extract_rows_from_current(1), 80)
        .into_iter()
        .map(|line| line.to_string())
        .collect::<Vec<_>>();
    assert_eq!(rendered, vec!["  name: example"]);
}

#[test]
fn toggle_on_the_first_root_mapping_key_preserves_the_key() {
    let input =
        serde_yaml::from_str::<serde_yaml::Value>("metadata:\n  name: example\nkind: Pod\n")
            .unwrap();
    let mut document = Document::new([&input]);

    document.toggle();

    let rendered = Config {
        overflow_mode: OverflowMode::Truncate,
        ..Default::default()
    }
    .render_terminal_rows(&document.extract_rows_from_current(1), 80)
    .into_iter()
    .map(|line| line.to_string())
    .collect::<Vec<_>>();
    assert_eq!(rendered, vec!["metadata: {…}"]);

    document.toggle();
    let rendered = Config {
        overflow_mode: OverflowMode::Truncate,
        ..Default::default()
    }
    .render_terminal_rows(&document.extract_rows_from_current(2), 80)
    .into_iter()
    .map(|line| line.to_string())
    .collect::<Vec<_>>();
    assert_eq!(rendered, vec!["metadata: ", "  name: example"]);
}

#[test]
fn toggle_on_the_first_root_sequence_key_preserves_the_key() {
    let input =
        serde_yaml::from_str::<serde_yaml::Value>("command:\n  - etcd\nkind: Pod\n").unwrap();
    let mut document = Document::new([&input]);

    document.toggle();

    let rendered = Config {
        overflow_mode: OverflowMode::Truncate,
        ..Default::default()
    }
    .render_terminal_rows(&document.extract_rows_from_current(1), 80)
    .into_iter()
    .map(|line| line.to_string())
    .collect::<Vec<_>>();
    assert_eq!(rendered, vec!["command: […]"]);

    document.toggle();
    let rendered = Config {
        overflow_mode: OverflowMode::Truncate,
        ..Default::default()
    }
    .render_terminal_rows(&document.extract_rows_from_current(2), 80)
    .into_iter()
    .map(|line| line.to_string())
    .collect::<Vec<_>>();
    assert_eq!(rendered, vec!["command: ", "  - etcd"]);
}

#[test]
fn toggle_on_a_sequence_item_mapping_key_preserves_the_key() {
    let input = serde_yaml::from_str::<serde_yaml::Value>(
        "- metadata:\n    name: example\n  enabled: true\n",
    )
    .unwrap();
    let mut document = Document::new([&input]);

    document.toggle();

    let rendered = Config {
        overflow_mode: OverflowMode::Truncate,
        ..Default::default()
    }
    .render_terminal_rows(&document.extract_rows_from_current(1), 80)
    .into_iter()
    .map(|line| line.to_string())
    .collect::<Vec<_>>();
    assert_eq!(rendered, vec!["- metadata: {…}"]);

    document.toggle();
    let rendered = Config {
        overflow_mode: OverflowMode::Truncate,
        ..Default::default()
    }
    .render_terminal_rows(&document.extract_rows_from_current(2), 80)
    .into_iter()
    .map(|line| line.to_string())
    .collect::<Vec<_>>();
    assert_eq!(rendered, vec!["- metadata: ", "    name: example"]);
}

#[test]
fn toggle_on_a_sequence_item_sequence_key_preserves_the_key() {
    let input =
        serde_yaml::from_str::<serde_yaml::Value>("- command:\n    - etcd\n  enabled: true\n")
            .unwrap();
    let mut document = Document::new([&input]);

    document.toggle();

    let rendered = Config {
        overflow_mode: OverflowMode::Truncate,
        ..Default::default()
    }
    .render_terminal_rows(&document.extract_rows_from_current(1), 80)
    .into_iter()
    .map(|line| line.to_string())
    .collect::<Vec<_>>();
    assert_eq!(rendered, vec!["- command: […]"]);

    document.toggle();
    let rendered = Config {
        overflow_mode: OverflowMode::Truncate,
        ..Default::default()
    }
    .render_terminal_rows(&document.extract_rows_from_current(2), 80)
    .into_iter()
    .map(|line| line.to_string())
    .collect::<Vec<_>>();
    assert_eq!(rendered, vec!["- command: ", "    - etcd"]);
}

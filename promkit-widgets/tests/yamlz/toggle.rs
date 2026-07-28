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

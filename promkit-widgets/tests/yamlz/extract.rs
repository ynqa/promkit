use promkit_widgets::yaml::{
    Config,
    config::OverflowMode,
    yamlz::{RowOperation, create_rows},
};

#[test]
fn extracts_enough_rows_when_a_sequence_mapping_is_rendered_on_one_line() {
    let input = serde_yaml::from_str::<serde_yaml::Value>(
        r#"
- name: alice
  age: 20
- tail
"#,
    )
    .unwrap();
    let rows = create_rows([&input]);

    // The sequence mapping and its first key are separate internal rows, but
    // render together as "- name: alice". Extract enough internal rows to
    // produce the requested three terminal rows.
    let extracted = rows.extract(1, 3);
    let rendered = Config {
        overflow_mode: OverflowMode::Truncate,
        ..Default::default()
    }
    .render_terminal_rows(&extracted, 80)
    .into_iter()
    .map(|line| line.to_string())
    .collect::<Vec<_>>();

    assert_eq!(rendered, vec!["- name: alice", "  age: 20", "- tail"]);
}

use promkit_widgets::yaml::{
    Document,
    yamlz::{RowOperation, create_rows},
};

#[test]
fn document_starts_on_the_first_visible_root_mapping_key() {
    let input = serde_yaml::from_str::<serde_yaml::Value>("apiVersion: v1\nkind: Pod\n").unwrap();
    let document = Document::new([&input]);

    let extracted = document.extract_rows_from_current(1);

    assert_eq!(extracted[0].key.as_deref(), Some("apiVersion"));
}

#[test]
fn a_collapsed_root_container_remains_navigable() {
    let input = serde_yaml::from_str::<serde_yaml::Value>("apiVersion: v1\nkind: Pod\n").unwrap();
    let mut rows = create_rows([&input]);

    rows.toggle(0);

    assert_eq!(rows.head(), 0);
}

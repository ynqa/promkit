use std::{cell::Cell, io::Read};

use crate::structured::yaml::{
    deserializer,
    yamlz::{
        self, ContainerNode, ContainerType, IndexedRows, PathKeyKind, Row, RowOperation,
        TagAwareContainer, YamlNode, is_invisible_root_container,
        sequence_mapping_line_start_for_path,
    },
};
use crate::structured::{
    PathIndex, PathRow, ProjectionViewport, create_path_indices,
    path::{append_bracket, append_string_key},
    projection_viewport,
};

/// Represents a navigable YAML document, allowing for efficient row navigation and folding.
#[derive(Clone)]
pub struct Document {
    rows: Vec<Row>,
    path_key_kinds: Vec<PathKeyKind>,
    path_indices: Box<[PathIndex]>,
    position: usize,
    line_numbers: Vec<Option<usize>>,
    line_count: usize,
    viewport: Cell<ProjectionViewport>,
}

impl Document {
    pub fn new<'a, I: IntoIterator<Item = &'a serde_yaml::Value>>(iter: I) -> Self {
        Self::from_rows(yamlz::create_indexed_rows(iter))
    }

    /// Parses one or more YAML documents directly into a navigable document.
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(input: &str) -> Result<Self, serde_yaml::Error> {
        <Self as std::str::FromStr>::from_str(input)
    }

    /// Reads one or more YAML documents directly into a navigable document.
    pub fn from_reader<R: Read>(reader: R) -> Result<Self, serde_yaml::Error> {
        deserializer::from_reader(reader).map(Self::from_rows)
    }

    fn from_rows(indexed_rows: IndexedRows) -> Self {
        let IndexedRows {
            rows,
            path_key_kinds,
        } = indexed_rows;
        debug_assert_eq!(rows.len(), path_key_kinds.len());
        let path_indices = yaml_path_indices(&rows);
        let position = rows.head();
        let mut line_numbers = vec![None; rows.len()];
        let mut line_count = 0;

        if !rows.is_empty() {
            let mut index = position;
            loop {
                line_count += 1;
                line_numbers[index] = Some(line_count);

                let next = rows.down(index);
                if next == index {
                    break;
                }
                index = next;
            }
        }

        Self {
            rows,
            path_key_kinds,
            path_indices,
            position,
            line_numbers,
            line_count,
            viewport: Cell::default(),
        }
    }
}

impl std::str::FromStr for Document {
    type Err = serde_yaml::Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        deserializer::from_str(input).map(Self::from_rows)
    }
}

impl Document {
    /// Returns a reference to the underlying vector of rows.
    pub fn rows(&self) -> &[Row] {
        &self.rows
    }

    /// Extract rows from the current cursor position.
    pub fn extract_rows_from_current(&self, n: usize) -> Vec<Row> {
        self.rows.extract(self.position, n)
    }

    pub(super) fn project_viewport(&self, n: usize) -> (usize, usize) {
        projection_viewport(&self.rows, self.position, n, &self.viewport)
    }

    pub(super) fn extract_rows_from(&self, position: usize, n: usize) -> Vec<Row> {
        self.rows.extract(position, n)
    }

    /// Returns all currently visible rows from the beginning of the document.
    pub fn visible_rows(&self) -> Vec<Row> {
        self.rows.extract(self.rows.head(), usize::MAX)
    }

    /// Returns stable one-based line numbers for the currently rendered rows.
    pub(super) fn visible_line_numbers(&self) -> Vec<usize> {
        self.line_numbers_from(self.rows.head(), usize::MAX)
    }

    pub(super) fn line_numbers_from_viewport(&self, position: usize, n: usize) -> Vec<usize> {
        self.line_numbers_from(position, n)
    }

    /// Returns the number of rows in the fully expanded YAML rendering.
    pub(super) fn line_count(&self) -> usize {
        self.line_count
    }

    fn line_numbers_from(&self, mut index: usize, n: usize) -> Vec<usize> {
        if self.rows.is_empty() || n == 0 {
            return Vec::new();
        }

        let mut line_numbers = Vec::new();
        for _ in 0..n {
            let stable_line_number = self.line_numbers[index].unwrap_or_else(|| {
                self.line_numbers[index + 1..]
                    .iter()
                    .find_map(|number| *number)
                    .expect("a collapsed YAML root must have a numbered visible descendant")
            });
            line_numbers.push(stable_line_number);
            let next = self.rows.down(index);
            if next == index {
                break;
            }
            index = next;
        }
        line_numbers
    }

    /// Returns the selected row's index in the rendered visible row sequence.
    pub fn visible_position(&self) -> usize {
        visible_position(&self.rows, self.position)
    }

    /// Returns the zero-based document index and jq-style path of the selected row.
    pub fn selected_path(&self) -> Option<(usize, String)> {
        path_at_row(
            &self.rows,
            &self.path_key_kinds,
            &self.path_indices,
            self.position,
        )
    }

    /// Toggles the container value associated with the YAML key at the cursor.
    ///
    /// The displayed key determines the toggle target:
    ///
    /// - A mapping value collapses to `key: {…}`.
    /// - A sequence value collapses to `key: […]`.
    /// - A scalar value is unchanged.
    ///
    /// For a sequence item such as `- key: value`, the `- ` prefix is retained
    /// and the same value-based rules apply. Containers without an associated
    /// key are not used as fallback toggle targets.
    pub fn toggle(&mut self) {
        let index = self.rows.toggle(self.position);
        self.position = index;
    }

    /// Toggles the row identified by its underlying document index.
    pub fn toggle_at(&mut self, row_index: usize) {
        if row_index < self.rows.len() {
            self.position = self.rows.toggle(row_index);
        }
    }

    /// Resolves a rendered visible row number to its underlying document index.
    pub fn row_index_at_visible_position(&self, visible_position: usize) -> Option<usize> {
        row_index_at_visible_position(&self.rows, self.rows.head(), visible_position)
    }

    /// Resolves a rendered visible row offset from the current cursor to its document index.
    pub fn row_index_at_visible_offset_from_current(&self, visible_offset: usize) -> Option<usize> {
        row_index_at_visible_position(&self.rows, self.position, visible_offset)
    }

    /// Resolves a jq-style path in a zero-based document to its navigable underlying row index.
    ///
    /// Each YAML document in the stream increments the document index.
    ///
    /// Paths use dot notation for identifier keys and bracket notation for sequence indices,
    /// non-identifier strings, and non-string scalar keys.
    pub fn row_index_for_path(&self, document_index: usize, path: &str) -> Option<usize> {
        locate_path(&self.rows, &self.path_key_kinds, document_index, path)
            .map(|located| navigable_row(&self.rows, located.row_index))
    }

    /// Moves the cursor to the value at a jq-style path in a zero-based document.
    ///
    /// Each YAML document in the stream increments the document index.
    ///
    /// Folded ancestors are expanded while unrelated folding state is preserved. Mapping keys
    /// that cannot be represented by a jq-style path are not addressable.
    pub fn move_to_path(&mut self, document_index: usize, path: &str) -> bool {
        let Some(located) = locate_path(&self.rows, &self.path_key_kinds, document_index, path)
        else {
            return false;
        };
        for open_index in located.ancestors {
            if matches!(
                TagAwareContainer::get(&self.rows[open_index].node),
                Some(ContainerNode::Open {
                    collapsed: true,
                    ..
                })
            ) {
                self.rows.toggle(open_index);
            }
        }
        self.position = navigable_row(&self.rows, located.row_index);
        true
    }

    pub(super) fn row_index_at_viewport_position(&self, visible_position: usize) -> Option<usize> {
        let viewport = self.viewport.get();
        viewport
            .initialized
            .then(|| row_index_at_visible_position(&self.rows, viewport.start, visible_position))
            .flatten()
    }

    /// Sets the visibility of all rows.
    pub fn set_nodes_visibility(&mut self, collapsed: bool) {
        self.rows.set_rows_visibility(collapsed);
        self.position = self.rows.head();
    }

    /// Moves the cursor backward through rows.
    pub fn up(&mut self) -> bool {
        let index = self.rows.up(self.position);
        let ret = index != self.position;
        self.position = index;
        ret
    }

    /// Moves the cursor to the head position.
    pub fn head(&mut self) -> bool {
        self.position = self.rows.head();
        true
    }

    /// Moves the cursor forward through rows.
    pub fn down(&mut self) -> bool {
        let index = self.rows.down(self.position);
        let ret = index != self.position;
        self.position = index;
        ret
    }

    /// Moves the cursor to the last position.
    pub fn tail(&mut self) -> bool {
        self.position = self.rows.tail();
        true
    }
}

struct PathFrame {
    path: Option<String>,
    typ: ContainerType,
    next_index: usize,
    open_index: usize,
}

struct LocatedRow {
    row_index: usize,
    ancestors: Vec<usize>,
}

fn locate_path(
    rows: &[Row],
    path_key_kinds: &[PathKeyKind],
    document_index: usize,
    target: &str,
) -> Option<LocatedRow> {
    let mut stack: Vec<PathFrame> = Vec::new();
    let mut current_document_index = None;
    let mut next_document_index = 0;

    for (row_index, row) in rows.iter().enumerate() {
        if matches!(row.node, YamlNode::DocumentSeparator) {
            stack.clear();
            continue;
        }
        if matches!(
            TagAwareContainer::get(&row.node),
            Some(ContainerNode::Close { .. })
        ) {
            stack.truncate(row.depth);
            continue;
        }
        if row.depth == 0 {
            if next_document_index > document_index {
                return None;
            }
            current_document_index = Some(next_document_index);
            next_document_index += 1;
            stack.clear();
        }
        if current_document_index != Some(document_index) {
            continue;
        }
        stack.truncate(row.depth);

        let path = if row.depth == 0 {
            Some(".".to_owned())
        } else {
            let parent = stack.get_mut(row.depth - 1)?;
            match parent.typ {
                ContainerType::Object => parent.path.as_ref().and_then(|parent_path| {
                    mapping_path(parent_path, row.key.as_deref()?, path_key_kinds[row_index])
                }),
                ContainerType::Array => {
                    let path = parent.path.as_ref().map(|parent_path| {
                        append_bracket(parent_path, &parent.next_index.to_string())
                    });
                    parent.next_index += 1;
                    path
                }
            }
        };

        if path.as_deref() == Some(target) {
            return Some(LocatedRow {
                row_index,
                ancestors: stack.iter().map(|frame| frame.open_index).collect(),
            });
        }

        if let Some(ContainerNode::Open { typ, .. }) = TagAwareContainer::get(&row.node) {
            stack.push(PathFrame {
                path,
                typ: typ.clone(),
                next_index: 0,
                open_index: row_index,
            });
        }
    }

    None
}

fn yaml_path_indices(rows: &[Row]) -> Box<[PathIndex]> {
    create_path_indices(rows.iter().map(|row| {
        if matches!(row.node, YamlNode::DocumentSeparator) {
            return PathRow::Separator;
        }
        match TagAwareContainer::get(&row.node) {
            Some(ContainerNode::Close { .. }) => PathRow::Close { depth: row.depth },
            Some(ContainerNode::Open { typ, .. }) => PathRow::Value {
                depth: row.depth,
                open_type: Some(typ.clone()),
            },
            _ => PathRow::Value {
                depth: row.depth,
                open_type: None,
            },
        }
    }))
}

fn path_at_row(
    rows: &[Row],
    path_key_kinds: &[PathKeyKind],
    path_indices: &[PathIndex],
    target_row_index: usize,
) -> Option<(usize, String)> {
    let target = rows.get(target_row_index)?;
    if matches!(target.node, YamlNode::DocumentSeparator) {
        return None;
    }
    let target_row_index = match TagAwareContainer::get(&target.node) {
        Some(ContainerNode::Close { open_index, .. }) => *open_index,
        _ => target_row_index,
    };
    let mut chain = vec![target_row_index];
    while rows[*chain.last()?].depth > 0 {
        chain.push(path_indices[*chain.last()?].parent()?);
    }

    let root_index = *chain.last()?;
    let document_index = path_indices[root_index].document_index()?;
    let mut path = ".".to_owned();
    for &row_index in chain.iter().rev().skip(1) {
        let index = path_indices[row_index];
        let parent_index = index.parent()?;
        let Some(ContainerNode::Open { typ, .. }) =
            TagAwareContainer::get(&rows[parent_index].node)
        else {
            return None;
        };
        path = match typ {
            ContainerType::Object => mapping_path(
                &path,
                rows[row_index].key.as_deref()?,
                path_key_kinds[row_index],
            )?,
            ContainerType::Array => append_bracket(&path, &index.array_index()?.to_string()),
        };
    }
    Some((document_index, path))
}

fn mapping_path(parent: &str, key: &str, kind: PathKeyKind) -> Option<String> {
    match kind {
        PathKeyKind::String => Some(append_string_key(parent, key)),
        PathKeyKind::Number | PathKeyKind::Bool | PathKeyKind::Null => {
            Some(append_bracket(parent, key))
        }
        PathKeyKind::None | PathKeyKind::Unsupported => None,
    }
}

fn navigable_row(rows: &Vec<Row>, row_index: usize) -> usize {
    if is_invisible_root_container(&rows[row_index]) {
        rows.down(row_index)
    } else {
        sequence_mapping_line_start_for_path(rows, row_index).unwrap_or(row_index)
    }
}

fn visible_position(rows: &Vec<Row>, target: usize) -> usize {
    let mut position = rows.head();
    let mut visible = 0;

    while position != target {
        let next = rows.down(position);
        if next == position {
            break;
        }
        position = next;
        visible += 1;
    }

    visible
}

fn row_index_at_visible_position(rows: &Vec<Row>, start: usize, target: usize) -> Option<usize> {
    if rows.is_empty() {
        return None;
    }

    let mut position = start;
    for _ in 0..target {
        let next = rows.down(position);
        if next == position {
            return None;
        }
        position = next;
    }
    Some(position)
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use serde::Deserialize;

    use super::Document;

    const INPUT: &str = r#"
---
name: alice
1: one
true: enabled
null: nothing
? [first, second]
: sequence-key
tagged: !Thing
  nested: value
items:
  - null
  - {}
  - &anchor
    aliased: true
  - *anchor
---
!Root
second: [1, 2]
"#;

    fn via_value(input: &str) -> Document {
        let values = serde_yaml::Deserializer::from_str(input)
            .map(serde_yaml::Value::deserialize)
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        Document::new(values.iter())
    }

    mod from_str {
        use super::*;

        #[test]
        fn matches_value_conversion() {
            let expected = via_value(INPUT);
            let actual = Document::from_str(INPUT).unwrap();

            assert_eq!(actual.rows(), expected.rows());
        }

        #[test]
        fn matches_value_conversion_for_scalar_forms() {
            for input in [
                "",
                "null\n",
                "~\n",
                ".nan\n",
                ".inf\n",
                "-.inf\n",
                "'quoted'\n",
                "|\n  multiline\n  text\n",
                "!!str 1\n",
                "!!int '1'\n",
                "---\n...\n---\n{}\n",
            ] {
                let expected = via_value(input);
                let actual = Document::from_str(input).unwrap();

                assert_eq!(actual.rows(), expected.rows(), "input: {input:?}");
            }
        }

        #[test]
        fn reports_invalid_yaml() {
            assert!(Document::from_str("key: [unterminated").is_err());
            assert!(Document::from_str("duplicate: one\nduplicate: two\n").is_err());
        }
    }

    mod from_reader {
        use super::*;

        #[test]
        fn matches_value_conversion() {
            let expected = via_value(INPUT);
            let actual = Document::from_reader(Cursor::new(INPUT.as_bytes())).unwrap();

            assert_eq!(actual.rows(), expected.rows());
            for (document_index, path) in [
                (0, ".name"),
                (0, "[1]"),
                (0, "[true]"),
                (0, "[null]"),
                (0, ".tagged.nested"),
                (0, ".items[2].aliased"),
                (1, ".second[1]"),
            ] {
                assert_eq!(
                    actual.row_index_for_path(document_index, path),
                    expected.row_index_for_path(document_index, path),
                    "path: {path}"
                );
            }
        }

        #[test]
        fn reports_invalid_yaml() {
            assert!(Document::from_reader(Cursor::new(b"key: {")).is_err());
        }
    }

    mod row_index_for_path {
        use super::*;

        #[test]
        fn distinguishes_scalar_mapping_keys_and_sequence_indices() {
            let document = Document::from_str(concat!(
                "name: Alice\n",
                "\"true\": string\n",
                "true: boolean\n",
                "1: number\n",
                "null: null-key\n",
                "items:\n",
                "  - first\n",
                "  - name: Bob\n",
            ))
            .unwrap();

            assert_eq!(document.row_index_for_path(0, ".name"), Some(1));
            assert_eq!(document.row_index_for_path(0, r#"["true"]"#), Some(2));
            assert_eq!(document.row_index_for_path(0, "[true]"), Some(3));
            assert_eq!(document.row_index_for_path(0, "[1]"), Some(4));
            assert_eq!(document.row_index_for_path(0, "[null]"), Some(5));
            assert_eq!(document.row_index_for_path(0, ".items[0]"), Some(7));
            assert_eq!(document.row_index_for_path(0, ".items[1]"), Some(8));
            assert_eq!(document.row_index_for_path(0, ".items[1].name"), Some(8));
            assert_eq!(document.row_index_for_path(0, ".missing"), None);
        }

        #[test]
        fn excludes_descendants_of_unrepresentable_mapping_keys() {
            let document = Document::from_str(concat!(
                "? [complex, key]\n",
                ": { hidden: value }\n",
                "visible: true\n",
            ))
            .unwrap();

            assert_eq!(document.row_index_for_path(0, ".hidden"), None);
            assert!(document.row_index_for_path(0, ".visible").is_some());
        }

        #[test]
        fn distinguishes_yaml_documents() {
            let document = Document::from_str(concat!(
                "name: first\n",
                "---\n",
                "name: second\n",
                "second_only: true\n",
            ))
            .unwrap();

            let first = document.row_index_for_path(0, ".name").unwrap();
            let second = document.row_index_for_path(1, ".name").unwrap();
            assert_ne!(first, second);
            assert_eq!(document.row_index_for_path(1, "."), Some(second));
            assert_eq!(document.row_index_for_path(0, ".second_only"), None);
            assert!(document.row_index_for_path(1, ".second_only").is_some());
            assert_eq!(document.row_index_for_path(2, "."), None);
        }
    }

    mod move_to_path {
        use super::*;

        #[test]
        fn expands_ancestors_and_preserves_unrelated_folding() {
            let mut document = Document::from_str(concat!(
                "items:\n",
                "  - name:\n",
                "      nested: true\n",
                "other:\n",
                "  value: 1\n",
            ))
            .unwrap();
            document.toggle_at(1);
            document.toggle_at(8);

            assert!(document.move_to_path(0, ".items[0].name.nested"));
            assert_eq!(document.visible_position(), 2);
            assert_eq!(
                document.selected_path(),
                Some((0, ".items[0].name.nested".to_owned()))
            );
            assert!(
                !document
                    .visible_rows()
                    .iter()
                    .any(|row| row.key.as_deref() == Some("value"))
            );
            assert!(!document.move_to_path(0, ".missing"));
        }

        #[test]
        fn selects_a_yaml_document() {
            let mut document = Document::from_str(concat!(
                "first_only: true\n",
                "---\n",
                "second_only: true\n",
            ))
            .unwrap();

            assert!(document.move_to_path(1, ".second_only"));
            assert_eq!(
                document.selected_path(),
                Some((1, ".second_only".to_owned()))
            );
            assert!(!document.move_to_path(0, ".second_only"));
            assert!(!document.move_to_path(2, "."));

            assert!(document.move_to_path(1, "."));
            let root_position = document.visible_position();
            assert!(document.move_to_path(1, ".second_only"));
            assert_eq!(document.visible_position(), root_position);
        }
    }
}

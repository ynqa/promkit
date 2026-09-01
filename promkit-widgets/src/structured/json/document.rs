use std::{cell::Cell, io::Read};

use super::{
    deserializer,
    jsonz::{self, ContainerNode, ContainerType, JsonNode, Row, RowOperation},
};
use crate::structured::{
    ProjectionViewport,
    path::{append_bracket, append_string_key},
    projection_viewport,
};

/// Represents a navigable JSON document, allowing for efficient row navigation and folding.
#[derive(Clone)]
pub struct Document {
    rows: Vec<Row>,
    position: usize,
    viewport: Cell<ProjectionViewport>,
}

impl Document {
    pub fn new<'a, I: IntoIterator<Item = &'a serde_json::Value>>(iter: I) -> Self {
        Self::from_rows(jsonz::create_rows(iter))
    }

    /// Parses one or more JSON values directly into a navigable document.
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(input: &str) -> Result<Self, serde_json::Error> {
        <Self as std::str::FromStr>::from_str(input)
    }

    /// Reads one or more JSON values directly into a navigable document.
    pub fn from_reader<R: Read>(reader: R) -> Result<Self, serde_json::Error> {
        deserializer::from_reader(reader).map(Self::from_rows)
    }

    fn from_rows(rows: Vec<Row>) -> Self {
        Self {
            rows,
            position: 0,
            viewport: Cell::default(),
        }
    }
}

impl std::str::FromStr for Document {
    type Err = serde_json::Error;

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

    /// Returns stable one-based line numbers for the currently visible rows.
    pub(super) fn visible_line_numbers(&self) -> Vec<usize> {
        self.line_numbers_from(self.rows.head(), usize::MAX)
    }

    pub(super) fn line_numbers_from_viewport(&self, position: usize, n: usize) -> Vec<usize> {
        self.line_numbers_from(position, n)
    }

    /// Returns the number of rows in the fully expanded document.
    pub(super) fn line_count(&self) -> usize {
        self.rows.len()
    }

    fn line_numbers_from(&self, mut index: usize, n: usize) -> Vec<usize> {
        if self.rows.is_empty() || n == 0 {
            return Vec::new();
        }

        let mut line_numbers = Vec::new();
        for _ in 0..n {
            line_numbers.push(index + 1);
            let next = self.rows.down(index);
            if next == index {
                break;
            }
            index = next;
        }
        line_numbers
    }

    /// Returns the selected row's index in the visible row sequence.
    pub fn visible_position(&self) -> usize {
        visible_position(&self.rows, self.position)
    }

    /// Returns the zero-based document index and jq-style path of the selected row.
    pub fn selected_path(&self) -> Option<(usize, String)> {
        path_at_row(&self.rows, self.position)
    }

    /// Toggles the visibility of a node at the cursor's current position.
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

    /// Resolves a visible row number to its underlying document index.
    pub fn row_index_at_visible_position(&self, visible_position: usize) -> Option<usize> {
        row_index_at_visible_position(&self.rows, self.rows.head(), visible_position)
    }

    /// Resolves a visible row offset from the current cursor to its document index.
    pub fn row_index_at_visible_offset_from_current(&self, visible_offset: usize) -> Option<usize> {
        row_index_at_visible_position(&self.rows, self.position, visible_offset)
    }

    /// Resolves a jq-style path in a zero-based document to its underlying row index.
    ///
    /// Each top-level JSON value, including each JSON Lines value, increments the document index.
    ///
    /// Paths use dot notation for identifier keys and bracket notation for array indices and
    /// other string keys, for example `.items[0].name` and `["first name"]`.
    pub fn row_index_for_path(&self, document_index: usize, path: &str) -> Option<usize> {
        locate_path(&self.rows, document_index, path).map(|located| located.row_index)
    }

    /// Moves the cursor to the value at a jq-style path in a zero-based document.
    ///
    /// Each top-level JSON value, including each JSON Lines value, increments the document index.
    ///
    /// Folded ancestors are expanded while unrelated folding state is preserved.
    pub fn move_to_path(&mut self, document_index: usize, path: &str) -> bool {
        let Some(located) = locate_path(&self.rows, document_index, path) else {
            return false;
        };
        for open_index in located.ancestors {
            if matches!(
                self.rows[open_index].node,
                JsonNode::Container(ContainerNode::Open {
                    collapsed: true,
                    ..
                })
            ) {
                self.rows.toggle(open_index);
            }
        }
        self.position = located.row_index;
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
        self.position = 0;
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
    path: String,
    typ: ContainerType,
    next_index: usize,
    open_index: usize,
}

struct LocatedRow {
    row_index: usize,
    ancestors: Vec<usize>,
}

fn locate_path(rows: &[Row], document_index: usize, target: &str) -> Option<LocatedRow> {
    let mut stack: Vec<PathFrame> = Vec::new();
    let mut current_document_index = None;
    let mut next_document_index = 0;

    for (row_index, row) in rows.iter().enumerate() {
        if matches!(row.node, JsonNode::Container(ContainerNode::Close { .. })) {
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
            ".".to_owned()
        } else {
            let parent = stack.get_mut(row.depth - 1)?;
            match parent.typ {
                ContainerType::Object => append_string_key(&parent.path, row.key.as_deref()?),
                ContainerType::Array => {
                    let path = append_bracket(&parent.path, &parent.next_index.to_string());
                    parent.next_index += 1;
                    path
                }
            }
        };

        if path == target {
            return Some(LocatedRow {
                row_index,
                ancestors: stack.iter().map(|frame| frame.open_index).collect(),
            });
        }

        if let JsonNode::Container(ContainerNode::Open { typ, .. }) = &row.node {
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

fn path_at_row(rows: &[Row], target_row_index: usize) -> Option<(usize, String)> {
    let target_row_index = match &rows.get(target_row_index)?.node {
        JsonNode::Container(ContainerNode::Close { open_index, .. }) => *open_index,
        _ => target_row_index,
    };
    let mut stack: Vec<PathFrame> = Vec::new();
    let mut document_index = 0;

    for (row_index, row) in rows.iter().enumerate() {
        if matches!(row.node, JsonNode::Container(ContainerNode::Close { .. })) {
            stack.truncate(row.depth);
            continue;
        }
        if row.depth == 0 {
            if row_index > 0 {
                document_index += 1;
            }
            stack.clear();
        }
        stack.truncate(row.depth);

        let path = if row.depth == 0 {
            ".".to_owned()
        } else {
            let parent = stack.get_mut(row.depth - 1)?;
            match parent.typ {
                ContainerType::Object => append_string_key(&parent.path, row.key.as_deref()?),
                ContainerType::Array => {
                    let path = append_bracket(&parent.path, &parent.next_index.to_string());
                    parent.next_index += 1;
                    path
                }
            }
        };

        if row_index == target_row_index {
            return Some((document_index, path));
        }

        if let JsonNode::Container(ContainerNode::Open { typ, .. }) = &row.node {
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

    use super::Document;

    const INPUT: &str = r#"
{"name":"alice","active":true,"score":12.5,"items":[null,1,{"nested":"value"}]}
[true]
"tail"
"#;

    fn via_value(input: &str) -> Document {
        let values = serde_json::Deserializer::from_str(input)
            .into_iter::<serde_json::Value>()
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
        fn matches_value_conversion_for_edge_cases() {
            for input in [
                "",
                " \n\t",
                "null",
                "{}",
                "[]",
                "-0 1.25e3",
                r#""escaped\ntext" {"a":[],"b":{}}"#,
            ] {
                let expected = via_value(input);
                let actual = Document::from_str(input).unwrap();

                assert_eq!(actual.rows(), expected.rows(), "input: {input:?}");
            }
        }

        #[test]
        fn reports_invalid_json() {
            assert!(Document::from_str(r#"{"missing":"close""#).is_err());
        }
    }

    mod from_reader {
        use super::*;

        #[test]
        fn matches_value_conversion() {
            let expected = via_value(INPUT);
            let actual = Document::from_reader(Cursor::new(INPUT.as_bytes())).unwrap();

            assert_eq!(actual.rows(), expected.rows());
        }

        #[test]
        fn reports_invalid_json() {
            assert!(Document::from_reader(Cursor::new(b"[1,")).is_err());
        }
    }

    mod row_index_for_path {
        use super::*;

        #[test]
        fn resolves_nested_values_arrays_and_quoted_keys() {
            let document = Document::from_str(
                r#"{"items":[null,{"first name":true}],"true":false,"a\"b\n":0}"#,
            )
            .unwrap();

            assert_eq!(document.row_index_for_path(0, "."), Some(0));
            assert_eq!(document.row_index_for_path(0, ".items"), Some(1));
            assert_eq!(document.row_index_for_path(0, ".items[1]"), Some(3));
            assert_eq!(
                document.row_index_for_path(0, r#".items[1]["first name"]"#),
                Some(4)
            );
            assert_eq!(document.row_index_for_path(0, r#"["true"]"#), Some(7));
            assert_eq!(document.row_index_for_path(0, r#"["a\"b\n"]"#), Some(8));
            assert_eq!(document.row_index_for_path(0, ".missing"), None);
        }

        #[test]
        fn distinguishes_json_lines_documents() {
            let document = Document::from_str(concat!(
                "{\"name\":\"first\"}\n",
                "{\"name\":\"second\",\"second_only\":true}\n",
            ))
            .unwrap();

            let first = document.row_index_for_path(0, ".name").unwrap();
            let second = document.row_index_for_path(1, ".name").unwrap();
            assert_ne!(first, second);
            assert_eq!(document.row_index_for_path(0, ".second_only"), None);
            assert!(document.row_index_for_path(1, ".second_only").is_some());
            assert_eq!(document.row_index_for_path(2, "."), None);
        }
    }

    mod move_to_path {
        use super::*;

        #[test]
        fn expands_ancestors_and_selects_the_target() {
            let mut document =
                Document::from_str(r#"{"items":[{"nested":true}],"other":{"value":1}}"#).unwrap();
            document.toggle_at(1);
            document.toggle_at(6);

            assert!(document.move_to_path(0, ".items[0].nested"));
            assert_eq!(document.visible_position(), 3);
            assert_eq!(
                document.selected_path(),
                Some((0, ".items[0].nested".to_owned()))
            );
            assert_eq!(document.visible_rows().len(), 8);
            assert!(!document.move_to_path(0, ".missing"));
        }

        #[test]
        fn selects_a_json_lines_document() {
            let mut document =
                Document::from_str("{\"first_only\":true}\n{\"second_only\":true}\n").unwrap();

            assert!(document.move_to_path(1, ".second_only"));
            assert_eq!(
                document.selected_path(),
                Some((1, ".second_only".to_owned()))
            );
            assert!(!document.move_to_path(0, ".second_only"));
            assert!(!document.move_to_path(2, "."));
        }
    }
}

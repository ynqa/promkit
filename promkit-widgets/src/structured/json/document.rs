use std::{cell::Cell, io::Read};

use super::{
    deserializer,
    jsonz::{self, Row, RowOperation},
};
use crate::structured::{ProjectionViewport, projection_viewport};

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
}

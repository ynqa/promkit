use std::{cell::Cell, io::Read};

use crate::structured::yaml::{
    deserializer,
    yamlz::{self, Row, RowOperation},
};
use crate::structured::{ProjectionViewport, projection_viewport};

/// Represents a navigable YAML document, allowing for efficient row navigation and folding.
#[derive(Clone)]
pub struct Document {
    rows: Vec<Row>,
    position: usize,
    line_numbers: Vec<Option<usize>>,
    line_count: usize,
    viewport: Cell<ProjectionViewport>,
}

impl Document {
    pub fn new<'a, I: IntoIterator<Item = &'a serde_yaml::Value>>(iter: I) -> Self {
        Self::from_rows(yamlz::create_rows(iter))
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

    fn from_rows(rows: Vec<Row>) -> Self {
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

    #[test]
    fn from_str_matches_value_conversion() {
        let expected = via_value(INPUT);
        let actual = Document::from_str(INPUT).unwrap();

        assert_eq!(actual.rows(), expected.rows());
    }

    #[test]
    fn from_reader_matches_value_conversion() {
        let expected = via_value(INPUT);
        let actual = Document::from_reader(Cursor::new(INPUT.as_bytes())).unwrap();

        assert_eq!(actual.rows(), expected.rows());
    }

    #[test]
    fn direct_parsing_matches_value_conversion_for_scalar_forms() {
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
    fn direct_parsing_reports_invalid_yaml() {
        assert!(Document::from_str("key: [unterminated").is_err());
        assert!(Document::from_reader(Cursor::new(b"key: {")).is_err());
        assert!(Document::from_str("duplicate: one\nduplicate: two\n").is_err());
    }
}

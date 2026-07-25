use super::jsonz::{self, Row, RowOperation};

/// Represents a navigable JSON document, allowing for efficient row navigation and folding.
#[derive(Clone)]
pub struct Document {
    rows: Vec<Row>,
    position: usize,
}

impl Document {
    pub fn new<'a, I: IntoIterator<Item = &'a serde_json::Value>>(iter: I) -> Self {
        Self {
            rows: jsonz::create_rows(iter),
            position: 0,
        }
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

    /// Returns all currently visible rows from the beginning of the document.
    pub fn visible_rows(&self) -> Vec<Row> {
        self.rows.extract(self.rows.head(), usize::MAX)
    }

    /// Returns stable one-based line numbers for the currently visible rows.
    pub(super) fn visible_line_numbers(&self) -> Vec<usize> {
        self.line_numbers_from(self.rows.head(), usize::MAX)
    }

    /// Returns stable one-based line numbers for rows projected from the cursor.
    pub(super) fn line_numbers_from_current(&self, n: usize) -> Vec<usize> {
        self.line_numbers_from(self.position, n)
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

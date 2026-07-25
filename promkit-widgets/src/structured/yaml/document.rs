use crate::structured::yaml::yamlz::{self, Row, RowOperation};

/// Represents a navigable YAML document, allowing for efficient row navigation and folding.
#[derive(Clone)]
pub struct Document {
    rows: Vec<Row>,
    position: usize,
}

impl Document {
    pub fn new<'a, I: IntoIterator<Item = &'a serde_yaml::Value>>(iter: I) -> Self {
        let rows = yamlz::create_rows(iter);
        let position = rows.head();
        Self { rows, position }
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
        row_index_at_visible_position(&self.rows, visible_position)
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

fn row_index_at_visible_position(rows: &Vec<Row>, target: usize) -> Option<usize> {
    if rows.is_empty() {
        return None;
    }

    let mut position = rows.head();
    for _ in 0..target {
        let next = rows.down(position);
        if next == position {
            return None;
        }
        position = next;
    }
    Some(position)
}

use super::jsonz::{self, Row, RowOperation};

/// Represents a stream of JSON data, allowing for efficient navigation and manipulation.
#[derive(Clone)]
pub struct JsonStream {
    rows: Vec<Row>,
    position: usize,
}

impl JsonStream {
    pub fn new<'a, I: IntoIterator<Item = &'a serde_json::Value>>(iter: I) -> Self {
        Self {
            rows: jsonz::create_rows(iter),
            position: 0,
        }
    }
}

impl JsonStream {
    /// Returns a reference to the underlying vector of rows.
    pub fn rows(&self) -> &[Row] {
        &self.rows
    }

    /// Extracts a specified number of rows from the current position in JSON stream.
    pub fn extract_rows_from_current(&self, n: usize) -> Vec<Row> {
        self.rows.extract(self.position, n)
    }

    /// Toggles the visibility of a node at the cursor's current position.
    pub fn toggle(&mut self) {
        let index = self.rows.toggle(self.position);
        self.position = index;
    }

    /// Sets the visibility of all rows in JSON stream.
    pub fn set_nodes_visibility(&mut self, collapsed: bool) {
        self.rows.set_rows_visibility(collapsed);
        self.position = 0;
    }

    /// Moves the cursor backward through JSON stream.
    pub fn up(&mut self) -> bool {
        let index = self.rows.up(self.position);
        let ret = index != self.position;
        self.position = index;
        ret
    }

    /// Moves the cursor to the head position in JSON stream.
    pub fn head(&mut self) -> bool {
        self.position = self.rows.head();
        true
    }

    /// Moves the cursor forward through JSON stream.
    pub fn down(&mut self) -> bool {
        let index = self.rows.down(self.position);
        let ret = index != self.position;
        self.position = index;
        ret
    }

    /// Moves the cursor to the last position in JSON stream.
    pub fn tail(&mut self) -> bool {
        self.position = self.rows.tail();
        true
    }
}

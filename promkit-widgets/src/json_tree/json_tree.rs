use super::jsonz::{self, Row, RowOperation};

/// Represents a navigable JSON tree, allowing for efficient navigation and folding.
#[derive(Clone)]
pub struct JsonTree {
    rows: Vec<Row>,
    position: usize,
}

impl JsonTree {
    pub fn new<'a, I: IntoIterator<Item = &'a serde_json::Value>>(iter: I) -> Self {
        Self {
            rows: jsonz::create_rows(iter),
            position: 0,
        }
    }
}

impl JsonTree {
    /// Returns a reference to the underlying vector of rows.
    pub fn rows(&self) -> &[Row] {
        &self.rows
    }

    /// Extract rows from the current cursor position.
    pub fn extract_rows_from_current(&self, n: usize) -> Vec<Row> {
        self.rows.extract(self.position, n)
    }

    /// Toggles the visibility of a node at the cursor's current position.
    pub fn toggle(&mut self) {
        let index = self.rows.toggle(self.position);
        self.position = index;
    }

    /// Sets the visibility of all rows in the tree.
    pub fn set_nodes_visibility(&mut self, collapsed: bool) {
        self.rows.set_rows_visibility(collapsed);
        self.position = 0;
    }

    /// Moves the cursor backward through the tree.
    pub fn up(&mut self) -> bool {
        let index = self.rows.up(self.position);
        let ret = index != self.position;
        self.position = index;
        ret
    }

    /// Moves the cursor to the head position in the tree.
    pub fn head(&mut self) -> bool {
        self.position = self.rows.head();
        true
    }

    /// Moves the cursor forward through the tree.
    pub fn down(&mut self) -> bool {
        let index = self.rows.down(self.position);
        let ret = index != self.position;
        self.position = index;
        ret
    }

    /// Moves the cursor to the last position in the tree.
    pub fn tail(&mut self) -> bool {
        self.position = self.rows.tail();
        true
    }
}

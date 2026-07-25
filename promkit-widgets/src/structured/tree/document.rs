use crate::structured::RowOperation;

use super::{Row, path::PathAdapter, treez::Adapter};

/// Represents a navigable tree document, allowing for efficient row navigation and folding.
#[derive(Clone)]
pub struct Document {
    rows: Vec<Row>,
    position: usize,
}

impl Document {
    pub fn new(rows: Vec<Row>) -> Self {
        Self { rows, position: 0 }
    }

    pub fn from_path(path: &std::path::Path) -> anyhow::Result<Self> {
        Ok(Self::new(PathAdapter.create_rows(&path.to_path_buf())?))
    }
}

impl Document {
    /// Returns a reference to the underlying vector of rows.
    pub fn rows(&self) -> &[Row] {
        &self.rows
    }

    /// Returns the selected tree path represented by visible row labels.
    pub fn get(&self) -> Vec<String> {
        self.rows
            .get(self.position)
            .map(|row| row.path.clone())
            .unwrap_or_default()
    }

    /// Extract rows from the current cursor position.
    pub fn extract_rows_from_current(&self, n: usize) -> Vec<Row> {
        self.rows.extract(self.position, n)
    }

    /// Returns all currently visible rows from the beginning of the document.
    pub fn visible_rows(&self) -> Vec<Row> {
        self.rows.extract(self.rows.head(), usize::MAX)
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

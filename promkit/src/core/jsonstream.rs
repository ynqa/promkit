mod state;
pub use state::State;

use crate::jsonz::{self, Row, RowOperation};

/// Represents a stream of JSON data, allowing for efficient navigation and manipulation.
#[derive(Clone)]
pub struct JsonStream {
    rows: Vec<Row>,
    position: usize,
}

impl JsonStream {
    pub fn new<I: IntoIterator<Item = serde_json::Value>>(iter: I) -> Self {
        Self {
            rows: jsonz::create_rows(iter),
            position: 0,
        }
    }
}

impl JsonStream {
    pub fn extract_rows(&self, n: usize) -> Vec<Row> {
        self.rows.extract(self.position, n)
    }

    /// Toggles the visibility of a node at the cursor's current position.
    pub fn toggle(&mut self) {
        let index = self.rows.toggle(self.position);
        self.position = index;
    }

    /// Moves the cursor backward through JSON stream.
    pub fn backward(&mut self) -> bool {
        let index = self.rows.up(self.position);
        let ret = index != self.position;
        self.position = index;
        ret
    }

    /// Moves the cursor forward through JSON stream.
    pub fn forward(&mut self) -> bool {
        let index = self.rows.down(self.position);
        let ret = index != self.position;
        self.position = index;
        ret
    }
}

use thiserror::Error;

/// Errors that can occur when constructing a screen representation for testing.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum ScreenError {
    #[error("row {row} is out of bounds for screen height {rows}")]
    RowOutOfBounds { row: u16, rows: u16 },
}

pub type Result<T = ()> = std::result::Result<T, Error>;

use thiserror::Error;

use crate::serde_json;

/// Represents all possible errors that can occur in the application.
///
/// This enum encapsulates different types of errors by leveraging the `thiserror` crate
/// for easy error handling and propagation. The errors include IO errors, serde json errors,
/// interruption errors, and evaluator phase errors.
///
/// # Variants
///
/// - `IO`: Wraps `std::io::Error`. Occurs during input/output operations.
/// - `SerdeJson`: Wraps `serde_json::Error`.
/// Occurs during serialization or deserialization with serde_json.
/// - `Interrupted`: Represents an error where an operation was interrupted.
/// Contains a message describing the interruption.
/// - `EvaluatorError`: Represents errors that occur during the evaluator phase of the application.
/// Contains an error message.
#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    IO(#[from] std::io::Error),

    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),

    #[error("Type downcast attempt failed: {0}")]
    DowncastError(String),

    #[error("Operation interrupted: {0}")]
    Interrupted(String),
}

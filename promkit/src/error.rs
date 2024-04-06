pub type Result<T = ()> = std::result::Result<T, Error>;

use thiserror::Error;

use crate::serde_json;

/// Represents all possible errors that can occur within the application.
///
/// This enum categorizes various types of errors by leveraging the `thiserror` crate
/// for efficient error handling and propagation.
///
/// # Variants
///
/// - `IO`: Represents `std::io::Error`. Arises during input/output operations.
/// - `SerdeJson`: Represents `serde_json::Error`.
///   Arises during serialization or deserialization processes with serde_json.
/// - `Interrupted`: Indicates an operation was prematurely interrupted.
///   Contains a message detailing the interruption.
/// - `DowncastError`: Indicates a failure in type downcasting.
///   Contains a message detailing the failure.
#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    IO(#[from] std::io::Error),

    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),

    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),

    #[error("Type downcast attempt failed: {0}")]
    DowncastError(String),

    #[error("Operation interrupted: {0}")]
    Interrupted(String),
}

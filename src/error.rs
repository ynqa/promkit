pub type Result<T = ()> = std::result::Result<T, Error>;

use serde_json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),

    #[error("{0} interrupted")]
    Interrupted(String),

    #[error("serde json error: {0}")]
    SerdeJsonError(#[from] serde_json::Error),
}

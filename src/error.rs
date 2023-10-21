pub type Result<T = ()> = std::result::Result<T, Error>;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),

    #[error("{0} interrupted")]
    Interrupted(String),
}

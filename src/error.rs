pub type Result<T = ()> = std::result::Result<T, Error>;

use thiserror::Error;

use crate::crossterm::event::Event;

#[derive(Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),

    #[error("{event:?} interrupted")]
    Interrupted { event: Event },
}

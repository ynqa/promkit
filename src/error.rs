use std::io;

/// Result for `prompt`.
pub type Result<T> = std::result::Result<T, io::Error>;

use std::{error::Error, fmt, io};

#[derive(Debug)]
pub enum BodoError {
    IoError(io::Error),
    WatcherError(String),
}

impl fmt::Display for BodoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BodoError::IoError(err) => write!(f, "IO error: {}", err),
            BodoError::WatcherError(err) => write!(f, "Watcher error: {}", err),
        }
    }
}

impl Error for BodoError {}

impl From<io::Error> for BodoError {
    fn from(err: io::Error) -> Self {
        BodoError::IoError(err)
    }
}

impl From<notify::Error> for BodoError {
    fn from(err: notify::Error) -> Self {
        BodoError::WatcherError(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, BodoError>;

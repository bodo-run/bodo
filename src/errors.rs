use thiserror::Error;

#[derive(Error, Debug)]
pub enum BodoError {
    #[error("Generic error: {0}")]
    Generic(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Invalid configuration: {0}")]
    Config(String),
}

pub type Result<T> = std::result::Result<T, BodoError>;
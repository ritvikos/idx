extern crate thiserror;

use std::io;

use thiserror::Error;

/// Error type.
#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    Config(#[from] ConfigError),

    #[error("{0}")]
    Io(#[from] IoError),
}

/// Configuration error.
#[derive(Debug, Error, PartialEq)]
pub enum ConfigError {
    #[error("File I/O Error: {0}")]
    File(io::ErrorKind),

    #[error("Reader I/O Error: {0}")]
    Reader(io::ErrorKind),

    #[error("Tokenizer Error: {0}")]
    Tokenizer(String),

    #[error("Serialization Error: {0}")]
    Serialization(String),
}

/// I/O errors.
#[derive(Debug, Error, PartialEq)]
pub enum IoError {
    #[error("File Error: {0}")]
    File(io::ErrorKind),

    #[error("Reader Error: {0}")]
    Reader(io::ErrorKind),
}

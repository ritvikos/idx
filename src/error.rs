use std::{error::Error as CoreError, fmt, io};

#[derive(Debug)]
pub struct Error(Kind);

impl Error {
    /// Create a new `Error`.
    pub(crate) fn new(kind: Kind) -> Self {
        Self(kind)
    }

    /// Create a new `ConfigError`.
    pub(crate) fn config(kind: ConfigError) -> Self {
        Self::new(Kind::Config(kind))
    }
}

impl CoreError for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.0 {
            Kind::Config(reason) => write!(f, "{reason}"),
        }
    }
}

/// Defines the error type.
#[derive(Debug, PartialEq)]
pub enum Kind {
    Config(ConfigError),
}

// Configuration error.
#[derive(Debug, PartialEq)]
pub enum ConfigError {
    File(io::ErrorKind),
}

impl CoreError for ConfigError {}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            ConfigError::File(_) => write!(f, "I/O Error"),
        }
    }
}

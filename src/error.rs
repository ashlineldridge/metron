use std::fmt::Display;

/// `Error` enumerates all errors in the application.
#[derive(Debug)]
pub enum Error {
    /// Represents a CLI error.
    Cli(clap::Error),
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Cli(err) => Some(err),
            _ => None,
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Cli(err) => err.fmt(f),
        }
    }
}

impl From<clap::Error> for Error {
    fn from(inner: clap::Error) -> Self {
        Self::Cli(inner)
    }
}

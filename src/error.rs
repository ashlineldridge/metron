use std::{fmt::Display, io};

use snafu::prelude::*;
use tokio::sync::mpsc;

// This is an antipattern

#[derive(Debug, Snafu)]
pub enum MyError {}

/// `Error` enumerates all errors in the application.
#[derive(Debug)]
pub enum Error {
    /// Represents a CLI error.
    Cli(clap::Error),
    /// Represents an IO error.
    Io(io::Error),
    /// Represents a server error.
    ServerError(hyper::Error),
    /// Represents a signalling error.
    SignalError(mpsc::error::SendError<crate::load::Signal>),
    ///
    GenericError(String),
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Cli(err) => Some(err),
            Error::ServerError(err) => Some(err),
            Error::SignalError(err) => Some(err),
            _ => None,
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // TODO: let source = self.source(); source.fmt(f);
        match self {
            Error::Cli(err) => err.fmt(f),
            Error::Io(err) => err.fmt(f),
            Error::ServerError(err) => err.fmt(f),
            Error::SignalError(err) => err.fmt(f),
            Error::GenericError(s) => f.write_str(s.as_str()),
        }
    }
}

impl From<clap::Error> for Error {
    fn from(inner: clap::Error) -> Self {
        Self::Cli(inner)
    }
}

impl From<io::Error> for Error {
    fn from(inner: io::Error) -> Self {
        Self::Io(inner)
    }
}

impl From<hyper::Error> for Error {
    fn from(inner: hyper::Error) -> Self {
        Self::ServerError(inner)
    }
}

impl From<mpsc::error::SendError<crate::load::Signal>> for Error {
    fn from(inner: mpsc::error::SendError<crate::load::Signal>) -> Self {
        Self::SignalError(inner)
    }
}

// TODO: This is just awful
impl From<mpsc::error::SendError<crate::load::ClientResult>> for Error {
    fn from(_: mpsc::error::SendError<crate::load::ClientResult>) -> Self {
        todo!()
    }
}

// TODO: What's the best practice here?
impl From<hyper::http::uri::InvalidUri> for Error {
    fn from(inner: hyper::http::uri::InvalidUri) -> Self {
        Self::GenericError(inner.to_string())
    }
}

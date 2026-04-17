// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

use serde::{de, ser};
use std::{
    error::Error as StdError,
    fmt::{self, Debug, Display},
    io, result,
};

/// An error that occurred during YAML processing.
pub struct Error(Box<ErrorImpl>);

/// Alias for a `Result` with error type `serde_yml::Error`.
pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
enum ErrorImpl {
    Message(String),
    Io(io::Error),
}

/// The input location where an error occurred.
#[derive(Clone, Copy, Debug)]
pub struct Location {
    index: usize,
    line: usize,
    column: usize,
}

impl Location {
    /// Returns the byte index where the error occurred.
    pub fn index(&self) -> usize {
        self.index
    }

    /// Returns the line number where the error occurred.
    pub fn line(&self) -> usize {
        self.line
    }

    /// Returns the column number where the error occurred.
    pub fn column(&self) -> usize {
        self.column
    }
}

impl Error {
    /// Returns the I/O error that caused this, if any.
    pub fn io_error(&self) -> Option<&io::Error> {
        if let ErrorImpl::Io(err) = &*self.0 {
            Some(err)
        } else {
            None
        }
    }

    /// Returns the location where the error occurred.
    pub fn location(&self) -> Option<Location> {
        None
    }

    pub(crate) fn msg(s: impl Display) -> Self {
        Error(Box::new(ErrorImpl::Message(s.to_string())))
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &*self.0 {
            ErrorImpl::Message(msg) => f.write_str(msg),
            ErrorImpl::Io(err) => write!(f, "I/O error: {}", err),
        }
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error({:?})", self.to_string())
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match &*self.0 {
            ErrorImpl::Io(err) => Some(err),
            _ => None,
        }
    }
}

impl Clone for Error {
    fn clone(&self) -> Self {
        match &*self.0 {
            ErrorImpl::Message(msg) => {
                Error(Box::new(ErrorImpl::Message(msg.clone())))
            }
            ErrorImpl::Io(err) => Error(Box::new(
                ErrorImpl::Message(err.to_string()),
            )),
        }
    }
}

impl de::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error(Box::new(ErrorImpl::Message(msg.to_string())))
    }
}

impl ser::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error(Box::new(ErrorImpl::Message(msg.to_string())))
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error(Box::new(ErrorImpl::Io(err)))
    }
}

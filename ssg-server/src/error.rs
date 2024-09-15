use std::io;
use thiserror::Error;

/// Represents the different types of errors that can occur in the server.
///
/// This enum defines various errors that can be encountered during the server's operation,
/// such as I/O errors, invalid requests, file not found, and forbidden access.
#[derive(Error, Debug)]
pub enum ServerError {
    /// An I/O error occurred, such as failure to read from or write to a file or network socket.
    ///
    /// This variant wraps an `io::Error`, which provides detailed information about the I/O issue.
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    /// The request received by the server was invalid or malformed.
    ///
    /// This typically occurs when the HTTP request line is incorrectly formatted or missing
    /// necessary components such as the HTTP method, path, or version.
    #[error("Invalid request")]
    InvalidRequest,

    /// The requested file was not found on the server.
    ///
    /// This error is returned when the requested resource is not available at the specified path.
    #[error("File not found")]
    NotFound,

    /// Access to the requested resource is forbidden.
    ///
    /// This error is returned when the client does not have the necessary permissions
    /// to access the requested resource.
    #[error("Forbidden")]
    Forbidden,
}

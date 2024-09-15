use crate::error::ServerError;
use std::io::{BufRead, BufReader};
use std::net::TcpStream;

/// Represents an HTTP request, containing the HTTP method, the requested path, and the HTTP version.
pub struct Request {
    /// The HTTP method (e.g., GET, POST, PUT, etc.) used in the request.
    pub method: String,

    /// The path of the requested resource (e.g., `/index.html`).
    pub path: String,

    /// The HTTP version of the request (e.g., HTTP/1.1).
    pub version: String,
}

impl Request {
    /// Attempts to create a `Request` from the provided TCP stream by reading the first line.
    ///
    /// The first line of an HTTP request typically contains the HTTP method, path, and version,
    /// which are extracted and used to populate the `Request` struct.
    ///
    /// # Arguments
    ///
    /// * `stream` - A reference to the `TcpStream` from which the request will be read.
    ///
    /// # Returns
    ///
    /// * `Ok(Request)` - If the request is valid and successfully parsed.
    /// * `Err(ServerError)` - If the request is malformed or cannot be read.
    ///
    /// # Errors
    ///
    /// This function returns a `ServerError::InvalidRequest` error if the request is invalid or
    /// if the first line cannot be read or parsed correctly.
    pub fn from_stream(
        stream: &TcpStream,
    ) -> Result<Self, ServerError> {
        let buf_reader = BufReader::new(stream);
        let request_line = buf_reader
            .lines()
            .next()
            .ok_or(ServerError::InvalidRequest)??;
        let mut parts = request_line.split_whitespace();

        Ok(Request {
            method: parts
                .next()
                .ok_or(ServerError::InvalidRequest)?
                .to_string(),
            path: parts
                .next()
                .ok_or(ServerError::InvalidRequest)?
                .to_string(),
            version: parts
                .next()
                .ok_or(ServerError::InvalidRequest)?
                .to_string(),
        })
    }
}

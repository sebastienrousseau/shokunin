// Copyright © 2023 Shokunin (職人) Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::path::Path;

/// ## Function: `start` - Start a web server to serve the public directory.
///
/// This function takes a string for the server address and a string for
/// the document root, and then creates a TCP listener that listens at
/// the server address.
///
/// It then iterates over the incoming connections on the listener, and
/// handles each connection by passing it to the handle_connection
/// function.
///
/// # Arguments
///
/// * `server_address` - A string for the server address.
/// * `document_root`  - A string for the document root.
///
/// # Returns
///
/// * A Result indicating success or failure.
/// - Ok() if the web server started successfully.
/// - Err() if the web server could not be started.
///
/// # Errors
///
/// * If the server fails to bind to the address, it will return an
/// error.
/// * If the server fails to accept a connection, it will return an
/// error.
/// * If the server fails to read data from a connection, it will
/// return an error.
/// * If the server fails to write data to a connection, it will
/// return an error.
///
pub fn start(
    server_address: &str,
    document_root: &str,
) -> std::io::Result<()> {
    let listener = TcpListener::bind(server_address)?;
    println!("❯ Server is now running at http://{}", server_address);
    println!("  Done.\n");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                if let Err(e) = handle_connection(stream, document_root)
                {
                    eprintln!("Error handling connection: {}", e);
                }
            }
            Err(e) => {
                eprintln!("Error: {}", e);
            }
        }
    }
    Ok(())
}

/// ## Function: `handle_connection` - Handle a single connection.
///
/// This function takes a TcpStream object and a string for the document
/// root, and handles a single connection.
///
/// # Arguments
///
/// * `stream`        - A TcpStream object.
/// * `document_root` - A string for the document root.
///
/// # Returns
///
/// * A Result indicating success or failure.
/// - Ok() if the connection was handled successfully.
/// - Err() if the connection could not be handled.
///
/// # Errors
///
/// * If the server fails to read data from a connection, it will
/// return an error.
///
pub fn handle_connection(
    mut stream: TcpStream,
    document_root: &str,
) -> std::io::Result<()> {
    let mut buffer = [0; 1024];
    let bytes_read = stream.read(&mut buffer)?;
    if bytes_read == 0 {
        return Ok(());
    }

    let request = String::from_utf8_lossy(&buffer[..bytes_read]);
    let request_line = request.lines().next().unwrap_or("");
    let mut request_parts = request_line.split_whitespace();

    let (_method, path, _version) = match (
        request_parts.next(),
        request_parts.next(),
        request_parts.next(),
    ) {
        (Some(method), Some(path), Some(version)) => {
            (method, path, version)
        }
        _ => {
            eprintln!("Malformed request line: {}", request_line);
            return Ok(());
        }
    };

    let requested_file = match path {
        "/" => "index.html",
        _ => &path[1..], // Remove the leading "/"
    };

    let document_root = Path::new(&document_root);
    let requested_path = document_root.join(requested_file);

    // Canonicalize paths and check for directory traversal attempts
    let canonical_document_root = document_root.canonicalize()?;
    let canonical_requested_path = requested_path.canonicalize()?;

    if !canonical_requested_path.starts_with(&canonical_document_root) {
        eprintln!(
            "Possible directory traversal attempt: {}",
            requested_file
        );
        return Ok(());
    }

    // Continue processing as normal, the requested path has been validated
    let (status_line, contents) = if canonical_requested_path.exists() {
        (
            "HTTP/1.1 200 OK\r\n\r\n",
            std::fs::read_to_string(&canonical_requested_path)
                .unwrap_or_default(),
        )
    } else {
        (
            "HTTP/1.1 404 NOT FOUND\r\n\r\n",
            std::fs::read_to_string(
                canonical_document_root.join("404/index.html"),
            )
            .unwrap_or_else(|_| String::from("File not found")),
        )
    };

    match stream.write_all(status_line.as_bytes()) {
        Err(e) => {
            eprintln!("Error writing to stream: {}", e);
            return Err(e);
        }
        _ => (),
    };

    match stream.write_all(contents.as_bytes()) {
        Err(e) => {
            eprintln!("Error writing to stream: {}", e);
            return Err(e);
        }
        _ => (),
    };

    match stream.flush() {
        Err(e) => {
            eprintln!("Error flushing stream: {}", e);
            return Err(e);
        }
        _ => (),
    };

    Ok(())
}

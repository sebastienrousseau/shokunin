// Copyright © 2023 Shokunin (職人). All rights reserved.
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
    println!("Server running at http://{}", server_address);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => handle_connection(stream, document_root)?,
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
        eprintln!("Empty request received");
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

    let file_path = Path::new(document_root).join(requested_file);

    let (status_line, contents) = if file_path.exists() {
        (
            "HTTP/1.1 200 OK\r\n\r

",
            std::fs::read_to_string(&file_path).unwrap_or_default(),
        )
    } else {
        (
            "HTTP/1.1 404 NOT FOUND\r\n\r

",
            std::fs::read_to_string(
                Path::new(document_root).join("404/index.html"),
            )
            .unwrap_or_else(|_| String::from("File not found")),
        )
    };

    stream.write_all(status_line.as_bytes())?;
    stream.write_all(contents.as_bytes())?;
    stream.flush()?;

    Ok(())
}

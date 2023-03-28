use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::path::Path;
/// Start a web server to serve the public directory.
pub fn start(
    server_address: &str,
    document_root: &str,
) -> std::io::Result<()> {
    let listener = TcpListener::bind(server_address)?;
    println!("Server running at http://{}", server_address);

    for stream in listener.incoming() {
        let stream = stream?;
        handle_connection(stream, document_root)?;
    }
    Ok(())
}
/// Handle a single connection.
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
            "HTTP/1.1 200 OK\r\n\r\n",
            std::fs::read_to_string(&file_path).unwrap_or_default(),
        )
    } else {
        (
            "HTTP/1.1 404 NOT FOUND\r\n\r\n",
            std::fs::read_to_string(
                Path::new(document_root).join("404.html"),
            )
            .unwrap_or_else(|_| String::from("File not found")),
        )
    };

    stream.write_all(status_line.as_bytes())?;
    stream.write_all(contents.as_bytes())?;
    stream.flush()?;

    Ok(())
}

use crate::error::ServerError;
use crate::request::Request;
use crate::response::Response;
use std::fs;
use std::io;
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::thread;

/// Represents the SSG server and its configuration.
pub struct Server {
    address: String,
    document_root: PathBuf,
}

impl Server {
    /// Creates a new `Server` instance.
    ///
    /// # Arguments
    ///
    /// * `address` - A string slice that holds the IP address and port (e.g., "127.0.0.1:8080").
    /// * `document_root` - A string slice that holds the path to the document root directory.
    ///
    /// # Returns
    ///
    /// A new `Server` instance.
    pub fn new(address: &str, document_root: &str) -> Self {
        Server {
            address: address.to_string(),
            document_root: PathBuf::from(document_root),
        }
    }

    /// Starts the server and begins listening for incoming connections.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or an I/O error.
    pub fn start(&self) -> io::Result<()> {
        let listener = TcpListener::bind(&self.address)?;
        println!("❯ Server is now running at http://{}", self.address);
        println!("  Document root: {}", self.document_root.display());
        println!("  Press Ctrl+C to stop the server.");

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let document_root = self.document_root.clone();
                    thread::spawn(move || {
                        if let Err(e) =
                            handle_connection(stream, &document_root)
                        {
                            eprintln!(
                                "Error handling connection: {}",
                                e
                            );
                        }
                    });
                }
                Err(e) => eprintln!("Connection error: {}", e),
            }
        }

        Ok(())
    }
}

/// Handles a single client connection.
///
/// # Arguments
///
/// * `stream` - A `TcpStream` representing the client connection.
/// * `document_root` - A `PathBuf` representing the server's document root.
///
/// # Returns
///
/// A `Result` indicating success or a `ServerError`.
fn handle_connection(
    mut stream: TcpStream,
    document_root: &Path,
) -> Result<(), ServerError> {
    let request = Request::from_stream(&stream)?;
    let response = generate_response(&request, document_root)?;
    response.send(&mut stream)?;
    Ok(())
}

/// Generates an HTTP response based on the requested file.
///
/// # Arguments
///
/// * `request` - A `Request` instance representing the client's request.
/// * `document_root` - A `Path` representing the server's document root.
///
/// # Returns
///
/// A `Result` containing the `Response` or a `ServerError`.
fn generate_response(
    request: &Request,
    document_root: &Path,
) -> Result<Response, ServerError> {
    let mut path = PathBuf::from(document_root);
    let request_path = request.path.trim_start_matches('/');

    if request_path.is_empty() {
        // If the request is for the root, append "index.html"
        path.push("index.html");
    } else {
        for component in request_path.split('/') {
            if component == ".." {
                path.pop();
            } else {
                path.push(component);
            }
        }
    }

    if !path.starts_with(document_root) {
        return Err(ServerError::Forbidden);
    }

    if path.is_file() {
        let contents = fs::read(&path)?;
        let content_type = get_content_type(&path);
        let mut response = Response::new(200, "OK", contents);
        response.add_header("Content-Type", content_type);
        Ok(response)
    } else if path.is_dir() {
        // If it's a directory, try to serve index.html from that directory
        path.push("index.html");
        if path.is_file() {
            let contents = fs::read(&path)?;
            let content_type = get_content_type(&path);
            let mut response = Response::new(200, "OK", contents);
            response.add_header("Content-Type", content_type);
            Ok(response)
        } else {
            generate_404_response(document_root)
        }
    } else {
        generate_404_response(document_root)
    }
}

/// Generates a 404 Not Found response.
///
/// # Arguments
///
/// * `document_root` - A `Path` representing the server's document root.
///
/// # Returns
///
/// A `Result` containing the `Response` or a `ServerError`.
fn generate_404_response(
    document_root: &Path,
) -> Result<Response, ServerError> {
    let not_found_path = document_root.join("404/index.html");
    let contents = if not_found_path.is_file() {
        fs::read(not_found_path)?
    } else {
        b"404 Not Found".to_vec()
    };
    let mut response = Response::new(404, "NOT FOUND", contents);
    response.add_header("Content-Type", "text/html");
    Ok(response)
}

/// Determines the content type based on the file extension.
///
/// # Arguments
///
/// * `path` - A `Path` representing the file path.
///
/// # Returns
///
/// A string slice representing the content type.
fn get_content_type(path: &Path) -> &'static str {
    match path.extension().and_then(std::ffi::OsStr::to_str) {
        Some("html") => "text/html",
        Some("css") => "text/css",
        Some("js") => "application/javascript",
        Some("json") => "application/json",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("svg") => "image/svg+xml",
        _ => "application/octet-stream",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;

    fn setup_test_directory() -> TempDir {
        let temp_dir = TempDir::new().unwrap();
        let root_path = temp_dir.path();

        // Create index.html
        let mut index_file =
            File::create(root_path.join("index.html")).unwrap();
        index_file
            .write_all(b"<html><body>Hello, World!</body></html>")
            .unwrap();

        // Create 404/index.html
        fs::create_dir(root_path.join("404")).unwrap();
        let mut not_found_file =
            File::create(root_path.join("404/index.html")).unwrap();
        not_found_file
            .write_all(b"<html><body>404 Not Found</body></html>")
            .unwrap();

        // Create a subdirectory with its own index.html
        fs::create_dir(root_path.join("subdir")).unwrap();
        let mut subdir_index_file =
            File::create(root_path.join("subdir/index.html")).unwrap();
        subdir_index_file
            .write_all(b"<html><body>Subdirectory Index</body></html>")
            .unwrap();

        temp_dir
    }

    #[test]
    fn test_server_creation() {
        let server = Server::new("127.0.0.1:8080", "/var/www");
        assert_eq!(server.address, "127.0.0.1:8080");
        assert_eq!(server.document_root, PathBuf::from("/var/www"));
    }

    #[test]
    fn test_get_content_type() {
        assert_eq!(
            get_content_type(Path::new("test.html")),
            "text/html"
        );
        assert_eq!(
            get_content_type(Path::new("style.css")),
            "text/css"
        );
        assert_eq!(
            get_content_type(Path::new("script.js")),
            "application/javascript"
        );
        assert_eq!(
            get_content_type(Path::new("data.json")),
            "application/json"
        );
        assert_eq!(
            get_content_type(Path::new("image.png")),
            "image/png"
        );
        assert_eq!(
            get_content_type(Path::new("photo.jpg")),
            "image/jpeg"
        );
        assert_eq!(
            get_content_type(Path::new("animation.gif")),
            "image/gif"
        );
        assert_eq!(
            get_content_type(Path::new("icon.svg")),
            "image/svg+xml"
        );
        assert_eq!(
            get_content_type(Path::new("unknown.xyz")),
            "application/octet-stream"
        );
    }

    #[test]
    fn test_generate_response() {
        let temp_dir = setup_test_directory();
        let document_root = temp_dir.path();

        // Test root request (should serve index.html)
        let root_request = Request {
            method: "GET".to_string(),
            path: "/".to_string(),
            version: "HTTP/1.1".to_string(),
        };

        let root_response =
            generate_response(&root_request, document_root).unwrap();
        assert_eq!(root_response.status_code, 200);
        assert_eq!(root_response.status_text, "OK");
        assert!(root_response
            .body
            .starts_with(b"<html><body>Hello, World!</body></html>"));

        // Test specific file request
        let file_request = Request {
            method: "GET".to_string(),
            path: "/index.html".to_string(),
            version: "HTTP/1.1".to_string(),
        };

        let file_response =
            generate_response(&file_request, document_root).unwrap();
        assert_eq!(file_response.status_code, 200);
        assert_eq!(file_response.status_text, "OK");
        assert!(file_response
            .body
            .starts_with(b"<html><body>Hello, World!</body></html>"));

        // Test subdirectory index request
        let subdir_request = Request {
            method: "GET".to_string(),
            path: "/subdir/".to_string(),
            version: "HTTP/1.1".to_string(),
        };

        let subdir_response =
            generate_response(&subdir_request, document_root).unwrap();
        assert_eq!(subdir_response.status_code, 200);
        assert_eq!(subdir_response.status_text, "OK");
        assert!(subdir_response.body.starts_with(
            b"<html><body>Subdirectory Index</body></html>"
        ));

        // Test non-existent file request
        let not_found_request = Request {
            method: "GET".to_string(),
            path: "/nonexistent.html".to_string(),
            version: "HTTP/1.1".to_string(),
        };

        let not_found_response =
            generate_response(&not_found_request, document_root)
                .unwrap();
        assert_eq!(not_found_response.status_code, 404);
        assert_eq!(not_found_response.status_text, "NOT FOUND");
        assert!(not_found_response
            .body
            .starts_with(b"<html><body>404 Not Found</body></html>"));

        // Test directory traversal attempt
        let traversal_request = Request {
            method: "GET".to_string(),
            path: "/../outside.html".to_string(),
            version: "HTTP/1.1".to_string(),
        };

        let traversal_response =
            generate_response(&traversal_request, document_root);
        assert!(matches!(
            traversal_response,
            Err(ServerError::Forbidden)
        ));
    }
}

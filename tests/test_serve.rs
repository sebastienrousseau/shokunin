#[cfg(test)]
mod tests {
    use ssg::utilities::serve::start;
    use std::fs::File;
    use std::io::{Read, Write};
    use std::net::TcpStream;
    use std::thread;
    use tempfile::TempDir;

    #[test]
    fn test_handle_connection() {
        // Create a temporary directory and a dummy file inside it.
        let tmp_dir = TempDir::new().unwrap();
        let file_path = tmp_dir.path().join("index.html");
        let mut tmp_file = File::create(file_path).unwrap();
        write!(tmp_file, "Hello, world!").unwrap();

        // Start a server in a new thread.
        let server_addr = "127.0.0.1:3000";
        let document_root =
            tmp_dir.path().to_str().unwrap().to_string();
        thread::spawn(move || {
            start(server_addr, &document_root).unwrap();
        });

        // Wait for the server to start.
        thread::sleep(std::time::Duration::from_secs(1));

        // Connect to the server and send a request.
        let mut stream = TcpStream::connect(server_addr).unwrap();
        stream
            .write_all(b"GET /index.html HTTP/1.1\r\n\r\n")
            .unwrap();

        // Check the response.
        let mut buffer = Vec::new();
        stream.read_to_end(&mut buffer).unwrap();
        let response = String::from_utf8(buffer).unwrap();

        assert!(response.contains("200 OK"));
        assert!(response.contains("Content-Type: text/html"));
        assert!(response.contains("Hello, world!"));
    }
}

use crate::error::ServerError;
use std::io::Write;
use std::net::TcpStream;

/// Represents an HTTP response, including the status code, status text, headers, and body.
pub struct Response {
    /// The HTTP status code (e.g., 200 for OK, 404 for Not Found).
    pub status_code: u16,

    /// The HTTP status text associated with the status code (e.g., "OK", "Not Found").
    pub status_text: String,

    /// A list of headers in the response, each represented as a tuple containing the header
    /// name and its corresponding value.
    pub headers: Vec<(String, String)>,

    /// The body of the response, represented as a vector of bytes.
    pub body: Vec<u8>,
}

impl Response {
    /// Creates a new `Response` with the given status code, status text, and body.
    ///
    /// The headers are initialized as an empty list and can be added later using the `add_header` method.
    ///
    /// # Arguments
    ///
    /// * `status_code` - The HTTP status code for the response.
    /// * `status_text` - The status text corresponding to the status code.
    /// * `body` - The body of the response, represented as a vector of bytes.
    ///
    /// # Returns
    ///
    /// A new `Response` instance with the specified status code, status text, and body.
    pub fn new(
        status_code: u16,
        status_text: &str,
        body: Vec<u8>,
    ) -> Self {
        Response {
            status_code,
            status_text: status_text.to_string(),
            headers: Vec::new(),
            body,
        }
    }

    /// Adds a header to the response.
    ///
    /// This method allows you to add custom headers to the response, which will be included
    /// in the HTTP response when it is sent to the client.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the header (e.g., "Content-Type").
    /// * `value` - The value of the header (e.g., "text/html").
    pub fn add_header(&mut self, name: &str, value: &str) {
        self.headers.push((name.to_string(), value.to_string()));
    }

    /// Sends the response over the provided `TcpStream`.
    ///
    /// This method writes the HTTP status line, headers, and body to the stream, ensuring
    /// the client receives the complete response. It uses the `write!` macro and the `TcpStream`
    /// to communicate with the client.
    ///
    /// # Arguments
    ///
    /// * `stream` - A mutable reference to the `TcpStream` over which the response will be sent.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the response is successfully sent.
    /// * `Err(ServerError)` - If an error occurs while sending the response.
    ///
    /// # Errors
    ///
    /// This function returns a `ServerError` if there is any issue writing to the stream or
    /// sending the response data.
    pub fn send(
        &self,
        stream: &mut TcpStream,
    ) -> Result<(), ServerError> {
        write!(
            stream,
            "HTTP/1.1 {} {}\r\n",
            self.status_code, self.status_text
        )?;

        for (name, value) in &self.headers {
            write!(stream, "{}: {}\r\n", name, value)?;
        }

        write!(stream, "\r\n")?;
        stream.write_all(&self.body)?;
        stream.flush()?;

        Ok(())
    }
}

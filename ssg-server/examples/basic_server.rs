//! # Basic SSG Server Example
//!
//! This example demonstrates how to use the `ssg-server` to serve static files from a public directory.
//! It sets up an HTTP server that listens on `127.0.0.1:8080` and serves files located in the `./public` directory.
//!
//! ## Usage
//!
//! To run this example, make sure you have a folder named `public` with some files to serve,
//! and then execute the binary. The server will start and serve files to any client making a request to it.

use ssg_server::Server;

fn main() -> std::io::Result<()> {
    let server = Server::new("127.0.0.1:8080", "./public");
    server.start()
}

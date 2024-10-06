// SPDX-License-Identifier: Apache-2.0 OR MIT
// See LICENSE-APACHE.md and LICENSE-MIT.md in the repository root for full license information.

#![allow(missing_docs)]

//! # Server Benchmark
//!
//! This benchmark measures the performance of the `ssg-server` by simulating a request to
//! serve a static HTML file. It uses the `Criterion` library to run the benchmarks and
//! measure the time taken to handle HTTP requests on a local TCP server.
//!
//! ## How it works
//!
//! - A temporary directory is created, and a test HTML file is written to it.
//! - A server is started in a separate thread, serving the contents of the temporary directory.
//! - A TCP client sends an HTTP request to fetch the HTML file, and the server responds.
//! - The benchmark measures the time taken to process the request and receive the response.

use criterion::{
    black_box, criterion_group, criterion_main, Criterion,
};
use ssg_server::Server;
use std::fs::File;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::thread;
use tempfile::TempDir;

/// Benchmarks the performance of the server by making a request for a static HTML file.
///
/// This function creates a temporary directory, starts the server, sends an HTTP request,
/// and measures the time taken to receive a response. The `Criterion` library is used to
/// manage the benchmarking process.
///
/// # Steps
///
/// 1. A temporary directory is created, and an HTML file is written to it.
/// 2. The server is started in a separate thread to serve the contents of the directory.
/// 3. A client sends a GET request for the HTML file.
/// 4. The response is read, and the duration is measured.
/// 5. The server is cleaned up after the benchmark.
fn benchmark_server(c: &mut Criterion) {
    // Wrap TempDir in Arc<Mutex<>> so we can share it between threads
    let temp_dir = Arc::new(Mutex::new(TempDir::new().unwrap()));
    let root_path = temp_dir.lock().unwrap().path().to_path_buf();

    // Create a test file
    let mut test_file =
        File::create(root_path.join("test.html")).unwrap();
    test_file
        .write_all(b"<html><body>Test Content</body></html>")
        .unwrap();

    // Clone Arc for the server thread
    let temp_dir_clone = Arc::clone(&temp_dir);

    // Start the server in a separate thread
    let _server_thread = thread::spawn(move || {
        let server =
            Server::new("127.0.0.1:8082", root_path.to_str().unwrap());
        server.start().unwrap();
        // Keep the TempDir alive for the duration of the thread
        drop(temp_dir_clone);
    });

    // Give the server a moment to start
    thread::sleep(std::time::Duration::from_millis(100));

    c.bench_function("server_request", |b| {
        b.iter(|| {
            let mut stream =
                TcpStream::connect("127.0.0.1:8082").unwrap();
            write!(stream, "GET /test.html HTTP/1.1\r\n\r\n").unwrap();

            // Read the response
            let mut buffer = [0; 1024];
            let bytes_read = stream.read(&mut buffer).unwrap();

            // Use black_box to prevent the compiler from optimizing away the read operation
            black_box(&buffer[..bytes_read]);
        })
    });

    // Clean up
    // Note: In a real scenario, we'd need a way to stop the server gracefully.
    // For this benchmark, we're relying on the process ending to stop the server.

    // Ensure the TempDir is kept alive until here
    drop(temp_dir);
}

// Criterion group and main function to set up and run the benchmark.
criterion_group!(benches, benchmark_server);
criterion_main!(benches);

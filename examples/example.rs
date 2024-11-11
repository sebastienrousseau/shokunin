// Copyright © 2023-2024 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! This is a main function for a simple static site generator (ssg) example.
//! It is intended to demonstrate how to use the `ssg` library to generate a
//! static website from Markdown files.
//!
//! This example demonstrates the process of compiling and generating the static
//! website from markdown files. It makes use of the `ssg::compiler::compile`
//! function from the `ssg` library, and standard Rust libraries for handling
//! paths and errors.
//!
//! The `main` function below defines the paths to the various directories
//! involved in the website generation process, then calls the `compile`
//! function to generate the website.

// Import the required libraries and modules.
use anyhow::Result;
use http_handle::Server;
use staticdatagen::compiler::service::compile;
use std::path::Path;

fn main() -> Result<()> {
    // Define the paths to the build, site, source and template directories.

    // The build directory.
    // This is where the generated website will be placed temporarily before
    // being moved to the site directory.
    let build_path = Path::new("examples/build");

    // The site directory.
    // This is where the final generated website will be placed.
    let site_path = Path::new("examples/public");

    // The content directory.
    // This is where the source content files are located (e.g., Markdown files).
    // These files will be converted into HTML files in the build process.
    let content_path = Path::new("content");

    // The template directory.
    // This is where the HTML template files are located.
    // These templates are used to structure the content from the Markdown files.
    let template_path = Path::new("templates");

    // Call the compile function to generate the website.
    // The function takes the paths defined above as arguments and will
    // throw an error if anything goes wrong during the compilation process.
    compile(build_path, content_path, site_path, template_path)?;

    // Serve the generated website locally.
    let example_root: String = site_path.to_str().unwrap().to_string();

    // Create a new server with an address and document root
    let server = Server::new("127.0.0.1:3000", example_root.as_str());

    // Start the server
    let _ = server.start();

    // If everything goes well, return Ok.
    Ok(())
}

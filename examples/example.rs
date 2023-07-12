// Copyright © 2023 Shokunin (職人) Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! The main function of the program.

use ssg::compiler::compile;
use std::path::Path;

/// The example.com website is a simple static website with a few pages and
/// assets.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Define the paths to the build, site, source and template directories.
    let build_path = Path::new("examples/example.com/build");
    let site_path = Path::new("examples/example.com/public");
    let content_path = Path::new("content");
    let template_path = Path::new("template");

    compile(build_path, content_path, site_path, template_path)?;

    Ok(())
}

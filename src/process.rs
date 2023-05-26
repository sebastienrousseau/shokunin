// Copyright © 2023 Shokunin (職人) Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

use crate::macro_check_directory;
use crate::{compile, macro_get_args};
use clap::ArgMatches;
use std::path::Path;

/// ## Function: args - returns a Result indicating success or failure
///
/// Parses the command-line arguments passed by the static site generator
/// command-line tool (ssg) and compiles the project.
///
/// - This function parses the `content` directory where the markdown files for
/// your website are stored and the `output` directory where the compiled site
/// will be created from the `matches` object.
///
/// - It then, validates that these directories exist, or creates them on the
/// fly if they do not. If either directory cannot be found or created, an
/// error is returned.
///
/// - Finally, it calls the `compile` function to create the new project using
/// the markdown files in the "content" directory, and returns an error if the
/// compilation process fails.
///
/// # Arguments
///
/// * `matches` - A reference to an ArgMatches object containing the command-
/// line arguments passed to the tool. This is created by the `clap` crate.
///
/// # Returns
///
/// * A Result indicating success or failure.
/// - Ok() if the project was created successfully and the output files were
/// written to the output directory.
/// - Err(anyhow::Error) if the project could not be created or the output files
/// could not be written to the output directory.
///
pub fn args(matches: &ArgMatches) -> Result<(), String> {
    // Set the content elements of the new project
    let content_dir = macro_get_args!(matches, "content");
    let output_dir = macro_get_args!(matches, "output");
    let site_dir = macro_get_args!(matches, "new");
    let template_dir = macro_get_args!(matches, "template");

    // Create Path objects for the content and output directories
    let content_path = Path::new(&content_dir);
    let build_path = Path::new(&output_dir);
    let site_path = Path::new(&site_dir);
    let template_path = Path::new(&template_dir);

    // Ensure the build, content, site and template directories exist
    macro_check_directory!(content_path, "content");
    macro_check_directory!(build_path, "output");
    macro_check_directory!(site_path, "new");
    macro_check_directory!(template_path, "template");

    // Create the new project
    let compilation_result = compile(build_path, content_path, site_path, template_path);
    match compilation_result {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("❌ Error: {}", e)),
    }
}

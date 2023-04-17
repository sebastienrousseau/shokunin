// Copyright © 2023 Shokunin (職人). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

use super::utilities::directory;
use clap::ArgMatches;
use std::path::Path;

use super::compile;

/// ## Function: args - returns a Result containing the parsed input options
///
/// Processes the command-line arguments passed by the static site
/// generator command-line tool (ssg).
///
/// - This function parses the `content` directory where the markdown
/// files for your website are stored and the `output` directory where
/// the compiled site will be created from the `matches` object.
///
/// - It then, validates that these directories exist, or creates them on
/// the fly if they do not. If either directory cannot be found or
/// created, an error message is returned.
///
/// - Finally, it calls the `compile` function to create the new project
/// using the markdown files in the "content" directory, and returns an
/// error message if the compilation process fails.
///
/// # Arguments
///
/// * `matches` - A reference to an ArgMatches object containing the
///               command-line arguments passed to the tool. This is
///               created by the `clap` crate.
///
/// # Returns
///
/// * A Result indicating success or failure.
/// - Ok() if the project was created successfully and the output files
///  were written to the output directory.
/// - Err() if the project could not be created or the output files
/// could not be written to the output directory.
///
pub fn args(matches: &ArgMatches) -> Result<(), String> {
    // Retrieve the name of the project
    let project_src = match matches.get_one::<String>("new") {
        Some(name) => name.to_owned(),
        None => {
            return Err("❌ Error: Argument \"name\" is required but missing.".to_owned());
        }
    };

    // Retrieve the content and output directory arguments
    let arg_src = match matches.get_one::<String>("content") {
        Some(src) => src.to_owned(),
        None => {
            return Err("❌ Error: Argument \"content\" is required but missing.".to_owned());
        }
    };

    let arg_out = match matches.get_one::<String>("output") {
        Some(out) => out.to_owned(),
        None => {
            return Err("❌ Error: Argument \"output\" is required but missing.".to_owned());
        }
    };

    let arg_template = matches.get_one::<String>("template");

    // Create Path objects for the content and output directories
    let src_dir = Path::new(&arg_src);
    let out_dir = Path::new(&arg_out);

    // Ensure source and output directories exist or create them
    if let Err(e) = directory(src_dir, "content") {
        return Err(format!("❌ Error: {}", e));
    }
    if let Err(e) = directory(out_dir, "output") {
        return Err(format!("❌ Error: {}", e));
    }

    // Create the new project
    let new_project = compile(src_dir, out_dir, arg_template, project_src);
    match new_project {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("❌ Error: {}", e)),
    }
}

use std::fs;
use std::path::Path;

use clap::ArgMatches;

use crate::create_new_project;

/// Function to ensure a directory exists or create it.
/// - If the directory does not exist, it will be created.
/// - If the directory exists, it will be checked to ensure it is a
/// directory and not a file.
///
/// # Arguments
///
/// * `matches` - A reference to an ArgMatches object containing command
///               line arguments.
///
///
pub fn process_arguments(matches: &ArgMatches) -> Result<(), String> {
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

    // Create Path objects for the content and output directories
    let src_dir = Path::new(&arg_src);
    let out_dir = Path::new(&arg_out);

    // Ensure source and output directories exist or create them
    if let Err(e) = ensure_directory_exists(&src_dir, "content") {
        return Err(format!("❌ Error: {}", e));
    }
    if let Err(e) = ensure_directory_exists(&out_dir, "output") {
        return Err(format!("❌ Error: {}", e));
    }

    // Create the new project
    let new_project = create_new_project(src_dir, out_dir);
    match new_project {
        Ok(_) => {
            println!("✅ Done.");
            Ok(())
        }
        Err(e) => Err(format!("❌ Error: {}", e)),
    }
}

/// Ensure a directory exists, creating it if necessary.
///
/// # Arguments
///
/// * `dir`  - A reference to a Path object for the directory.
/// * `name` - A human-readable name for the directory, used in error
///            messages.
///
/// # Returns
///
/// * A Result indicating success or failure.
///  - Ok() if the directory exists or was created successfully.
///  - Err() if the directory does not exist and could not be created.
///
///
fn ensure_directory_exists(dir: &Path, name: &str) -> Result<(), String> {
    if dir.exists() {
        if !dir.is_dir() {
            return Err(format!("❌ Error: {} is not a directory.", name));
        }
    } else if let Err(e) = fs::create_dir_all(dir) {
        return Err(format!("❌ Error: Cannot create {} directory: {}", name, e));
    }

    Ok(())
}

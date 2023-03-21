use std::fs;
use std::path::Path;

use clap::ArgMatches;

use super::compile;

/// Processes the command-line arguments passed to the static site
/// generator command-line tool.
///
/// This function retrieves the "content" and "output" directory
/// arguments from the `matches` object, validates them, and then
/// creates a new project using the `compile` function.
///
/// The function first ensures that the "content" and "output"
/// directories exist, or creates them if they do not exist. If either
/// directory cannot be found or created, an error message is returned.
///
/// The function then calls the `compile` function to create a new
/// project from the files in the "content" directory, and returns an
/// error message if the compilation process fails.
///
/// If all operations are successful, `Ok(())` is returned.
///
/// # Arguments
///
/// * `matches` - A reference to an ArgMatches object containing the
///               command-line arguments passed to the tool.
///
/// # Returns
///
/// * A Result indicating success or failure.
/// - Ok() if the project was created successfully.
/// - Err() if the project could not be created.
///
pub fn process_arguments(matches: &ArgMatches) -> Result<(), String> {
    // Retrieve the name of the project
    let project_src = match matches.get_one::<String>("new") {
        Some(name) => name.to_owned(),
        None => {
            return Err(
                "❌ Error: Argument \"name\" is required but missing."
                    .to_owned(),
            );
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

    // Create Path objects for the content and output directories
    let project_dir = Path::new(&project_src);
    let src_dir = Path::new(&arg_src);
    let out_dir = Path::new(&arg_out);

    // Ensure source and output directories exist or create them
    if let Err(e) = ensure_directory_exists(project_dir, "new") {
        return Err(format!("❌ Error: {}", e));
    }
    if let Err(e) = ensure_directory_exists(src_dir, "content") {
        return Err(format!("❌ Error: {}", e));
    }
    if let Err(e) = ensure_directory_exists(&out_dir, "output") {
        return Err(format!("❌ Error: {}", e));
    }

    // Create the new project
    let new_project = compile(src_dir, &out_dir);
    match new_project {
        Ok(_) => Ok(()),
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
fn ensure_directory_exists(
    dir: &Path,
    name: &str,
) -> Result<(), String> {
    if dir.exists() {
        if !dir.is_dir() {
            return Err(format!(
                "❌ Error: {} is not a directory.",
                name
            ));
        }
    } else if let Err(e) = fs::create_dir_all(dir) {
        return Err(format!(
            "❌ Error: Cannot create {} directory: {}",
            name, e
        ));
    }

    Ok(())
}

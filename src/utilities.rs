use std::{fs, path::Path};

/// ## Function: `directory` - Ensure a directory exists, creating it if necessary.
///
/// This function takes a reference to a Path object for a directory and
/// a human-readable name for the directory, and creates the directory
/// if it does not already exist.
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
/// # Example
///
/// ```
/// use std::path::Path;
/// use std::fs;
/// use ssg::utilities::directory;
///
/// // Create a "logs" directory if it doesn't exist
/// let dir = Path::new("logs");
/// directory(dir, "logs").expect("Could not create logs directory");
/// fs::remove_dir_all(dir).expect("Could not remove logs directory");
/// ```
///
/// Note that the error message returned by this function is formatted
/// as a string, starting with the "❌ Error: " prefix, followed by the
/// error message.
///
pub fn directory(dir: &Path, name: &str) -> Result<(), String> {
    if dir.exists() {
        if !dir.is_dir() {
            return Err(format!(
                "❌ Error: {} is not a directory.",
                name
            ));
        }
    } else {
        match fs::create_dir_all(dir) {
            Ok(_) => {}
            Err(e) => {
                return Err(format!(
                    "❌ Error: Cannot create {} directory: {}",
                    name, e
                ))
            }
        }
    }

    Ok(())
}

/// ## Function: `move_output_directory` - Move the output directory to the public directory.
///
/// This function takes a reference to a Path object for the output
/// directory and a string for the site name, and moves the output
/// directory to the public directory.
///
/// # Arguments
///
/// * `site_name` - A string for the site name.
/// * `out_dir`   - A reference to a Path object for the output directory.
///
/// # Returns
///
/// * A Result indicating success or failure.
/// - Ok() if the output directory was moved successfully.
/// - Err() if the output directory could not be moved.
///
pub fn move_output_directory(
    // The name of the site.
    site_name: &str,
    // The path to the output directory.
    out_dir: &Path,
) -> std::io::Result<()> {
    // Move the output directory to the public directory
    println!("❯ Moving output directory...");

    // Create a Path object for the public directory
    let public_dir = Path::new("public");

    // Remove the public directory if it exists
    if public_dir.exists() {
        fs::remove_dir_all(public_dir)?;
    }

    // Create the public directory
    fs::create_dir(public_dir)?;

    // Replace spaces with underscores in the site_name
    let site_name = site_name.replace(' ', "_");

    // Create a new directory under public with the site_name
    let new_project_dir = public_dir.join(site_name);
    fs::create_dir_all(new_project_dir.clone())?;

    // Move the output directory to the new_project_dir
    fs::rename(out_dir, &new_project_dir)?;

    // Print a success message
    println!("  Done.\n");

    Ok(())
}

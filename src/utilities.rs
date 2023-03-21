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
/// use ssg::utilities::directory;
///
/// // Create a "logs" directory if it doesn't exist
/// let dir = Path::new("logs");
/// directory(dir, "logs").expect("Could not create logs directory");
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

use minify_html::{Cfg, minify};
use std::{
    fs::{self, File},
    io::{self, Write},
    path::{Path, PathBuf},
};

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

/// ## Function: `minify_html_files` - Minify HTML files in the output directory.
pub fn minify_html_files(
    // The path to the output directory.
    out_dir: &Path,
) -> io::Result<()> {
    let html_files = find_html_files(out_dir)?;

    for file in html_files {
        let minified_html = minify_html(&file)?;
        let backup_path = backup_file(&file)?;
        write_minified_html(&file, &minified_html)?;
        println!(
            "Minified HTML file '{}' to '{}'",
            file.display(),
            backup_path.display()
        );
    }

    Ok(())
}

fn find_html_files(dir: &Path) -> io::Result<Vec<PathBuf>> {
    let mut html_files = Vec::new();

    for entry in fs::read_dir(dir)? {
        let entry = entry?;

        if entry.path().is_dir() {
            let sub_html_files = find_html_files(&entry.path())?;
            html_files.extend(sub_html_files);
        } else if let Some("html") =
            entry.path().extension().and_then(|ext| ext.to_str())
        {
            html_files.push(entry.path());
        }
    }

    Ok(html_files)
}

fn minify_html(file_path: &Path) -> io::Result<String> {
    let mut cfg = Cfg::new();
    cfg.do_not_minify_doctype = true;
    cfg.keep_closing_tags = true;
    cfg.keep_comments = false;
    cfg.minify_css = true;
    cfg.minify_js = true;
    cfg.remove_processing_instructions = true;
    cfg.remove_bangs = true;
    let file_content = fs::read(file_path)?;
    let minified_content = minify(&file_content,  &cfg);

    Ok(String::from_utf8(minified_content).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?)
}

fn backup_file(file_path: &Path) -> io::Result<PathBuf> {
    let backup_path = file_path.with_extension("src.html");
    fs::copy(file_path, &backup_path)?;
    Ok(backup_path)
}

fn write_minified_html(
    file_path: &Path,
    minified_html: &str,
) -> io::Result<()> {
    let mut file = File::create(file_path)?;
    file.write_all(minified_html.as_bytes())?;
    Ok(())
}

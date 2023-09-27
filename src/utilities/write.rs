// Copyright © 2023 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

// Import the FileData object definition.
use crate::models::data::FileData;

// Import the minify_html function.
use crate::utilities::minification::minify_html;

// Import the Error trait.
use std::error::Error;

// Import the fs module.
use std::fs;

// Import the Path type.
use std::path::Path;

/// Write the files to the build directory.
///
/// The `build_dir_path` parameter is the path to the build directory.
///
/// The `file` parameter is the file data object to write.
///
/// The `template_path` parameter is the path to the template directory.
///
/// Returns a `Result` value, which is either `Ok(())` on success or
/// `Err(Box<dyn Error>)` on failure.
pub fn write_files_to_build_directory(
    build_dir_path: &Path,
    file: &FileData,
    template_path: &Path
) -> Result<(), Box<dyn Error>> {

    // Determine the file name without extension and check if it's "index".
    let file_name = match (Path::new(&file.name).extension().and_then(
        |s| s.to_str()
    ), &file.name) {
        (Some(ext), name) if
        ["js", "json", "md", "toml", "txt", "xml"].contains(&ext) => {
            name.replace(&format!(".{}", ext), "")
        },
        _ => file.name.to_string(),
    };

    // Check if the file name is "index" to decide the directory structure.
    let index_html_minified = file_name == "index";
    let dir_name = build_dir_path.join(&file_name);

    if file_name == "index" {
        // List of files that need special handling when the file name is
        // "index".
        let file_paths = [
            ("CNAME", &file.cname),
            ("humans.txt", &file.human),
            ("index.html", &file.content),
            ("manifest.json", &file.json),
            ("robots.txt", &file.txt),
            ("rss.xml", &file.rss),
            ("sitemap.xml", &file.sitemap),
        ];

        // Print the section header.
        println!("\n{}", file_name.to_uppercase());

        // Process each file in the list.
        for (file_name, content) in &file_paths {
            write_and_minify_file(
                build_dir_path,
                file_name,
                content,
                index_html_minified
            )?;
        }

        // Copy other template files to the build directory.
        let other_files = ["main.js", "sw.js"];
        for file_name in &other_files {
            copy_template_file(
                template_path,
                build_dir_path,
                file_name
            )?;
        }
    } else {
        // Create a subdirectory based on the file name.
        fs::create_dir_all(&dir_name)?;

        // List of files that don't require special handling.
        let file_paths = [
            ("index.html", &file.content),
            ("manifest.json", &file.json),
            ("robots.txt", &file.txt),
            ("rss.xml", &file.rss),
            ("sitemap.xml", &file.sitemap),
        ];

        // Print the section header.
        println!("\n{}", file_name.to_uppercase());

        // Process each file in the list.
        for (file_name, content) in &file_paths {
            write_and_minify_file(
                &dir_name,
                file_name,
                content,
                index_html_minified
            )?;
        }
    }

    Ok(())
}

// Function to write a file and optionally minify its content.
fn write_and_minify_file(
    dir_path: &Path,
    file_name: &str,
    content: &str,
    minify: bool,
) -> Result<(), Box<dyn Error>> {
    let file_path = dir_path.join(file_name);
    fs::write(&file_path, content)?;

    // Minify "index.html" content if required.
    if minify && file_name == "index.html" {
        let minified_content = minify_html(&file_path)?;
        fs::write(&file_path, minified_content)?;
    }

    // Print the file path.
    println!("  - {}", file_path.display());
    Ok(())
}

// Function to copy a template file to the build directory.
fn copy_template_file(
    template_path: &Path,
    dest_dir: &Path,
    file_name: &str
) -> Result<(), Box<dyn Error>> {
    let dest_path = dest_dir.join(file_name);
    fs::copy(template_path.join(file_name), dest_path)?;

    Ok(())
}
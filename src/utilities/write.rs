// Copyright © 2024 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

// Import the FileData object definition.
use crate::models::data::FileData;

// Import the minify_html function.
use crate::utilities::minification::minify_html;

// Import the Error trait.
use std::error::Error;

// Import the fs module.
use std::fs::{self, copy, read_dir};
use std::path::Path;

// Import the time module.
use std::time::Instant;

/// Constants for other files.
const OTHER_FILES: [&str; 2] = ["main.js", "sw.js"];

/// Constants for index file names.
const INDEX_FILES: [&str; 7] =
    [
        "CNAME",
        "humans.txt",
        "index.html",
        "manifest.json",
        "robots.txt",
        "rss.xml",
        "sitemap.xml",
    ];

/// Writes the files to the build directory.
///
/// # Arguments
///
/// * `build_dir_path` - The path to the build directory.
/// * `file` - The file data object to write.
/// * `template_path` - The path to the template directory.
///
/// # Errors
///
/// Returns an error if any file operation fails.
pub fn write_files_to_build_directory(
    build_dir_path: &Path,
    file: &FileData,
    template_path: &Path,
) -> Result<(), Box<dyn Error>> {
    let start_time = Instant::now();
    let file_name = match (
        Path::new(&file.name).extension().and_then(|s| s.to_str()),
        &file.name,
    ) {
        (Some(ext), name)
            if ["js", "json", "md", "toml", "txt", "xml"]
                .contains(&ext) =>
        {
            name.replace(&format!(".{}", ext), "")
        }
        _ => file.name.to_string(),
    };

    let index_html_minified = file_name == "index";
    let dir_name = build_dir_path.join(&file_name);

    if file_name == "index" {
        for file_name in &INDEX_FILES {
            write_file(
                build_dir_path,
                file_name,
                &get_file_content(file, file_name),
                index_html_minified,
            )?;
        }

        for file_name in &OTHER_FILES {
            copy_template_file(
                template_path,
                build_dir_path,
                file_name,
            )?;
        }
    } else {
        fs::create_dir_all(&dir_name)?;

        for (file_name, content) in &get_file_paths(file) {
            write_file(
                &dir_name,
                file_name,
                content,
                index_html_minified,
            )?;
        }

        print_section_headers(&dir_name, start_time)?;
    }

    Ok(())
}

/// Writes content to a file.
///
/// # Arguments
///
/// * `dir_path` - The path to the directory where the file will be written.
/// * `file_name` - The name of the file.
/// * `content` - The content to write to the file.
/// * `minify` - Indicates whether to minify HTML content.
///
/// # Errors
///
/// Returns an error if writing to the file fails.
fn write_file(
    dir_path: &Path,
    file_name: &str,
    content: &str,
    minify: bool,
) -> Result<(), Box<dyn Error>> {
    let file_path = dir_path.join(file_name);
    fs::write(&file_path, content)?;

    if minify && file_name == "index.html" {
        minify_file(&file_path)?;
    }

    Ok(())
}

/// Minifies a file's content.
///
/// # Arguments
///
/// * `file_path` - The path to the file to minify.
///
/// # Errors
///
/// Returns an error if minification fails or writing to the file fails.
fn minify_file(file_path: &Path) -> Result<(), Box<dyn Error>> {
    let minified_content = minify_html(file_path)?;
    fs::write(file_path, minified_content)?;
    Ok(())
}

/// Copies a template file to a destination directory.
///
/// # Arguments
///
/// * `template_path` - The path to the template directory.
/// * `dest_dir` - The destination directory.
/// * `file_name` - The name of the file to copy.
///
/// # Errors
///
/// Returns an error if copying the file fails.
fn copy_template_file(
    template_path: &Path,
    dest_dir: &Path,
    file_name: &str,
) -> Result<(), Box<dyn Error>> {
    let dest_path = dest_dir.join(file_name);
    copy(template_path.join(file_name), dest_path)?;

    Ok(())
}

/// Retrieves file paths and content from a FileData object.
///
/// # Arguments
///
/// * `file` - The FileData object.
///
/// # Returns
///
/// A vector of tuples containing file paths and content.
fn get_file_paths(file: &FileData) -> Vec<(&'static str, &str)> {
    vec![
        ("index.html", &file.content),
        ("manifest.json", &file.json),
        ("robots.txt", &file.txt),
        ("rss.xml", &file.rss),
        ("sitemap.xml", &file.sitemap),
    ]
}

/// Retrieves content from a FileData object based on file name.
///
/// # Arguments
///
/// * `file` - The FileData object.
/// * `file_name` - The name of the file.
///
/// # Returns
///
/// The content of the file.
fn get_file_content(file: &FileData, file_name: &str) -> String {
    match file_name {
        "CNAME" => file.cname.clone(),
        "humans.txt" => file.human.clone(),
        "index.html" => file.content.clone(),
        "manifest.json" => file.json.clone(),
        "robots.txt" => file.txt.clone(),
        "rss.xml" => file.rss.clone(),
        "sitemap.xml" => file.sitemap.clone(),
        _ => String::new(),
    }
}

/// Prints section headers for a directory and includes timing information.
///
/// # Arguments
///
/// * `dir_path` - The path to the directory.
/// * `start_time` - The instant when the operation started.
///
/// # Errors
///
/// Returns an error if reading the directory fails.
fn print_section_headers(
    dir_path: &Path,
    start_time: Instant,
) -> Result<(), Box<dyn Error>> {
    let mut section_headers = Vec::<String>::new(); // Collect section headers

    for entry in (read_dir(dir_path)?).flatten() {
        let path = entry.path();
        if let Some(file_name) =
            path.file_name().and_then(|s| s.to_str())
        {
            let header = if path.is_dir() {
                file_name.to_uppercase()
            } else {
                format!("  - {}", file_name)
            };
            section_headers.push(header);
        }
    }

    section_headers.sort(); // Sort the section headers alphabetically

    let file_name =
        dir_path.file_name().and_then(|s| s.to_str()).unwrap_or("");
    let duration = start_time.elapsed();
    println!(
        "\n❯ Generating the `{}` directory content ({} microseconds)\n",
        file_name,
        duration.as_micros()
    );
    for header in section_headers {
        println!("{}", header);
    }

    println!("  Done.\n");

    Ok(())
}

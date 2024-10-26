// Copyright © 2024 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Utility functions for directory operations
//!
//! This module provides various functions for working with directories,
//! including creation, cleanup, file discovery, and path manipulation.

use regex::Regex;
use std::{
    error::Error,
    fs, io,
    path::{Path, PathBuf},
};

/// Ensures a directory exists, creating it if necessary.
///
/// # Arguments
///
/// * `dir` - A reference to a `Path` object for the directory.
/// * `name` - A human-readable name for the directory, used in error messages.
///
/// # Returns
///
/// A `Result<String, String>` indicating success or failure.
///
/// # Examples
///
/// ```rust
/// use std::path::Path;
/// use std::fs;
/// use staticrux::utilities::directory::directory;
///
/// let dir = Path::new("logs");
/// match directory(dir, "logs") {
///     Ok(_) => println!("Directory exists or was created successfully"),
///     Err(e) => eprintln!("Error: {}", e),
/// }
///
/// // Ensure the directory is removed after the test
/// if dir.exists() {
///     fs::remove_dir_all(dir).expect("Failed to remove logs directory");
/// }
///
/// assert!(!dir.exists(), "The logs directory should be removed after the test");
/// ```
pub fn directory(dir: &Path, name: &str) -> Result<String, String> {
    if dir.exists() {
        if !dir.is_dir() {
            return Err(format!(
                "❌ Error: {} is not a directory.",
                name
            ));
        }
    } else {
        fs::create_dir_all(dir).map_err(|e| {
            format!("❌ Error: Cannot create {} directory: {}", name, e)
        })?;
    }
    Ok(String::new())
}

/// Moves the output directory to the public directory.
///
/// # Arguments
///
/// * `site_name` - The name of the site.
/// * `out_dir` - A reference to the output directory `Path`.
///
/// # Returns
///
/// An `io::Result<()>` indicating success or failure.
pub fn move_output_directory(
    site_name: &str,
    out_dir: &Path,
) -> io::Result<()> {
    println!("❯ Moving output directory...");

    let public_dir = Path::new("public");

    if public_dir.exists() {
        fs::remove_dir_all(public_dir)?;
    }

    fs::create_dir(public_dir)?;

    let site_name = site_name.replace(' ', "_");
    let new_project_dir = public_dir.join(site_name);
    fs::create_dir_all(&new_project_dir)?;

    fs::rename(out_dir, &new_project_dir)?;

    println!("  Done.\n");

    Ok(())
}

/// Finds all HTML files in a directory and its subdirectories.
///
/// # Arguments
///
/// * `dir` - A reference to the directory `Path` to search.
///
/// # Returns
///
/// An `io::Result<Vec<PathBuf>>` containing paths to all HTML files found.
pub fn find_html_files(dir: &Path) -> io::Result<Vec<PathBuf>> {
    let mut html_files = Vec::new();

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            html_files.extend(find_html_files(&path)?);
        } else if let Some(extension) = path.extension() {
            if extension.eq_ignore_ascii_case("html") {
                html_files.push(path);
            }
        }
    }

    Ok(html_files)
}

/// Cleans up the specified directories.
///
/// # Arguments
///
/// * `directories` - A slice of references to `Path` objects to be cleaned up.
///
/// # Returns
///
/// A `Result<(), Box<dyn Error>>` indicating success or failure.
pub fn cleanup_directory(
    directories: &[&Path],
) -> Result<(), Box<dyn Error>> {
    for directory in directories {
        if !directory.exists() {
            continue;
        }

        println!("\n❯ Cleaning up directories");

        fs::remove_dir_all(directory)?;

        println!("  Done.\n");
    }

    Ok(())
}

/// Creates new directories at the specified paths.
///
/// # Arguments
///
/// * `directories` - A slice of references to `Path` objects to be created.
///
/// # Returns
///
/// A `Result<(), Box<dyn Error>>` indicating success or failure.
pub fn create_directory(
    directories: &[&Path],
) -> Result<(), Box<dyn Error>> {
    for directory in directories {
        if directory.exists() {
            continue;
        }

        fs::create_dir(directory)?;
    }

    Ok(())
}

/// Converts a string to title case.
///
/// # Arguments
///
/// * `s` - A reference to the input string.
///
/// # Returns
///
/// A `String` with the first letter of each word capitalized.
pub fn to_title_case(s: &str) -> String {
    let re = Regex::new(r"(?:^|\s)(\p{L})").unwrap();
    re.replace_all(s, |caps: &regex::Captures| caps[1].to_uppercase())
        .into_owned()
}

/// Formats a header string with an ID and class attribute.
///
/// # Arguments
///
/// * `header_str` - A reference to the header string.
/// * `id_regex` - A reference to a `Regex` object for ID formatting.
///
/// # Returns
///
/// A `String` containing the formatted header.
///
/// # Example
///
/// ```
/// use regex::Regex;
/// use staticrux::utilities::directory::format_header_with_id_class;
///
/// let id_regex = Regex::new(r"[^a-z0-9]+").unwrap();
/// let header = "<h1>Hello World</h1>";
/// let formatted = format_header_with_id_class(header, &id_regex);
/// assert!(formatted.contains("id=\"h1-hello-world\""));
/// ```
pub fn format_header_with_id_class(
    header_str: &str,
    id_regex: &Regex,
) -> String {
    // Match HTML header tags with a named capture group for the tag name
    let re = Regex::new(r"<(?P<tag>\w+)([^>]*)>(?P<content>.+?)</\w+>")
        .unwrap();

    re.replace(header_str, |caps: &regex::Captures| {
        let tag = caps.name("tag")
            .map_or("", |m| m.as_str());
        let attrs = caps.get(2)
            .map_or("", |m| m.as_str());
        let content = caps.name("content")
            .map_or("", |m| m.as_str());

        let binding = content.to_lowercase();
        let id = id_regex.replace_all(&binding, "-");
        let class = id.clone();

        format!(
            r#"<{0}{1} id="{0}-{2}" class="{3}" tabindex="0" aria-label="{4} Heading" {5}>{6}</{0}>"#,
            tag,
            attrs,
            id,
            class,
            to_title_case(content),
            if tag == "h1" { r#"itemprop="headline""# } else { r#"itemprop="name""# },
            content
        )
    }).into_owned()
}

/// Extracts the front matter from the given content.
///
/// # Arguments
///
/// * `content` - A reference to the content string.
///
/// # Returns
///
/// A `&str` slice containing the content without the front matter.
pub fn extract_front_matter(content: &str) -> &str {
    let patterns =
        [("---\n", "\n---\n"), ("+++\n", "\n+++\n"), ("{\n", "\n}\n")];

    for (start, end) in patterns.iter() {
        if content.starts_with(start) {
            if let Some(end_pos) = content.find(end) {
                return &content[end_pos + end.len()..];
            }
            return "";
        }
    }

    content
}

/// Creates and returns a `comrak::ComrakOptions` instance with custom settings.
///
/// # Returns
///
/// A `comrak::ComrakOptions` instance with non-standard Markdown features enabled.
pub fn create_comrak_options() -> comrak::ComrakOptions<'static> {
    let mut options = comrak::ComrakOptions::default();
    options.extension.autolink = true;
    options.extension.description_lists = true;
    options.extension.footnotes = true;
    options.extension.front_matter_delimiter = Some("---".to_owned());
    options.extension.strikethrough = true;
    options.extension.superscript = true;
    options.extension.table = true;
    options.extension.tagfilter = true;
    options.extension.tasklist = true;
    options.parse.smart = true;
    options.render.github_pre_lang = true;
    options.render.hardbreaks = false;
    options.render.unsafe_ = true;
    options
}

/// Updates the 'class' attributes within the provided HTML line.
///
/// # Arguments
///
/// * `line` - A reference to the HTML line string.
/// * `class_regex` - A reference to a `Regex` object for extracting class values.
/// * `img_regex` - A reference to a `Regex` object for identifying image tags.
///
/// # Returns
///
/// An updated `String` with class attributes properly placed.
pub fn update_class_attributes(
    line: &str,
    class_regex: &Regex,
    img_regex: &Regex,
) -> String {
    if line.contains(".class=&quot;") && line.contains("<img") {
        let captures = class_regex.captures(line).unwrap();
        let class_value = captures.get(1).unwrap().as_str();
        let updated_line = class_regex.replace(line, "");
        img_regex
            .replace(
                &updated_line,
                &format!(r#"$1 class="{}" />"#, class_value),
            )
            .into_owned()
    } else {
        line.to_owned()
    }
}

/// Truncates a path to only have a set number of path components.
///
/// # Arguments
///
/// * `path` - The path to truncate.
/// * `length` - The number of path components to keep.
///
/// # Returns
///
/// An `Option<String>` containing the truncated path, or `None` if not truncated.
pub fn truncate(path: &Path, length: usize) -> Option<String> {
    if length == 0 {
        return None;
    }

    let components: Vec<_> =
        path.components().rev().take(length).collect();
    if components.len() == length {
        Some(
            components
                .into_iter()
                .rev()
                .collect::<PathBuf>()
                .to_string_lossy()
                .into_owned(),
        )
    } else {
        None
    }
}

// Copyright © 2024 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! File writing utilities for the static site generator
//!
//! This module provides functionality for writing files to the build directory,
//! including handling various file types, minification, and content generation.

use anyhow::{Context, Result};
use std::fs::{self, copy, read_dir};
use std::path::Path;
use std::time::Instant;

use crate::models::data::FileData;
use html_generator::performance::minify_html;

/// Constants for auxiliary files that should be copied to the build directory
const OTHER_FILES: [&str; 2] = ["main.js", "sw.js"];

/// Constants for index and configuration files
const INDEX_FILES: [&str; 9] = [
    "CNAME",
    "humans.txt",
    "index.html",
    "manifest.json",
    "robots.txt",
    "rss.xml",
    "security.txt",
    "sitemap.xml",
    "news-sitemap.xml",
];

/// Writes the files to the build directory.
///
/// # Arguments
///
/// * `build_dir_path` - The path to the build directory
/// * `file` - The file data object to write
/// * `template_path` - The path to the template directory
///
/// # Returns
///
/// Returns `Ok(())` if successful, or an error if any operation fails
///
/// # Example
///
/// ```no_run
/// use std::path::Path;
/// use staticrux::models::data::FileData;
/// use staticrux::utilities::write::write_files_to_build_directory;
///
/// let build_dir = Path::new("build");
/// let template_dir = Path::new("templates");
/// let file = FileData::default();
///
/// write_files_to_build_directory(build_dir, &file, template_dir)
///     .expect("Failed to write files");
/// ```
pub fn write_files_to_build_directory(
    build_dir_path: &Path,
    file: &FileData,
    template_path: &Path,
) -> Result<()> {
    let start_time = Instant::now();
    let file_name = get_processed_file_name(&file.name);
    let index_html_minified = file_name == "index";
    let dir_name = build_dir_path.join(&file_name);

    if file_name == "index" {
        write_index_files(build_dir_path, file, index_html_minified)
            .context("Failed to write index files")?;

        copy_auxiliary_files(template_path, build_dir_path)
            .context("Failed to copy auxiliary files")?;
    } else {
        write_content_files(&dir_name, file, index_html_minified)
            .context("Failed to write content files")?;

        print_section_headers(&dir_name, start_time)
            .context("Failed to print section headers")?;
    }

    Ok(())
}

/// Gets the processed file name without extension for supported file types
fn get_processed_file_name(original_name: &str) -> String {
    let path = Path::new(original_name);
    match path.extension().and_then(|s| s.to_str()) {
        Some(ext)
            if ["js", "json", "md", "toml", "txt", "xml"]
                .contains(&ext) =>
        {
            path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or(original_name)
                .to_string()
        }
        _ => original_name.to_string(),
    }
}

/// Writes content to a file, with optional HTML minification
///
/// # Arguments
///
/// * `dir_path` - Directory path where the file will be written
/// * `file_name` - Name of the file to write
/// * `content` - Content to write to the file
/// * `minify` - Whether to minify HTML content
///
/// # Returns
///
/// Returns `Ok(())` if successful, or an error if writing fails
fn write_file(
    dir_path: &Path,
    file_name: &str,
    content: &str,
    minify: bool,
) -> Result<()> {
    let file_path = dir_path.join(file_name);
    fs::write(&file_path, content).context("Failed to write file")?;

    if minify && file_name == "index.html" {
        minify_file(&file_path)
            .context("Failed to minify HTML file")?;
    }

    Ok(())
}

/// Minifies an HTML file's content
///
/// # Arguments
///
/// * `file_path` - Path to the file to minify
///
/// # Returns
///
/// Returns `Ok(())` if successful, or an error if minification fails
fn minify_file(file_path: &Path) -> Result<()> {
    let minified_content = minify_html(file_path)
        .context("Failed to minify HTML content")?;
    fs::write(file_path, minified_content)
        .context("Failed to write minified content")?;
    Ok(())
}

/// Copies a template file to a destination directory
///
/// # Arguments
///
/// * `template_path` - Source template directory path
/// * `dest_dir` - Destination directory path
/// * `file_name` - Name of the file to copy
///
/// # Returns
///
/// Returns `Ok(())` if successful, or an error if copying fails
fn copy_template_file(
    template_path: &Path,
    dest_dir: &Path,
    file_name: &str,
) -> Result<()> {
    let dest_path = dest_dir.join(file_name);
    copy(template_path.join(file_name), dest_path)
        .context("Failed to copy template file")?;
    Ok(())
}

/// Gets file paths and content from a FileData object
fn get_file_paths(file: &FileData) -> Vec<(&'static str, &str)> {
    vec![
        ("index.html", &file.content),
        ("manifest.json", &file.json),
        ("robots.txt", &file.txt),
        ("rss.xml", &file.rss),
        ("sitemap.xml", &file.sitemap),
        ("news-sitemap.xml", &file.sitemap_news),
    ]
}

/// Gets content from a FileData object based on file name
fn get_file_content(file: &FileData, file_name: &str) -> String {
    match file_name {
        "CNAME" => file.cname.clone(),
        "humans.txt" => file.human.clone(),
        "index.html" => file.content.clone(),
        "manifest.json" => file.json.clone(),
        "robots.txt" => file.txt.clone(),
        "rss.xml" => file.rss.clone(),
        "security.txt" => file.security.clone(),
        "sitemap.xml" => file.sitemap.clone(),
        "news-sitemap.xml" => file.sitemap_news.clone(),
        _ => String::new(),
    }
}

/// Writes index files to the build directory
fn write_index_files(
    build_dir_path: &Path,
    file: &FileData,
    index_html_minified: bool,
) -> Result<()> {
    for file_name in &INDEX_FILES {
        write_file(
            build_dir_path,
            file_name,
            &get_file_content(file, file_name),
            index_html_minified,
        )
        .with_context(|| {
            format!("Failed to write index file: {}", file_name)
        })?;
    }
    Ok(())
}

/// Copies auxiliary files from template directory to build directory
fn copy_auxiliary_files(
    template_path: &Path,
    build_dir_path: &Path,
) -> Result<()> {
    for file_name in &OTHER_FILES {
        copy_template_file(template_path, build_dir_path, file_name)
            .with_context(|| {
                format!("Failed to copy auxiliary file: {}", file_name)
            })?;
    }
    Ok(())
}

/// Writes content files to the build directory
fn write_content_files(
    dir_name: &Path,
    file: &FileData,
    index_html_minified: bool,
) -> Result<()> {
    fs::create_dir_all(dir_name)
        .context("Failed to create content directory")?;

    for (file_name, content) in &get_file_paths(file) {
        write_file(dir_name, file_name, content, index_html_minified)
            .with_context(|| {
            format!("Failed to write content file: {}", file_name)
        })?;
    }
    Ok(())
}

/// Prints section headers for a directory and includes timing information
fn print_section_headers(
    dir_path: &Path,
    start_time: Instant,
) -> Result<()> {
    let mut section_headers = Vec::new();

    for entry in read_dir(dir_path)
        .context("Failed to read directory")?
        .flatten()
    {
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

    section_headers.sort();

    let file_name =
        dir_path.file_name().and_then(|s| s.to_str()).unwrap_or("");
    let duration = start_time.elapsed();
    println!("\n❯ Generating the `{}` directory content.\n", file_name);
    for header in section_headers {
        println!("{}", header);
    }
    println!("\n❯ Done in {} microseconds.\n", duration.as_micros());

    Ok(())
}

// Copyright © 2024 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

use crate::utilities::backup::backup_file;
use crate::utilities::directory::find_html_files;
use minify_html::{minify, Cfg};
use std::{
    fs::{self, File},
    io::{self, Write},
    path::Path,
};

/// Minifies HTML files in the output directory.
///
/// This function takes a reference to a `Path` object for the output directory
/// and minifies all HTML files in the output directory.
///
/// # Arguments
///
/// * `out_dir` - A reference to a `Path` object for the output directory.
///
/// # Returns
///
/// * `Result<(), std::io::Error>` - A result indicating success or failure.
///     - `Ok(())` if all HTML files were minified successfully.
///     - `Err(std::io::Error)` if any HTML files could not be minified.
///
pub fn minify_html_files(out_dir: &Path) -> io::Result<()> {
    let html_files = find_html_files(out_dir)?;

    for file in &html_files {
        let minified_html = minify_html(file)?;
        let backup_path = backup_file(file)?;
        write_minified_html(file, &minified_html)?;
        println!(
            "Minified HTML file '{}' to '{}'",
            file.display(),
            backup_path.display()
        );
    }

    Ok(())
}

/// Minifies a single HTML file.
///
/// This function takes a reference to a `Path` object for an HTML file and
/// returns a string containing the minified HTML.
///
/// # Arguments
///
/// * `file_path` - A reference to a `Path` object for the HTML file.
///
/// # Returns
///
/// * `Result<String, std::io::Error>` - A result containing a string
///    containing the minified HTML.
///     - `Ok(String)` if the HTML file was minified successfully.
///     - `Err(std::io::Error)` if the HTML file could not be minified.
///
pub fn minify_html(file_path: &Path) -> io::Result<String> {
    let mut cfg = Cfg::new();
    cfg.do_not_minify_doctype = true;
    cfg.ensure_spec_compliant_unquoted_attribute_values = true;
    cfg.keep_closing_tags = true;
    cfg.keep_html_and_head_opening_tags = true;
    cfg.keep_spaces_between_attributes = true;
    cfg.keep_comments = false;
    cfg.minify_css = true;
    cfg.minify_js = true;
    cfg.remove_bangs = true;
    cfg.remove_processing_instructions = true;
    let file_content = fs::read(file_path)?;
    let minified_content = minify(&file_content, &cfg);

    String::from_utf8(minified_content)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

/// Writes a minified HTML file.
///
/// This function takes a reference to a `Path` object for the file to write
/// and a string containing the minified HTML, and writes the minified HTML to
/// the file.
///
/// # Arguments
///
/// * `file_path` - A reference to a `Path` object for the file to write.
/// * `minified_html` - A string containing the minified HTML.
///
/// # Returns
///
/// * `Result<(), std::io::Error>` - A result indicating success or failure.
///     - `Ok(())` if the minified HTML was written successfully.
///     - `Err(std::io::Error)` if the minified HTML could not be written.
///
pub fn write_minified_html(
    file_path: &Path,
    minified_html: &str,
) -> io::Result<()> {
    let mut file = File::create(file_path)?;
    file.write_all(minified_html.as_bytes())?;
    Ok(())
}

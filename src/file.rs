// Copyright © 2023 Shokunin (職人) Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

use quick_xml::escape::escape;
use std::borrow::Cow;
use std::fs;
use std::io;
use std::path::Path;

#[derive(Debug, Default, PartialEq, Eq, Hash, Clone)]
/// File struct to hold the name and content of a file.
pub struct File {
    /// The name of the file.
    pub name: String,
    /// The content of the file.
    pub content: String,
    /// The content of the file, escaped for RSS.
    pub rss: String,
    /// The content of the file, escaped for JSON.
    pub json: String,
    /// The content of the file, escaped for TXT.
    pub txt: String,
    /// The content of the file, escaped for CNAME.
    pub cname: String,
    /// The content of the file, escaped for sitemap.
    pub sitemap: String,
}
/// ## Function: add - returns a Result containing a vector of File structs
///
/// Reads all files in a directory specified by the given path and adds
/// them to a vector. Each file is represented as a `File` struct
/// containing the name and content of the file.
///
/// If an error occurs while reading a file, such as the file not
/// existing or being unreadable, an error is printed to the console
/// and the file is skipped. If all files are read successfully, the
/// function returns a `Vec<File>` containing all the files in the
/// directory.
///
/// # Arguments
///
/// - `path`: A `Path` struct representing the directory containing the
/// files to be read.
///
/// # Returns
///
/// A `Result<Vec<File>, io::Error>` containing a vector of `File`
/// structs representing all files in the directory, or an `io::Error`
/// if the directory cannot be read.
///
pub fn add(path: &Path) -> io::Result<Vec<File>> {
    let mut files = Vec::new();
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            let file_name =
                match path.file_name().and_then(|name| name.to_str()) {
                    Some(name) => name,
                    None => continue,
                };

            if file_name == ".DS_Store" {
                continue;
            }

            let content = match fs::read_to_string(&path) {
                Ok(content) => content,
                Err(_) => continue,
            };

            // Rest of the code remains unchanged

            let rss = match escape(&content) {
                Cow::Borrowed(rss) => rss.to_string(),
                Cow::Owned(rss) => rss,
            };

            let json = match serde_json::to_string(&content) {
                Ok(json) => json,
                Err(_) => continue,
            };

            let txt = match escape(&content) {
                Cow::Borrowed(txt) => txt.to_string(),
                Cow::Owned(txt) => txt,
            };

            let cname = match escape(&content) {
                Cow::Borrowed(cname) => cname.to_string(),
                Cow::Owned(cname) => cname,
            };

            let sitemap = match escape(&content) {
                Cow::Borrowed(sitemap) => sitemap.to_string(),
                Cow::Owned(sitemap) => sitemap,
            };

            files.push(File {
                name: file_name.to_string(),
                content,
                rss,
                json,
                txt,
                cname,
                sitemap,
            });
        }
    }
    Ok(files)
}

// Copyright © 2024 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

use crate::models::data::FileData;
use quick_xml::escape::escape;
use std::{fs, io, path::Path};

/// Reads all files in a directory specified by the given path and returns a vector of FileData.
///
/// Each file is represented as a `FileData` struct containing the name and content of the file.
///
/// # Arguments
///
/// * `path` - A `Path` representing the directory containing the files to be read.
///
/// # Returns
///
/// A `Result` containing a vector of `FileData` structs representing all files in the directory,
/// or an `io::Error` if the directory cannot be read.
pub fn add(path: &Path) -> io::Result<Vec<FileData>> {
    let files = fs::read_dir(path)?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.is_file() {
                let file_name =
                    path.file_name()?.to_string_lossy().to_string();
                if file_name == ".DS_Store" {
                    return None;
                }
                let content = fs::read_to_string(&path)
                    .map_err(|e| {
                        eprintln!(
                            "Error reading file {:?}: {}",
                            path, e
                        );
                        e
                    })
                    .ok()?;
                Some((file_name, content))
            } else {
                None
            }
        })
        .map(|(file_name, content)| {
            let rss = escape(&content).to_string();
            let json =
                serde_json::to_string(&content).unwrap_or_else(|e| {
                    eprintln!(
                        "Error serializing JSON for file {}: {}",
                        file_name, e
                    );
                    String::new()
                });
            let cname = escape(&content).to_string();
            let keyword = escape(&content).to_string();
            let human = content.clone();
            let security = content.clone();
            let sitemap = escape(&content).to_string();
            let sitemap_news = escape(&content).to_string();
            let txt = content.clone();

            FileData {
                cname,
                content,
                json,
                human,
                keyword,
                name: file_name,
                rss,
                security,
                sitemap,
                sitemap_news,
                // tags,
                txt,
            }
        })
        .collect::<Vec<FileData>>();

    Ok(files)
}

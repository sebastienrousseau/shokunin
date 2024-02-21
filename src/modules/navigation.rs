// Copyright © 2024 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

use crate::models::data::FileData;
use crate::utilities::directory::to_title_case;
use std::collections::{BTreeMap, HashSet};
use std::fmt::Write;
use std::path::Path;

// Define supported file extensions as a lazy static HashSet
lazy_static::lazy_static! {
    static ref SUPPORTED_EXTENSIONS: HashSet<&'static str> = {
        let mut set = HashSet::new();
        set.insert("md");
        set.insert("toml");
        set.insert("json");
        set
    };
}

/// Struct representing the components of a file name
struct FileNameComponents {
    file_stem: String, // The stem of the file name (without extension)
    file_extension: Option<String>, // The extension of the file name (if any)
}

impl FileNameComponents {
    /// Constructs FileNameComponents from a given Path
    fn from_path(path: &Path) -> Self {
        let file_stem = path
            .file_stem()
            .and_then(|n| n.to_str()) // Convert OsStr to &str
            .unwrap_or_else(|| path.to_str().unwrap_or("")) // Handle non-Unicode file names
            .to_string(); // Convert &str to String
        let file_extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(String::from);
        FileNameComponents {
            file_stem,
            file_extension,
        }
    }

    /// Gets the file name without the extension
    fn file_name(&self) -> &str {
        match &self.file_extension {
            Some(ext)
                if SUPPORTED_EXTENSIONS.contains(ext.as_str()) =>
            {
                &self.file_stem
            }
            _ => &self.file_stem,
        }
    }
}

/// Struct responsible for generating navigation HTML.
#[derive(Debug)]
pub struct NavigationGenerator;

impl NavigationGenerator {
    /// Generates a navigation menu as an unordered list of links.
    ///
    /// # Arguments
    ///
    /// * `files` - A slice of `FileData` structs containing the compiled HTML files.
    ///
    /// # Returns
    ///
    /// A string containing the HTML code for the navigation menu.
    ///
    /// The HTML code is wrapped in a `<ul>` element with the class `navbar-nav`.
    /// Each file is wrapped in a `<li>` element, and each link is wrapped
    /// in an `<a>` element.
    pub fn generate_navigation(files: &[FileData]) -> String {
        // Check if there are files
        if files.is_empty() {
            return String::new(); // Return an empty string if there are no files
        }

        // Filter supported files
        let files_supported: Vec<&FileData> = files
            .iter()
            .filter(|file| {
                let extension = Path::new(&file.name)
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .unwrap_or("");
                SUPPORTED_EXTENSIONS.contains(&extension)
            })
            .collect();

        // Check if there are supported files
        if files_supported.is_empty() {
            return String::new(); // Return an empty string if there are no supported files
        }

        // Map file names to URLs
        let mut file_links = BTreeMap::new();
        for file in files_supported {
            let path = Path::new(&file.name);
            let file_components = FileNameComponents::from_path(path);
            let file_name = file_components.file_name();

            if !["index", "404", "privacy", "terms", "offline"]
                .contains(&file_name)
            {
                let url = format!(
                    "/{}/index.html",
                    path.with_extension("").display()
                );
                file_links.insert(file_name.to_string(), url);
            }
        }

        // Generate navigation links
        let mut nav_links = String::new();
        for (file_name, url) in file_links {
            write!(
                &mut nav_links,
                "<li class=\"nav-item\"><a aria-label=\"{}\" href=\"{}\" title=\"Navigation link for the {} page\" class=\"text-uppercase p-2 \">{}</a></li>",
                file_name,
                url,
                to_title_case(&file_name),
                &file_name,
            )
            .unwrap_or_else(|e| {
                eprintln!("Error writing navigation link: {}", e);
            });
        }

        // Format navigation links into a HTML unordered list
        format!(
            "<ul class=\"navbar-nav ms-auto mb-2 mb-lg-0\">\n{}</ul>",
            nav_links
        )
    }
}

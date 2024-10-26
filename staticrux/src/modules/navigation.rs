// Copyright © 2024 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Navigation Menu Generation Module
//!
//! This module handles the generation of navigation menus for the static site.
//! It processes files and directories to create accessible, semantic HTML
//! navigation structures.
//!
//! # Features
//! - Automatic navigation menu generation from content files
//! - Accessibility support with ARIA attributes
//! - Semantic HTML structure
//! - Support for multiple file types
//! - URL path normalization
//! - Title case conversion for display
//!
//! # Example
//! ```
//! use staticrux::models::data::FileData;
//! use staticrux::modules::navigation::NavigationGenerator;
//!
//! let files = vec![
//!     FileData {
//!         name: "about.md".to_string(),
//!         content: "About page content".to_string(),
//!         ..Default::default()
//!     }
//! ];
//!
//! let nav = NavigationGenerator::generate_navigation(&files);
//! ```

use crate::models::data::FileData;
use crate::utilities::directory::to_title_case;
use std::collections::BTreeMap;
use std::path::Path;

/// Set of supported file extensions for navigation
const SUPPORTED_EXTENSIONS: [&str; 3] = ["md", "toml", "json"];

/// File patterns to exclude from navigation
const EXCLUDED_FILES: [&str; 5] =
    ["index", "404", "privacy", "terms", "offline"];

/// Navigation menu generator
///
/// Handles the creation of navigation menus from content files.
#[derive(Debug, Clone, Copy)]
pub struct NavigationGenerator;

impl NavigationGenerator {
    /// Generates a navigation menu as an unordered list of links.
    ///
    /// Creates an accessible navigation structure with proper HTML semantics
    /// and ARIA attributes.
    ///
    /// # Arguments
    /// * `files` - Slice of FileData containing compiled files
    ///
    /// # Returns
    /// * `String` - Generated HTML navigation menu
    ///
    /// # Example
    /// ```
    /// use staticrux::models::data::FileData;
    /// use staticrux::modules::navigation::NavigationGenerator;
    ///
    /// let files = vec![FileData {
    ///     name: "about.md".to_string(),
    ///     content: "About page".to_string(),
    ///     ..Default::default()
    /// }];
    ///
    /// let nav = NavigationGenerator::generate_navigation(&files);
    /// assert!(nav.contains("about"));
    /// ```
    pub fn generate_navigation(files: &[FileData]) -> String {
        if files.is_empty() {
            return String::new();
        }

        // Filter supported files and collect into navigation items
        let nav_items = Self::collect_nav_items(files);
        if nav_items.is_empty() {
            return String::new();
        }

        // Generate HTML with semantic structure and accessibility attributes
        Self::build_nav_html(&nav_items)
    }

    /// Collects and processes navigation items from files.
    fn collect_nav_items(
        files: &[FileData],
    ) -> BTreeMap<String, String> {
        let mut nav_items = BTreeMap::new();

        for file in files {
            if let Some(nav_item) = Self::process_file(file) {
                nav_items.insert(nav_item.0, nav_item.1);
            }
        }

        nav_items
    }

    /// Processes a single file for navigation.
    fn process_file(file: &FileData) -> Option<(String, String)> {
        let path = Path::new(&file.name);

        // Check file extension
        let extension =
            path.extension().and_then(|ext| ext.to_str())?;

        if !SUPPORTED_EXTENSIONS.contains(&extension) {
            return None;
        }

        // Get file stem (filename without extension)
        let file_stem =
            path.file_stem().and_then(|stem| stem.to_str())?;

        // Skip excluded files
        if EXCLUDED_FILES.contains(&file_stem) {
            return None;
        }

        // Generate URL and display name
        let url = format!(
            "/{}/index.html",
            path.with_extension("").display()
        );
        // Convert hyphenated-names to "Hyphenated Names"
        let display_name = file_stem
            .replace('-', " ")
            .split_whitespace()
            .map(to_title_case)
            .collect::<Vec<_>>()
            .join(" ");

        Some((display_name, url))
    }

    /// Builds HTML navigation structure.
    fn build_nav_html(nav_items: &BTreeMap<String, String>) -> String {
        let mut nav_links =
            String::with_capacity(nav_items.len() * 100);

        for (name, url) in nav_items {
            nav_links.push_str(&format!(
                r#"<li class="nav-item"><a aria-label="{}" href="{}" title="Navigation link for the {} page" class="text-uppercase p-2">{}</a></li>"#,
                name, url, name, name
            ));
        }

        format!(
            r#"<ul class="navbar-nav ms-auto mb-2 mb-lg-0">{}</ul>"#,
            nav_links
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_navigation() {
        let nav = NavigationGenerator::generate_navigation(&[]);
        assert!(nav.is_empty());
    }

    #[test]
    fn test_single_file_navigation() {
        let files = vec![FileData {
            name: "about.md".to_string(),
            content: "About page".to_string(),
            ..Default::default()
        }];

        let nav = NavigationGenerator::generate_navigation(&files);
        assert!(nav.contains("About"));
        assert!(nav.contains("about/index.html"));
        assert!(nav.contains(r#"class="nav-item"#));
        assert!(nav.contains(r#"aria-label="About""#));
    }

    #[test]
    fn test_multiple_files_navigation() {
        let files = vec![
            FileData {
                name: "about.md".to_string(),
                content: "About".to_string(),
                ..Default::default()
            },
            FileData {
                name: "blog.md".to_string(),
                content: "Blog".to_string(),
                ..Default::default()
            },
        ];

        let nav = NavigationGenerator::generate_navigation(&files);
        assert!(nav.contains("About"));
        assert!(nav.contains("Blog"));
        assert!(nav.contains("about/index.html"));
        assert!(nav.contains("blog/index.html"));
    }

    #[test]
    fn test_excluded_files() {
        let files = vec![
            FileData {
                name: "index.md".to_string(),
                content: "Home".to_string(),
                ..Default::default()
            },
            FileData {
                name: "404.md".to_string(),
                content: "Not Found".to_string(),
                ..Default::default()
            },
            FileData {
                name: "about.md".to_string(),
                content: "About".to_string(),
                ..Default::default()
            },
        ];

        let nav = NavigationGenerator::generate_navigation(&files);
        assert!(!nav.contains("index/"));
        assert!(!nav.contains("404/"));
        assert!(nav.contains("about/"));
    }

    #[test]
    fn test_unsupported_extensions() {
        let files = vec![FileData {
            name: "document.txt".to_string(),
            content: "Text file".to_string(),
            ..Default::default()
        }];

        let nav = NavigationGenerator::generate_navigation(&files);
        assert!(!nav.contains("document"));
    }

    #[test]
    fn test_title_case_conversion() {
        let files = vec![
            FileData {
                name: "about-us.md".to_string(),
                content: "About Us".to_string(),
                ..Default::default()
            },
            FileData {
                name: "contact-me.md".to_string(),
                content: "Contact Me".to_string(),
                ..Default::default()
            },
        ];

        let nav = NavigationGenerator::generate_navigation(&files);

        // Test both title case conversion and hyphen handling
        assert!(
            nav.contains(">About Us<"),
            "Navigation should contain 'About Us'"
        );
        assert!(
            nav.contains(">Contact Me<"),
            "Navigation should contain 'Contact Me'"
        );

        // Test URL format
        assert!(
            nav.contains("href=\"/about-us/index.html\""),
            "URL should maintain hyphens"
        );
        assert!(
            nav.contains("href=\"/contact-me/index.html\""),
            "URL should maintain hyphens"
        );
    }

    #[test]
    fn test_nav_structure() {
        let files = vec![FileData {
            name: "about.md".to_string(),
            content: "About".to_string(),
            ..Default::default()
        }];

        let nav = NavigationGenerator::generate_navigation(&files);
        assert!(nav.starts_with(r#"<ul class="navbar-nav"#));
        assert!(nav.contains(r#"<li class="nav-item">"#));
        assert!(nav.ends_with("</ul>"));
    }
}

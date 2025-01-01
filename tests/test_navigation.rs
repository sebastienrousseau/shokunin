//! Tests for the `NavigationGenerator` module in staticdatagen,
//! ensuring navigation data structures and generation work correctly.

// Copyright © 2025 Shokunin Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

#[cfg(test)]
mod tests {
    use staticdatagen::models::data::FileData;
    use staticdatagen::modules::navigation::NavigationGenerator;

    #[test]
    fn test_generate_navigation_empty_input() {
        // Test with empty file list
        let files = vec![];
        let navigation =
            NavigationGenerator::generate_navigation(&files);
        assert!(
            navigation.is_empty(),
            "Navigation should be empty for empty file list"
        );
    }

    #[test]
    fn test_generate_navigation_single_file() {
        let files = vec![FileData {
            name: "page.md".to_string(),
            content: "Test content".to_string(),
            ..Default::default()
        }];

        let navigation =
            NavigationGenerator::generate_navigation(&files);
        assert!(
            !navigation.is_empty(),
            "Navigation should not be empty for single file"
        );
        // The navigation generator appears to create additional entries for structure
        assert!(
            !navigation.is_empty(),
            "Navigation should contain at least one entry"
        );
    }

    #[test]
    fn test_generate_navigation_multiple_files() {
        let files = vec![
            FileData {
                name: "page1.md".to_string(),
                content: "Content 1".to_string(),
                ..Default::default()
            },
            FileData {
                name: "page2.md".to_string(),
                content: "Content 2".to_string(),
                ..Default::default()
            },
            FileData {
                name: "page3.md".to_string(),
                content: "Content 3".to_string(),
                ..Default::default()
            },
        ];

        let navigation =
            NavigationGenerator::generate_navigation(&files);
        assert!(navigation.len() > 3, "Navigation should contain entries for all files plus structure");
    }

    #[test]
    fn test_generate_navigation_nested_structure() {
        let files = vec![
            FileData {
                name: "docs/guide/getting-started.md".to_string(),
                content: "Getting Started Guide".to_string(),
                ..Default::default()
            },
            FileData {
                name: "docs/api/reference.md".to_string(),
                content: "API Reference".to_string(),
                ..Default::default()
            },
        ];

        let navigation =
            NavigationGenerator::generate_navigation(&files);
        assert!(
            !navigation.is_empty(),
            "Navigation should handle nested paths"
        );
    }

    #[test]
    fn test_generate_navigation_with_index_files() {
        let files = vec![
            FileData {
                name: "index.md".to_string(),
                content: "Home page".to_string(),
                ..Default::default()
            },
            FileData {
                name: "docs/index.md".to_string(),
                content: "Documentation index".to_string(),
                ..Default::default()
            },
        ];

        let navigation =
            NavigationGenerator::generate_navigation(&files);
        assert!(
            !navigation.is_empty(),
            "Navigation should handle index files"
        );
    }

    #[test]
    fn test_generate_navigation_with_mixed_content() {
        let files = vec![
            FileData {
                name: "index.md".to_string(),
                content: "Home".to_string(),
                ..Default::default()
            },
            FileData {
                name: "about.html".to_string(),
                content: "<h1>About</h1>".to_string(),
                ..Default::default()
            },
            FileData {
                name: "posts/post1.md".to_string(),
                content: "First post".to_string(),
                ..Default::default()
            },
        ];

        let navigation =
            NavigationGenerator::generate_navigation(&files);
        assert!(
            !navigation.is_empty(),
            "Navigation should handle mixed content types"
        );
    }

    #[test]
    fn test_generate_navigation_with_invalid_paths() {
        let files = vec![
            FileData {
                name: "valid.md".to_string(),
                content: "Valid content".to_string(),
                ..Default::default()
            },
            FileData {
                name: "../invalid.md".to_string(), // Path traversal attempt
                content: "Invalid content".to_string(),
                ..Default::default()
            },
        ];

        let navigation =
            NavigationGenerator::generate_navigation(&files);
        // Should still generate navigation but handle invalid paths appropriately
        assert!(
            !navigation.is_empty(),
            "Navigation should handle invalid paths safely"
        );
    }

    #[test]
    fn test_generate_navigation_duplicate_names() {
        let files = vec![
            FileData {
                name: "docs/guide.md".to_string(),
                content: "Guide 1".to_string(),
                ..Default::default()
            },
            FileData {
                name: "tutorials/guide.md".to_string(),
                content: "Guide 2".to_string(),
                ..Default::default()
            },
        ];

        let navigation =
            NavigationGenerator::generate_navigation(&files);
        assert!(
            !navigation.is_empty(),
            "Navigation should handle duplicate file names"
        );
    }

    #[test]
    fn test_generate_navigation_with_empty_content() {
        let files = vec![
            FileData {
                name: "empty.md".to_string(),
                content: "".to_string(),
                ..Default::default()
            },
            FileData {
                name: "nonempty.md".to_string(),
                content: "Content".to_string(),
                ..Default::default()
            },
        ];

        let navigation =
            NavigationGenerator::generate_navigation(&files);
        assert!(
            !navigation.is_empty(),
            "Navigation should handle empty content files"
        );
    }

    #[test]
    fn test_generate_navigation_stress_test_large_paths() {
        let mut files = vec![];
        for i in 0..100 {
            files.push(FileData {
                name: format!(
                    "very/deeply/nested/path/structure/file{}.md",
                    i
                ),
                content: format!("Content {}", i),
                ..Default::default()
            });
        }

        let navigation =
            NavigationGenerator::generate_navigation(&files);
        assert!(
            !navigation.is_empty(),
            "Navigation should handle deeply nested paths"
        );
    }

    #[test]
    fn test_generate_navigation_with_special_filenames() {
        let files = vec![
            FileData {
                name: "README.md".to_string(),
                content: "Readme content".to_string(),
                ..Default::default()
            },
            FileData {
                name: ".hidden.md".to_string(),
                content: "Hidden content".to_string(),
                ..Default::default()
            },
            FileData {
                name: "_draft.md".to_string(),
                content: "Draft content".to_string(),
                ..Default::default()
            },
        ];

        let navigation =
            NavigationGenerator::generate_navigation(&files);
        assert!(
            !navigation.is_empty(),
            "Navigation should handle special filenames"
        );
    }
}

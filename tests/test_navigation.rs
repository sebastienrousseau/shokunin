// Copyright © 2024 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

#[cfg(test)]
mod tests {
    use staticdatagen::models::data::FileData;
    use staticdatagen::modules::navigation::NavigationGenerator;

    #[test]
    fn test_generate_navigation_empty_input() {
        // Arrange
        let files: Vec<FileData> = vec![];

        // Act
        let navigation =
            NavigationGenerator::generate_navigation(&files);

        // Assert
        assert!(
            navigation.is_empty(),
            "Navigation is not empty for empty input"
        );
    }

    #[test]
    fn test_generate_navigation_non_markdown_file() {
        // Arrange
        let files = vec![FileData {
            name: "index.txt".to_string(),
            ..Default::default()
        }];

        // Act
        let navigation =
            NavigationGenerator::generate_navigation(&files);

        // Assert
        assert!(
            navigation.is_empty(),
            "Navigation is not empty for non-markdown file"
        );
    }

    #[test]
    fn test_generate_navigation_stress_test() {
        // Arrange
        let mut files = vec![];
        for i in 0..100 {
            files.push(FileData {
                name: format!("page{}.md", i),
                ..Default::default()
            });
        }

        // Act
        let navigation =
            NavigationGenerator::generate_navigation(&files);

        // Assert
        assert!(
            !navigation.is_empty(),
            "Navigation is empty for stress test"
        );
    }

    #[test]
    fn test_generate_navigation_special_characters() {
        // Arrange
        let files = vec![FileData {
            name: "special!@#$%^&*()-_+=[]{}|;:'\",.<>?`~.md"
                .to_string(),
            ..Default::default()
        }];

        // Act
        let navigation =
            NavigationGenerator::generate_navigation(&files);

        // Assert
        assert!(
            !navigation.is_empty(),
            "Navigation is empty for file with special characters"
        );
    }

    #[test]
    fn test_generate_navigation_empty_string_file_name() {
        // Arrange
        let files = vec![FileData {
            name: "".to_string(),
            ..Default::default()
        }];

        // Act
        let navigation =
            NavigationGenerator::generate_navigation(&files);

        // Assert
        assert!(
            navigation.is_empty(),
            "Navigation is not empty for file with empty string name"
        );
    }

    #[test]
    fn test_generate_navigation_no_extension() {
        // Arrange
        let files = vec![FileData {
            name: "file_without_extension".to_string(),
            ..Default::default()
        }];

        // Act
        let navigation =
            NavigationGenerator::generate_navigation(&files);

        // Assert
        assert!(
            navigation.is_empty(),
            "Navigation is not empty for file with no extension"
        );
    }
}

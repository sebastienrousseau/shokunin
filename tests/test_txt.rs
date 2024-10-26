// Copyright © 2024 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

#[cfg(test)]
mod tests {
    use staticrux::modules::robots::create_txt_data;
    use std::collections::HashMap;

    /// Test for creating TxtData with a valid permalink.
    #[test]
    fn test_create_txt_data_with_permalink() {
        // Arrange
        let mut metadata = HashMap::new();
        let _ = metadata.insert(
            "permalink".to_string(),
            "https://example.com".to_string(),
        );

        // Act
        let txt_data = create_txt_data(&metadata);

        // Assert
        assert_eq!(txt_data.permalink, "https://example.com");
    }

    /// Test for creating TxtData with missing permalink.
    #[test]
    fn test_create_txt_data_with_missing_permalink() {
        // Arrange
        let metadata = HashMap::new(); // Empty metadata

        // Act
        let txt_data = create_txt_data(&metadata);

        // Assert
        assert_eq!(txt_data.permalink, "");
    }

    /// Test for creating TxtData with invalid UTF-8 metadata.
    #[test]
    fn test_create_txt_data_with_invalid_utf8_metadata() {
        // Arrange
        let mut metadata = HashMap::new();
        let _ = metadata.insert(
            "permalink".to_string(),
            "https://example.com".to_string(),
        );
        let _ = metadata.insert(
            "title".to_string(),
            "Invalid UTF-8: \u{FFFD}".to_string(),
        );

        // Act
        let txt_data = create_txt_data(&metadata);

        // Assert
        assert_eq!(txt_data.permalink, "https://example.com");
    }

    /// Test for creating TxtData with empty metadata.
    #[test]
    fn test_create_txt_data_with_empty_metadata() {
        // Arrange
        let metadata = HashMap::new(); // Empty metadata

        // Act
        let txt_data = create_txt_data(&metadata);

        // Assert
        assert_eq!(txt_data.permalink, "");
    }
}

// Copyright © 2025 Shokunin Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! This crate tests human data generation functionality using `HumansGenerator`.

#[cfg(test)]
mod tests {
    use staticdatagen::generators::humans::{
        HumansConfig, HumansGenerator,
    };
    use std::collections::HashMap;

    #[test]
    fn test_create_human_data_with_all_fields() {
        let mut metadata = HashMap::new();
        let _ = metadata.insert(
            "author_location".to_string(),
            "Location".to_string(),
        );
        let _ = metadata.insert(
            "author_twitter".to_string(),
            "@twitter_handle".to_string(),
        );
        let _ = metadata.insert(
            "author_website".to_string(),
            "https://example.com".to_string(),
        );
        let _ = metadata
            .insert("author".to_string(), "John Doe".to_string());
        let _ = metadata.insert(
            "site_components".to_string(),
            "Components".to_string(),
        );
        let _ = metadata.insert(
            "site_last_updated".to_string(),
            "2023-01-01".to_string(),
        );
        let _ = metadata.insert(
            "site_software".to_string(),
            "Software".to_string(),
        );
        let _ = metadata.insert(
            "site_standards".to_string(),
            "Standards".to_string(),
        );
        let _ = metadata
            .insert("thanks".to_string(), "Contributors".to_string());

        let config = HumansConfig::from_metadata(&metadata)
            .expect("Expected valid config from full metadata");
        let generated = HumansGenerator::new(config).generate();

        // Check that all expected values are present in the generated output
        assert!(
            generated.contains("John Doe"),
            "Expected 'John Doe' in output"
        );
        assert!(
            generated.contains("Location"),
            "Expected 'Location' in output"
        );
        assert!(
            generated.contains("@twitter_handle"),
            "Expected '@twitter_handle' in output"
        );
        assert!(
            generated.contains("https://example.com"),
            "Expected 'https://example.com' in output"
        );
        assert!(
            generated.contains("Components"),
            "Expected 'Components' in output"
        );
        assert!(
            generated.contains("2023-01-01"),
            "Expected '2023-01-01' in output"
        );
        assert!(
            generated.contains("Software"),
            "Expected 'Software' in output"
        );
        assert!(
            generated.contains("Standards"),
            "Expected 'Standards' in output"
        );
        assert!(
            generated.contains("Contributors"),
            "Expected 'Contributors' in output"
        );
    }
}

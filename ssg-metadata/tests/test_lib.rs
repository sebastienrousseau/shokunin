#[cfg(test)]
mod tests {
    use ssg_metadata::{
        self, extract_and_prepare_metadata, MetadataError,
    };

    /// Test the `extract_and_prepare_metadata` function with valid content.
    ///
    /// This test ensures that metadata extraction, keyword generation, and meta tag creation
    /// work correctly for valid input content.
    #[test]
    fn test_extract_and_prepare_metadata_valid() {
        let content = r#"---
title: "My Page"
description: "A sample page"
keywords: "rust, static site generator, metadata"
---
# Content goes here
"#;

        let result = extract_and_prepare_metadata(content);
        assert!(
            result.is_ok(),
            "Metadata extraction should succeed for valid content"
        );

        let (metadata_map, keywords, meta_tags) = result.unwrap();

        // Ensure metadata is correctly extracted
        assert_eq!(
            metadata_map.get("title"),
            Some(&"My Page".to_string()),
            "Title metadata should be extracted correctly"
        );
        assert_eq!(
            metadata_map.get("description"),
            Some(&"A sample page".to_string()),
            "Description metadata should be extracted correctly"
        );

        // Ensure keywords are correctly generated
        assert_eq!(
            keywords,
            vec!["rust", "static site generator", "metadata"],
            "Keywords should be extracted correctly"
        );

        // Ensure meta tags contain the correct description
        assert!(
            meta_tags.primary.contains("description"),
            "Primary meta tags should contain 'description'"
        );
    }

    /// Test the `extract_and_prepare_metadata` function with missing metadata.
    ///
    /// This test ensures that missing metadata fields are handled gracefully.
    #[test]
    fn test_extract_and_prepare_metadata_missing_metadata() {
        let content = r#"---
title: "No Description"
---
# Content goes here
"#;

        let result = extract_and_prepare_metadata(content);
        assert!(result.is_ok(), "Metadata extraction should succeed even with missing fields");

        let (metadata_map, keywords, meta_tags) = result.unwrap();

        // Ensure title is correctly extracted, even if description is missing
        assert_eq!(
            metadata_map.get("title"),
            Some(&"No Description".to_string()),
            "Title metadata should be extracted correctly"
        );
        assert_eq!(
            keywords.len(),
            0,
            "No keywords should be extracted if none are provided"
        );

        // Ensure the description is absent from the meta tags
        assert!(
            !meta_tags.primary.contains("description"),
            "Primary meta tags should not contain 'description'"
        );
    }

    /// Test the `extract_and_prepare_metadata` function with invalid front matter format.
    ///
    /// This test checks whether the function returns an appropriate error when the front matter is malformed.
    #[test]
    fn test_extract_and_prepare_metadata_invalid_format() {
        let content = r#"---
title My Page
description A sample page
---
# Content goes here
"#;

        let result = extract_and_prepare_metadata(content);
        assert!(
            result.is_err(),
            "Invalid front matter format should result in an error"
        );

        // Ensure the error is of type MetadataError::ExtractionError
        if let Err(MetadataError::ExtractionError(_)) = result {
            // This is the expected error
        } else {
            panic!("Expected MetadataError::ExtractionError");
        }
    }
}

#[cfg(test)]
mod integration_tests {
    use ssg_metadata::error::MetadataError;
    use ssg_metadata::metadata::extract_metadata;
    use ssg_metadata::metatags::generate_metatags;
    use ssg_metadata::utils::escape_html_entities;

    /// Integration test: Metadata extraction and meta tag generation.
    ///
    /// This test verifies that metadata extraction from content works correctly and meta tags are generated properly.
    #[test]
    fn test_metadata_and_metatags_integration() {
        let content = r#"
---
title: "Integration Test Page"
description: "This is a page for integration testing."
keywords: "integration, test, metadata"
---
# Content for integration testing.
"#;

        // Extract metadata from content
        let metadata = extract_metadata(content)
            .expect("Failed to extract metadata");

        // Verify extracted metadata
        assert_eq!(
            metadata.get("title"),
            Some(&"Integration Test Page".to_string())
        );
        assert_eq!(
            metadata.get("description"),
            Some(
                &"This is a page for integration testing.".to_string()
            )
        );
        assert_eq!(
            metadata.get("keywords"),
            Some(&"integration, test, metadata".to_string())
        );

        // Generate meta tags from the extracted metadata
        let metatags = generate_metatags(&metadata.into_inner());

        // Verify the generated meta tags
        assert!(metatags.primary.contains("description"));
        assert!(metatags.primary.contains("keywords"));
    }

    /// Integration test: HTML escaping and metadata processing.
    ///
    /// This test ensures that HTML content is properly escaped and metadata is processed correctly.
    #[test]
    fn test_html_escaping_and_metadata() {
        let html_content = r#"
---
title: "Escaping Test"
description: "<script>alert('test');</script>"
keywords: "escape, html, test"
---
# Content for escaping test.
"#;

        // Extract metadata from content
        let metadata = extract_metadata(html_content)
            .expect("Failed to extract metadata");

        // Escape HTML characters in metadata fields
        let escaped_description =
            escape_html_entities(metadata.get("description").unwrap());

        // Verify that HTML in the description is escaped
        assert_eq!(
            escaped_description,
            "&lt;script&gt;alert(&#x27;test&#x27;);&lt;/script&gt;"
        );
    }

    /// Integration test: Metadata extraction and error handling.
    ///
    /// This test checks that an invalid front matter format results in an appropriate error.
    #[test]
    fn test_metadata_extraction_error_handling() {
        let invalid_content = r#"
---
title Integration Test Page
description: This is an invalid front matter format.
---
# Content for invalid test.
"#;

        // Try to extract metadata from invalid content
        let result = extract_metadata(invalid_content);

        // Verify that an error is returned
        assert!(result.is_err());

        // Check for the specific type of error (MetadataError::ExtractionError)
        if let Err(MetadataError::ExtractionError(_)) = result {
            // Expected error
        } else {
            panic!("Expected MetadataError::ExtractionError");
        }
    }

    /// Integration test: Metadata extraction from file and meta tag generation.
    ///
    /// This async test ensures that metadata can be extracted from a file and meta tags generated correctly.
    #[tokio::test]
    async fn test_async_metadata_and_metatags_integration() {
        use tempfile::tempdir;
        use tokio::fs::File;
        use tokio::io::AsyncWriteExt;

        // Create a temporary file with some content
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test_async.md");
        let mut file = File::create(&file_path).await.unwrap();
        let content = r#"
---
title: "Async Test Page"
description: "This is an async test for metadata extraction."
keywords: "async, test, metadata"
---
# Async Test Content
"#;
        file.write_all(content.as_bytes()).await.unwrap();

        // Test the async_extract_metadata_from_file function
        let result =
            ssg_metadata::utils::async_extract_metadata_from_file(
                file_path.to_str().unwrap(),
            )
            .await;
        assert!(result.is_ok());

        let (metadata, keywords, meta_tags) = result.unwrap();
        assert_eq!(
            metadata.get("title"),
            Some(&"Async Test Page".to_string())
        );
        assert_eq!(
            metadata.get("description"),
            Some(
                &"This is an async test for metadata extraction."
                    .to_string()
            )
        );
        assert_eq!(keywords, vec!["async", "test", "metadata"]);
        assert!(!meta_tags.primary.is_empty());
    }
}

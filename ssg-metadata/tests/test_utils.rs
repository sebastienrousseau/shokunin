//! Unit tests for the `utils` module.
//!
//! This module tests utility functions such as string manipulation and validation.

#[cfg(test)]
mod tests {
    use ssg_metadata::async_extract_metadata_from_file;
    use ssg_metadata::utils::escape_html_entities;

    /// Test if string escaping works as expected.
    ///
    /// This test ensures that the escaping function properly handles unsafe characters.
    #[test]
    fn test_string_escaping() {
        let input = "<script>alert('test');</script>";
        let escaped = escape_html_entities(input);
        let expected =
            "&lt;script&gt;alert(&#x27;test&#x27;);&lt;/script&gt;"; // No forward slash escaping
        assert_eq!(escaped, expected, "The escaped HTML entities should match the expected result.");
    }

    /// Test for invalid input in the utility function.
    ///
    /// This test checks how the utility function handles invalid or unsafe input.
    #[test]
    fn test_invalid_input_handling() {
        let input = "";
        let result = escape_html_entities(input);
        assert_eq!(
            result, "",
            "Empty string should return empty result"
        );
    }

    /// Test escaping of a variety of special HTML characters.
    #[test]
    fn test_escape_html_entities() {
        let input = "Hello, <world> & \"friends\"!";
        let expected =
            "Hello, &lt;world&gt; &amp; &quot;friends&quot;!";
        assert_eq!(escape_html_entities(input), expected);
    }

    /// Test async file-based metadata extraction.
    #[tokio::test]
    async fn test_async_extract_metadata_from_file() {
        use tempfile::tempdir;
        use tokio::fs::File;
        use tokio::io::AsyncWriteExt;

        // Create a temporary file with some content
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test.md");
        let mut file = File::create(&file_path).await.unwrap();
        let content = r#"---
title: Test Page
description: A test page for metadata extraction
keywords: test, metadata, extraction
---
# Test Content
This is a test file for metadata extraction."#;
        file.write_all(content.as_bytes()).await.unwrap();

        // Test the async_extract_metadata_from_file function
        let result = async_extract_metadata_from_file(
            file_path.to_str().unwrap(),
        )
        .await;
        assert!(result.is_ok());

        let (metadata, keywords, meta_tags) = result.unwrap();
        assert_eq!(
            metadata.get("title"),
            Some(&"Test Page".to_string())
        );
        assert_eq!(
            metadata.get("description"),
            Some(&"A test page for metadata extraction".to_string())
        );
        assert_eq!(keywords, vec!["test", "metadata", "extraction"]);
        assert!(!meta_tags.primary.is_empty());
    }
}

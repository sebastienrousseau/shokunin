use crate::error::MetadataError;
use crate::extract_and_prepare_metadata;
use crate::metatags::MetaTagGroups;
use std::collections::HashMap;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

/// Escapes special HTML characters in a string.
///
/// # Arguments
///
/// * `value` - The string to escape.
///
/// # Returns
///
/// A new string with special HTML characters escaped.
///
/// # Example
///
/// ```
/// use metadata_gen::escape_html_entities;
///
/// let input = "Hello, <world>!";
/// let expected = "Hello, &lt;world&gt;!";
///
/// assert_eq!(escape_html_entities(input), expected);
/// ```
pub fn escape_html_entities(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('\"', "&quot;")
        .replace('\'', "&#x27;")
}

/// Asynchronously reads a file and extracts metadata from its content.
///
/// This function reads the content of a file asynchronously and then extracts
/// metadata, generates keywords, and prepares meta tag groups.
///
/// # Arguments
///
/// * `file_path` - A string slice representing the path to the file.
///
/// # Returns
///
/// Returns a Result containing a tuple with:
/// * `HashMap<String, String>`: Extracted metadata
/// * `Vec<String>`: A list of keywords
/// * `MetaTagGroups`: A structure containing various meta tags
///
/// # Errors
///
/// This function will return a `MetadataError` if file reading, metadata extraction, or processing fails.
///
pub async fn async_extract_metadata_from_file(
    file_path: &str,
) -> Result<
    (HashMap<String, String>, Vec<String>, MetaTagGroups),
    MetadataError,
> {
    let mut file = File::open(file_path)
        .await
        .map_err(MetadataError::IoError)?;
    let mut content = String::new();
    file.read_to_string(&mut content)
        .await
        .map_err(MetadataError::IoError)?;

    extract_and_prepare_metadata(&content)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_html_entities() {
        let input = "Hello, <world> & \"friends\"!";
        let expected =
            "Hello, &lt;world&gt; &amp; &quot;friends&quot;!";
        assert_eq!(escape_html_entities(input), expected);
    }

    #[tokio::test]
    async fn test_async_extract_metadata_from_file() {
        use tokio::fs::File;
        use tokio::io::AsyncWriteExt;

        // Create a temporary file with some content
        let temp_dir = tempfile::tempdir().unwrap();
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

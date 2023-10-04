#[cfg(test)]
mod tests {
    use ssg::modules::keywords::extract_keywords;
    use std::collections::HashMap;

    #[test]
    fn test_extract_keywords_with_valid_keywords() {
        let mut metadata = HashMap::new();
        metadata.insert("keywords".to_string(), "rust,programming,testing".to_string());

        let keywords = extract_keywords(&metadata);

        assert_eq!(keywords, vec!["rust", "programming", "testing"]);
    }

    #[test]
    fn test_extract_keywords_with_missing_keywords() {
        let metadata = HashMap::new(); // Empty metadata

        let keywords = extract_keywords(&metadata);

        assert_eq!(keywords, Vec::<String>::new());
    }

    #[test]
    fn test_extract_keywords_with_whitespace() {
        let mut metadata = HashMap::new();
        metadata.insert("keywords".to_string(), "  rust ,  programming  ,  testing  ".to_string());

        let keywords = extract_keywords(&metadata);

        assert_eq!(keywords, vec!["rust", "programming", "testing"]);
    }
}
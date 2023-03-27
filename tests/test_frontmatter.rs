#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use ssg::frontmatter::extract;

    #[test]
    fn test_extract_with_valid_content() {
        let content = "---\ntitle: Hello World\nauthor: John Doe\n---\nHello, world!";
        let result = extract(content);
        let expected: HashMap<String, String> = [
            ("title".to_string(), "Hello World".to_string()),
            ("author".to_string(), "John Doe".to_string()),
        ]
        .iter()
        .cloned()
        .collect();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_extract_with_invalid_content() {
        let content = "Hello, world!";
        let result = extract(content);
        let expected: HashMap<String, String> = HashMap::new();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_extract_with_empty_content() {
        let content = "";
        let result = extract(content);
        let expected: HashMap<String, String> = HashMap::new();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_extract_with_empty_frontmatter() {
        let content = "";
        let result = extract(content);
        let expected: HashMap<String, String> = HashMap::new();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_extract_with_toml_frontmatter() {
        let content = "+++\ntitle = \"Hello World\"\nauthor = \"John Doe\"\n+++\nHello, world!";
        let result = extract(content);
        let expected: HashMap<String, String> = [
            ("title".to_string(), "Hello World".to_string()),
            ("author".to_string(), "John Doe".to_string()),
        ]
        .iter()
        .cloned()
        .collect();
        assert_eq!(result, expected);
    }
}

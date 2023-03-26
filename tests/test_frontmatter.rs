#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use ssg::frontmatter::extract;

    #[test]
    fn test_extract_with_valid_content() {
        let content = "---\ntitle: Hello World\nauthor: John Doe\n---\nHello, world!";
        let result = extract(&content);
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
        let result = extract(&content);
        let expected: HashMap<String, String> = HashMap::new();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_extract_with_invalid_front_matter() {
        let content =
            "---\ntitle: Hello World\nauthor\n---\nHello, world!";
        let result = extract(&content);
        let expected: HashMap<String, String> =
            [("title".to_string(), "Hello World".to_string())]
                .iter()
                .cloned()
                .collect();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_extract_with_multiple_colons() {
        let content = "---\ntitle: Hello: World\nauthor: John Doe\n---\nHello, world!";
        let result = extract(&content);
        let expected: HashMap<String, String> = [
            ("title".to_string(), "Hello: World".to_string()),
            ("author".to_string(), "John Doe".to_string()),
        ]
        .iter()
        .cloned()
        .collect();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_extract_with_nested_front_matter() {
        let content = "---\ntitle: Hello World\nauthor: John Doe\ninfo:\n  date: 2022-03-25\n  category: Rust\n---\nHello, world!";
        let result = extract(&content);
        let mut expected: Vec<(String, String)> = [
            ("author".to_string(), "John Doe".to_string()),
            ("category".to_string(), "Rust".to_string()),
            ("date".to_string(), "2022-03-25".to_string()),
            ("info".to_string(), "".to_string()),
            ("title".to_string(), "Hello World".to_string()),
        ]
        .iter()
        .cloned()
        .collect();
        expected.sort();
        let mut result: Vec<(String, String)> =
            result.into_iter().collect();
        result.sort();
        assert_eq!(result, expected);
    }
}

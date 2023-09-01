#[cfg(test)]
mod tests {
    use ssg::modules::metatags::generate_metatags;

    #[test]
    fn test_generate_metatags() {
        let meta = vec![
            (
                "description".to_string(),
                "A blog about Rust programming.".to_string(),
            ),
            ("keywords".to_string(), "Rust, programming".to_string()),
        ];
        let result = generate_metatags(&meta);
        let expected = "<meta name=\"description\" content=\"A blog about Rust programming.\">\n<meta name=\"keywords\" content=\"Rust, programming\">";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_generate_metatags_empty() {
        let meta: &[(String, String)] = &[];
        let result = generate_metatags(meta);
        assert_eq!(result, "");
    }

    #[test]
    fn test_generate_metatags_single() {
        let meta = &[(
            "description".to_string(),
            "My site description".to_string(),
        )];
        let result = generate_metatags(meta);
        assert_eq!(
            result,
            "<meta name=\"description\" content=\"My site description\">"
        );
    }

    #[test]
    fn test_generate_metatags_multiple() {
        let meta = &[
            (
                "description".to_string(),
                "My site description".to_string(),
            ),
            ("keywords".to_string(), "rust,web,ssg".to_string()),
            ("author".to_string(), "John Doe".to_string()),
        ];
        let result = generate_metatags(meta);
        assert_eq!(
            result,
            "<meta name=\"description\" content=\"My site description\">\n<meta name=\"keywords\" content=\"rust,web,ssg\">\n<meta name=\"author\" content=\"John Doe\">"
        );
    }

    #[test]
    fn test_generate_metatags_multiple_lines() {
        let meta = &[
            (
                "description".to_string(),
                "My site description".to_string(),
            ),
            (
                "keywords".to_string(),
                "rust,web,ssg\nrust,web,ssg".to_string(),
            ),
            ("author".to_string(), "John Doe".to_string()),
        ];
        let result = generate_metatags(meta);
        assert_eq!(
            result,
            "<meta name=\"description\" content=\"My site description\">\n<meta name=\"keywords\" content=\"rust,web,ssg\nrust,web,ssg\">\n<meta name=\"author\" content=\"John Doe\">"
        );
    }
    #[test]
    fn test_generate_metatags_empty_lines() {
        let meta = &[
            (
                "description".to_string(),
                "My site description".to_string(),
            ),
            ("keywords".to_string(), "".to_string()),
            ("author".to_string(), "John Doe".to_string()),
        ];
        let result = generate_metatags(meta);
        assert_eq!(
            result,
            "<meta name=\"description\" content=\"My site description\">\n<meta name=\"keywords\" content=\"\">\n<meta name=\"author\" content=\"John Doe\">"
        );
        let meta: &[(String, String)] = &[];
        let result = generate_metatags(meta);
        assert_eq!(result, "");
    }
    #[test]
    fn test_generate_metatags_multiple_lines_empty() {
        let meta = &[
            (
                "description".to_string(),
                "My site description".to_string(),
            ),
            (
                "keywords".to_string(),
                "rust,web,ssg\nrust,web,ssg".to_string(),
            ),
            ("author".to_string(), "".to_string()),
        ];
        let result = generate_metatags(meta);
        assert_eq!(
            result,
            "<meta name=\"description\" content=\"My site description\">\n<meta name=\"keywords\" content=\"rust,web,ssg\nrust,web,ssg\">\n<meta name=\"author\" content=\"\">"
        );
    }
}

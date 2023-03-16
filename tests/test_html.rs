#[cfg(test)]
mod tests {
    use ssg::frontmatter::extract_front_matter;
    use ssg::html::{generate_html, generate_meta_tags};

    #[test]
    fn test_generate_meta_tags() {
        let meta = vec![
            (
                "description".to_string(),
                "A blog about Rust programming.".to_string(),
            ),
            ("keywords".to_string(), "Rust, programming".to_string()),
        ];
        let result = generate_meta_tags(&meta);
        let expected = "<meta name=\"description\" content=\"A blog about Rust programming.\">\n<meta name=\"keywords\" content=\"Rust, programming\">";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_generate_html() {
        let content = "---\ntitle: Hello, world!\ndescription: Welcome to my blog.\n---\n\nThis is the content of my first blog post.";
        let (title, description, _, _) = extract_front_matter(&content);
        let result = generate_html(&content, &title, &description);
        let expected = "<h1>Hello, world!</h1><h2>Welcome to my blog.</h2><p>This is the content of my first blog post.</p>\n";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_generate_html_no_front_matter() {
        let content = "This is the content of my first blog post.";
        let result = generate_html(&content, "", "");
        let expected =
            "<p>This is the content of my first blog post.</p>\n";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_generate_html_missing_fields() {
        let content = "---\ntitle: Hello, world!\n---\n\nThis is the content of my first blog post.";
        let (title, description, _, _) = extract_front_matter(&content);
        let result = generate_html(&content, &title, &description);
        let expected = "<h1>Hello, world!</h1><p>This is the content of my first blog post.</p>\n";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_generate_html_empty_fields() {
        let content = "---\ntitle: Hello, world!\ndescription: \n---\n\nThis is the content of my first blog post.";
        let (title, description, _, _) = extract_front_matter(&content);
        let result = generate_html(&content, &title, &description);
        let expected = "<h1>Hello, world!</h1><p>This is the content of my first blog post.</p>\n";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_generate_html_no_title() {
        let content = "---\ndescription: Welcome to my blog.\n---\n\nThis is the content of my first blog post.";
        let (_, description, _, _) = extract_front_matter(&content);
        let result = generate_html(&content, "", &description);
        let expected = "<h2>Welcome to my blog.</h2><p>This is the content of my first blog post.</p>\n";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_generate_html_no_description() {
        let content = "---\ntitle: Hello, world!\n---\n\nThis is the content of my first blog post.";
        let (title, _, _, _) = extract_front_matter(&content);
        let result = generate_html(&content, &title, "");
        let expected = "<h1>Hello, world!</h1><p>This is the content of my first blog post.</p>\n";
        assert_eq!(result, expected);
    }
}

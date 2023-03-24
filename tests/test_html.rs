#[cfg(test)]
mod tests {
    use ssg::frontmatter::extract;
    use ssg::html::generate_html;

    #[test]
    fn test_generate_html() {
        let content = "---\ntitle: Hello, world!\ndescription: Welcome to my blog.\n---\n\nThis is the content of my first blog post.";
        let (title, _, description, _, _, _) = extract(content);
        let result = generate_html(content, &title, &description);
        let expected = "<h1>Hello, world!</h1><h2>Welcome to my blog.</h2><p>This is the content of my first blog post.</p>\n";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_generate_html_no_front_matter() {
        let content = "This is the content of my first blog post.";
        let result = generate_html(content, "", "");
        let expected =
            "<p>This is the content of my first blog post.</p>\n";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_generate_html_missing_fields() {
        let content = "---\ntitle: Hello, world!\n---\n\nThis is the content of my first blog post.";
        let (title, _, description, _, _, _) = extract(content);
        let result = generate_html(content, &title, &description);
        let expected = "<h1>Hello, world!</h1><p>This is the content of my first blog post.</p>\n";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_generate_html_empty_fields() {
        let content = "---\ntitle: Hello, world!\ndescription: \n---\n\nThis is the content of my first blog post.";
        let (title, _, description, _, _, _) = extract(content);
        let result = generate_html(content, &title, &description);
        let expected = "<h1>Hello, world!</h1><p>This is the content of my first blog post.</p>\n";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_generate_html_no_title() {
        let content = "---\ndescription: Welcome to my blog.\n---\n\nThis is the content of my first blog post.";
        let (_, _, description, _, _, _) = extract(content);
        let result = generate_html(content, "", &description);
        let expected = "<h2>Welcome to my blog.</h2><p>This is the content of my first blog post.</p>\n";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_generate_html_no_description() {
        let content = "---\ntitle: Hello, world!\n---\n\nThis is the content of my first blog post.";
        let (title, _, _, _, _, _) = extract(content);
        let result = generate_html(content, &title, "");
        let expected = "<h1>Hello, world!</h1><p>This is the content of my first blog post.</p>\n";
        assert_eq!(result, expected);
    }
}

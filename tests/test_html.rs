#[cfg(test)]
mod tests {
    use ssg::html::generate_html;

    #[test]
    fn test_generate_html_with_front_matter() {
        let content = "---\ntitle: Hello, world!\ndescription: A simple greeting\n---\n# Hello, world!";
        let title = "Welcome";
        let description = "Say hi to the world!";
        let result = generate_html(content, title, description, None);
        let expected = "<h1>Welcome</h1><h2>Say hi to the world!</h2><h1>Hello, world!</h1>\n";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_generate_html_without_front_matter() {
        let content = "# Hello, world!";
        let title = "Welcome";
        let description = "Say hi to the world!";
        let result = generate_html(content, title, description, None);
        let expected = "<h1>Welcome</h1><h2>Say hi to the world!</h2><h1>Hello, world!</h1>\n";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_generate_html_without_title() {
        let content =
            "---\ndescription: A simple greeting\n---\n# Hello, world!";
        let title = "";
        let description = "Say hi to the world!";
        let result = generate_html(content, title, description, None);
        let expected =
            "<h2>Say hi to the world!</h2><h1>Hello, world!</h1>\n";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_generate_html_without_description() {
        let content = "---\ntitle: Hello, world!\n---\n# Hello, world!";
        let title = "Welcome";
        let description = "";
        let result = generate_html(content, title, description, None);
        let expected = "<h1>Welcome</h1><h1>Hello, world!</h1>\n";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_generate_html_with_empty_fields() {
        let content = "---\ntitle:\ndescription:\n---\n# Hello, world!";
        let title = "Welcome";
        let description = "Say hi to the world!";
        let result = generate_html(content, title, description, None);
        let expected = "<h1>Welcome</h1><h2>Say hi to the world!</h2><h1>Hello, world!</h1>\n";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_generate_html_with_empty_content() {
        let content = "";
        let title = "Welcome";
        let description = "Say hi to the world!";
        let result = generate_html(content, title, description, None);
        let expected = "<h1>Welcome</h1><h2>Say hi to the world!</h2>";
        assert_eq!(result, expected);
    }
}

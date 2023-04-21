#[cfg(test)]
mod tests {
    use ssg::html::generate_html;

    #[test]
    fn test_generate_html_with_front_matter() {
        let content =
            "---\ntitle: Hello, world!\ndescription: A simple greeting\n---\n# Hello, world!";
        let title = "Welcome";
        let description = "Say hi to the world!";
        let result = generate_html(content, title, description, None);
        let expected = "<h1 id=\"h1-id-welcome\" id=\"\" class=\"h1-id-welcome\">Welcome</h1><p>Say hi to the world!</p><h1><a href=\"#hello-world\" aria-hidden=\"true\" class=\"anchor\" id=\"hello-world\"></a>Hello, world!</h1>";
        assert_eq!(result.trim(), expected);
    }

    #[test]
    fn test_generate_html_without_front_matter() {
        let content = "# Hello, world!";
        let title = "Welcome";
        let description = "Say hi to the world!";
        let result = generate_html(content, title, description, None);
        let expected = "<h1 id=\"h1-id-welcome\" id=\"\" class=\"h1-id-welcome\">Welcome</h1><p>Say hi to the world!</p><h1><a href=\"#hello-world\" aria-hidden=\"true\" class=\"anchor\" id=\"hello-world\"></a>Hello, world!</h1>";
        assert_eq!(result.trim(), expected);
    }

    #[test]
    fn test_generate_html_without_title() {
        let content = "---\ndescription: A simple greeting\n---\n# Hello, world!";
        let title = "";
        let description = "Say hi to the world!";
        let result = generate_html(content, title, description, None);
        let expected = "<p>Say hi to the world!</p><h1><a href=\"#hello-world\" aria-hidden=\"true\" class=\"anchor\" id=\"hello-world\"></a>Hello, world!</h1>";
        assert_eq!(result.trim(), expected);
    }

    #[test]
    fn test_generate_html_without_description() {
        let content = "---\ntitle: Hello, world!\n---\n# Hello, world!";
        let title = "Welcome";
        let description = "";
        let result = generate_html(content, title, description, None);
        let expected = "<h1 id=\"h1-id-welcome\" id=\"\" class=\"h1-id-welcome\">Welcome</h1><h1><a href=\"#hello-world\" aria-hidden=\"true\" class=\"anchor\" id=\"hello-world\"></a>Hello, world!</h1>";
        assert_eq!(result.trim(), expected);
    }

    #[test]
    fn test_generate_html_with_empty_fields() {
        let content = "---\ntitle:\ndescription:\n---\n# Hello, world!";
        let title = "Welcome";
        let description = "Say hi to the world!";
        let result = generate_html(content, title, description, None);
        let expected = "<h1 id=\"h1-id-welcome\" id=\"\" class=\"h1-id-welcome\">Welcome</h1><p>Say hi to the world!</p><h1><a href=\"#hello-world\" aria-hidden=\"true\" class=\"anchor\" id=\"hello-world\"></a>Hello, world!</h1>";
        assert_eq!(result.trim(), expected);
    }

    #[test]
    fn test_generate_html_with_empty_content() {
        let content = "";
        let title = "Welcome";
        let description = "Say hi to the world!";
        let result = generate_html(content, title, description, None);
        let expected = "<h1 id=\"h1-id-welcome\" id=\"\" class=\"h1-id-welcome\">Welcome</h1><p>Say hi to the world!</p>";
        assert_eq!(result.trim(), expected);
    }
}

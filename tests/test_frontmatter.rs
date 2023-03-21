#[cfg(test)]
mod tests {
    use ssg::frontmatter::extract;

    #[test]
    fn test_extract() {
        let content = "---\ntitle: Hello, world!\ndescription: Welcome to my blog.\nkeywords: Rust, programming\npermalink: /hello-world/\n---\n\nThis is the content of my first blog post.";
        let (title, description, keywords, permalink) =
            extract(&content);
        assert_eq!(title, "Hello, world!");
        assert_eq!(description, "Welcome to my blog.");
        assert_eq!(keywords, "Rust, programming");
        assert_eq!(permalink, "/hello-world/");
    }

    #[test]
    fn test_extract_no_front_matter() {
        let content = "This is a blog post without front matter.";
        let (title, description, keywords, permalink) =
            extract(&content);
        assert_eq!(title, "");
        assert_eq!(description, "");
        assert_eq!(keywords, "");
        assert_eq!(permalink, "");
    }

    #[test]
    fn test_extract_invalid_front_matter() {
        let content = "---\ninvalid-front-matter\n---\n\nThis is a blog post with invalid front matter.";
        let (title, description, keywords, permalink) =
            extract(&content);
        assert_eq!(title, "");
        assert_eq!(description, "");
        assert_eq!(keywords, "");
        assert_eq!(permalink, "");
    }

    #[test]
    fn test_extract_missing_values() {
        let content = "---\ntitle: Hello, world!\npermalink: /hello-world/\n---\n\nThis is a blog post with missing values in the front matter.";
        let (title, description, keywords, permalink) =
            extract(&content);
        assert_eq!(title, "Hello, world!");
        assert_eq!(description, "");
        assert_eq!(keywords, "");
        assert_eq!(permalink, "/hello-world/");
    }

    #[test]
    fn test_extract_whitespace_values() {
        let content = "---\ntitle: Hello, world!\ndescription: \nkeywords: Rust, programming\npermalink: /hello-world/\n---\n\nThis is a blog post with whitespace values in the front matter.";
        let (title, description, keywords, permalink) =
            extract(&content);
        assert_eq!(title, "Hello, world!");
        assert_eq!(description, "");
        assert_eq!(keywords, "Rust, programming");
        assert_eq!(permalink, "/hello-world/");
    }
}

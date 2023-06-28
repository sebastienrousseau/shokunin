#[cfg(test)]
mod tests {
    use ssg::{data::FileData, navigation::generate_navigation};

    #[test]
    fn test_generate_navigation() {
        // Create sample FileData items
        let files = vec![
            FileData {
                name: "index.md".to_string(),
                content: "".to_string(),
                rss: "".to_string(),
                json: "".to_string(),
                txt: "".to_string(),
                cname: "".to_string(),
                sitemap: "".to_string(),
            },
            FileData {
                name: "about.md".to_string(),
                content: "".to_string(),
                rss: "".to_string(),
                json: "".to_string(),
                txt: "".to_string(),
                cname: "".to_string(),
                sitemap: "".to_string(),
            },
            FileData {
                name: "contact.md".to_string(),
                content: "".to_string(),
                rss: "".to_string(),
                json: "".to_string(),
                txt: "".to_string(),
                cname: "".to_string(),
                sitemap: "".to_string(),
            },
        ];

        let navigation = generate_navigation(&files);

        // Assert that the generated navigation contains expected HTML elements
        assert!(navigation.contains("<li class=\"nav-item\"><a href=\"/about/index.html\" class=\"text-uppercase p-2 \">About</a></li>"));
        assert!(navigation.contains("<li class=\"nav-item\"><a href=\"/contact/index.html\" class=\"text-uppercase p-2 \">Contact</a></li>"));

        // Assert that the generated navigation does not contain excluded files
        assert!(!navigation.contains("<li class=\"nav-item\"><a href=\"/index/index.html\" class=\"text-uppercase p-2 \">Index</a></li>"));
    }
}

#[cfg(test)]
mod tests {
    use ssg::{data::FileData, modules::navigation::generate_navigation};

    #[test]
    fn test_generate_navigation() {
        // Create sample FileData items
        let files = vec![
            FileData {
                cname: "".to_string(),
                content: "".to_string(),
                human: "".to_string(),
                json: "".to_string(),
                keyword: "".to_string(),
                name: "index.md".to_string(),
                rss: "".to_string(),
                sitemap: "".to_string(),
                txt: "".to_string(),
            },
            FileData {
                cname: "".to_string(),
                content: "".to_string(),
                human: "".to_string(),
                json: "".to_string(),
                keyword: "".to_string(),
                name: "about.md".to_string(),
                rss: "".to_string(),
                sitemap: "".to_string(),
                txt: "".to_string(),
            },
            FileData {
                cname: "".to_string(),
                content: "".to_string(),
                human: "".to_string(),
                json: "".to_string(),
                keyword: "".to_string(),
                name: "blog.md".to_string(),
                rss: "".to_string(),
                sitemap: "".to_string(),
                txt: "".to_string(),
            },
            FileData {
                cname: "".to_string(),
                content: "".to_string(),
                human: "".to_string(),
                json: "".to_string(),
                keyword: "".to_string(),
                name: "contact.md".to_string(),
                rss: "".to_string(),
                sitemap: "".to_string(),
                txt: "".to_string(),
            },
        ];

        let navigation = generate_navigation(&files);

        println!("Generated navigation: {}", navigation);

        // Assert that the generated navigation contains expected HTML elements
        assert!(navigation.contains("<li class=\"nav-item\"><a href=\"/about/index.html\" class=\"text-uppercase p-2 \">About</a></li>"));
        assert!(navigation.contains("<li class=\"nav-item\"><a href=\"/contact/index.html\" class=\"text-uppercase p-2 \">Contact</a></li>"));

        // Assert that the generated navigation does not contain excluded files
        assert!(!navigation.contains("<li class=\"nav-item\"><a href=\"/index/index.html\" class=\"text-uppercase p-2 \">Index</a></li>"));
    }
}

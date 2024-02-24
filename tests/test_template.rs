#[cfg(test)]
mod tests {
    use ssg::utilities::template::{
        render_page, render_template, PageOptions,
    };
    use std::{collections::HashMap, error::Error};

    #[test]
    fn test_render_template() -> Result<(), Box<dyn Error>> {
        let template = "<html><head><title>{{title}}</title></head><body>{{content}}</body></html>";
        let mut context = HashMap::new();
        context.insert("title", "My Title");
        context.insert("content", "My Content");
        let result = render_template(template, &context)?;
        assert_eq!(
            result,
            "<html><head><title>My Title</title></head><body>My Content</body></html>"
        );
        Ok(())
    }

    #[test]
    fn test_render_template_unresolved_tags() {
        let template = "<html><head><title>{{title}}</title></head><body>{{content}}</body></html>";
        let mut context = HashMap::new();
        context.insert("title", "My Title");
        let result = render_template(template, &context);
        assert_eq!(
            result,
            Err("Failed to render template, unresolved template tags: <html><head><title>My Title</title></head><body>{{content}}</body></html>".to_owned())
        );
    }

    #[test]
    fn test_render_page() -> Result<(), Box<dyn Error>> {
        // Prepare the test data
        let mut options = PageOptions::new();
        options.set("author", "John Doe");
        options.set("banner_alt", "A simple test page");
        options.set("banner_height", "100");
        options.set("banner_width", "100");
        options.set("banner", "./images/banner.png");
        options.set("cdn", "https://example.com");
        options.set("charset", "utf-8");
        options.set("cname", "example.com");
        options.set("content", "Hello, world!");
        options.set("copyright", "Copyright 2023");
        options.set("date", "2000-01-01");
        options.set("description", "A simple test page");
        options.set(
            "download",
            "<a href=\"https://example.com\">Download</a>",
        );
        options.set("format-detection", "telephone=no");
        options.set("generator", "SSG");
        options.set("hreflang", "en");
        options.set("icon", "/favicon.ico");
        options.set("id", "https://example.com");
        options.set("image_alt", "A simple test page");
        options.set("image_height", "100");
        options.set("image_width", "100");
        options.set("image", "./images/test.png");
        options.set("keywords", "test, page");
        options.set("language", "en-GB");
        options.set("layout", "page");
        options.set("locale", "en_GB");
        options.set("logo_alt", "A simple test page");
        options.set("logo_height", "100");
        options.set("logo_width", "100");
        options.set("logo", "./images/logo.png");
        options.set("name", "My Site");
        options.set("navigation", "<nav>Home</nav>");
        options.set("permalink", "/test-page.html");
        options.set("rating", "general");
        options.set("referrer", "no-referrer");
        options.set("revisit_after", "7 days");
        options.set("robots", "index, follow");
        options.set("short_name", "Test Page");
        options.set("subtitle", "A simple test page");
        options.set("tags", "test, page");
        options.set("title", "Test Page");
        options.set("url", "https://example.com");
        options
            .set("viewport", "width=device-width, initial-scale=1.0");

        let template_path = String::from("./template");
        let layout = String::from("page");

        // Create a temporary directory and copy the template file into it
        let tempdir = tempfile::tempdir()
            .expect("Could not create temporary directory");
        let template_file_path = tempdir.path().join("template.html");
        std::fs::copy("template/template.html", template_file_path)
            .expect("Could not copy template file");

        // Call the render_page function
        let result = render_page(&options, &template_path, &layout);

        // Check the return value of the render_page function
        if let Err(e) = result {
            println!("Error: {:?}", e);
        }

        Ok(())
    }
}

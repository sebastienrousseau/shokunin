#[cfg(test)]
mod tests {
    use ssg::template::{render_page, render_template, PageOptions};
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
    fn test_render_page() -> Result<(), String> {
        // Prepare the test data
        let options = PageOptions {
            author: "John Doe",
            banner: "./images/banner.png",
            bing_site_verification: "1234567890",
            charset: "utf-8",
            content: "Hello, world!",
            copyright: "Copyright 2023",
            cname: "example",
            css: "styles.css",
            date: "2021-01-01",
            description: "A simple test page",
            generator: "SSG",
            google_site_verification: "1234567890",
            image: "./images/test.png",
            keywords: "test, page",
            lang: "en",
            layout: "page",
            meta: "",
            msapplication_config: "/browserconfig.xml",
            msapplication_tap_highlight: "no",
            msapplication_tile_color: "#da532c",
            msapplication_tile_image: "/mstile-144x144.png",
            msvalidate1: "1234567890",
            name: "My Site",
            navigation: "<nav>Home</nav>",
            og_description: "A simple test page",
            og_image_alt: "A simple test page",
            og_image: "./images/test.png",
            og_locale: "en_US",
            og_site_name: "My Site",
            og_title: "Test Page",
            og_type: "website",
            og_url: "https://example.com",
            robots: "index, follow",
            subtitle: "A simple test page",
            theme_color: "#ffffff",
            title: "Test Page",
            twitter_card: "summary",
            twitter_creator: "johndoe",
            twitter_description: "A simple test page",
            twitter_image_alt: "A simple test page",
            twitter_image: "./images/test.png",
            twitter_site: "johndoe",
            twitter_title: "Test Page",
            twitter_url: "https://example.com",
            url: "https://example.com",
            banner_width: "100",
            banner_height: "100",
            banner_alt: "A simple test page",
            logo: "./images/logo.png",
            logo_width: "100",
            logo_height: "100",
            logo_alt: "A simple test page",
        };
        let template_path = String::from("./template");
        let layout = String::from("page");

        // Create a temporary directory and copy the template file into it
        let tempdir = tempfile::tempdir()
            .map_err(|err| format!("Could not create temporary directory: {}", err))?;
        let template_file_path = tempdir.path().join("template.html");
        std::fs::copy("template/template.html", template_file_path)
            .map_err(|err| format!("Could not copy template file: {}", err))?;

        // Call the render_page function
        let result = render_page(&options, &template_path, &layout);

        // Assert that the result is correct
        assert!(result.is_ok());

        Ok(())
    }
}

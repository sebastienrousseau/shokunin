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
        assert_eq!(result, "<html><head><title>My Title</title></head><body>My Content</body></html>");
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
    fn test_render_page() {
        // Prepare the test data
        let options = PageOptions {
            content: "Hello, world!",
            copyright: "Copyright 2023",
            css: "styles.css",
            date: "2021-01-01",
            description: "A simple test page",
            keywords: "test, page",
            lang: "en",
            meta: "",
            navigation: "<nav>Home</nav>",
            title: "Test Page",
        };
        let template_path = String::from("./template");

        // Create a temporary directory and copy the template file into it
        let tempdir = tempfile::tempdir().unwrap();
        let template_file_path = tempdir.path().join("template.html");
        std::fs::copy("template/template.html", &template_file_path)
            .unwrap();

        // Call the render_page function
        let result = render_page(&options, &template_path);

        // Assert that the result is correct
        assert!(result.is_ok());
    }
}

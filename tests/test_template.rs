#[cfg(test)]
mod tests {
    use ssg::template::render_template;
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
}

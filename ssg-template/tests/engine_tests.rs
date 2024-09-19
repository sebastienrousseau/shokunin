/// Unit tests for the `Engine` struct and its methods.
#[cfg(test)]
mod tests {
    use ssg_template::{Context, Engine, TemplateError};
    use std::collections::HashMap;
    use std::fs::File;
    use std::io::Write;
    use std::time::Duration;
    use tempfile::tempdir;

    /// Helper function to create an `Engine` instance.
    fn create_engine() -> Engine {
        Engine::new("dummy/path", Duration::from_secs(60))
    }

    /// Helper function to create a basic context with default values.
    fn create_basic_context() -> HashMap<String, String> {
        let mut context = HashMap::new();
        context.insert("name".to_string(), "World".to_string());
        context.insert("greeting".to_string(), "Hello".to_string());
        context
    }

    /// Helper function to assert template rendering results.
    fn assert_template_rendering(
        engine: &Engine,
        template: &str,
        context: &HashMap<String, String>,
        expected_result: Result<&str, TemplateError>,
    ) {
        let result = engine.render_template(template, context);
        match expected_result {
            Ok(expected) => assert_eq!(result.unwrap(), expected),
            Err(_) => assert!(result.is_err()),
        }
    }

    /// Tests for template rendering in the `Engine` struct.
    mod render_tests {
        use super::*;

        #[test]
        fn test_engine_render_template() {
            let engine = create_engine();
            let context = create_basic_context();
            let template = "{{greeting}}, {{name}}!";
            assert_template_rendering(
                &engine,
                template,
                &context,
                Ok("Hello, World!"),
            );
        }

        #[test]
        fn test_engine_render_template_unresolved_tags() {
            let engine = create_engine();
            let context: HashMap<String, String> = HashMap::new();
            let template = "{{greeting}}, {{name}}!";
            assert_template_rendering(
                &engine,
                template,
                &context,
                Err(TemplateError::RenderError("".to_string())),
            );
        }

        #[test]
        fn test_engine_render_empty_template() {
            let engine = create_engine();
            let context: HashMap<String, String> = HashMap::new();
            let template = "";
            assert_template_rendering(
                &engine,
                template,
                &context,
                Err(TemplateError::RenderError(
                    "Template is empty".to_string(),
                )),
            );
        }

        #[test]
        fn test_engine_render_special_characters_in_context() {
            let engine = create_engine();
            let mut context = HashMap::new();
            context.insert(
                "name".to_string(),
                "<script>alert('XSS')</script>".to_string(),
            );
            context.insert("greeting".to_string(), "&".to_string());
            let template = "{{greeting}} {{name}}";
            assert_template_rendering(
                &engine,
                template,
                &context,
                Ok("& <script>alert('XSS')</script>"),
            );
        }

        #[test]
        fn test_engine_large_context() {
            let engine = create_engine();
            let mut context = HashMap::new();
            let keys: Vec<String> =
                (0..1000).map(|i| format!("key{}", i)).collect();
            let values: Vec<String> =
                (0..1000).map(|i| format!("value{}", i)).collect();

            for i in 0..1000 {
                context.insert(keys[i].clone(), values[i].clone());
            }

            let mut template = String::new();
            for i in 0..1000 {
                template.push_str(&format!("{{{{key{}}}}}", i));
            }

            let result =
                engine.render_template(&template, &context).unwrap();
            let expected_result =
                (0..1000).fold(String::new(), |mut acc, i| {
                    use std::fmt::Write;
                    write!(&mut acc, "value{}", i).unwrap();
                    acc
                });

            assert_eq!(result, expected_result);
        }
    }

    /// Tests related to file operations, such as downloading templates.
    mod file_tests {
        use super::*;

        #[test]
        fn test_engine_download_file() {
            let engine = create_engine();
            let url = "https://raw.githubusercontent.com/sebastienrousseau/shokunin/main/templates";
            let file = "index.html";
            let temp_dir = tempdir().unwrap();
            let result =
                engine.download_file(url, file, temp_dir.path());
            assert!(result.is_ok());
        }

        #[test]
        fn test_engine_invalid_template_path() {
            let mut engine =
                Engine::new("invalid/path", Duration::from_secs(60));
            let context = Context {
                elements: HashMap::new(),
            };
            let result =
                engine.render_page(&context, "nonexistent_layout");
            assert!(matches!(result, Err(TemplateError::Io(_))));
        }

        #[test]
        fn test_render_page_valid_path() {
            let temp_dir = tempdir().unwrap(); // Create a temporary directory
            let layout_path = temp_dir.path().join("layout.html"); // Define the layout file path within the temp directory

            // Create the layout file and write content to it
            let mut file = File::create(&layout_path)
                .expect("Failed to create temp layout file");
            writeln!(file, "<html><body>{{{{greeting}}}}, {{{{name}}}}</body></html>").expect("Failed to write content to layout file");

            // Log the layout directory path for debugging
            println!(
                "Layout directory path: {}",
                temp_dir.path().to_str().unwrap()
            );

            // Initialize the engine with the template directory path, not the full file path
            let mut engine = Engine::new(
                temp_dir.path().to_str().unwrap(),
                Duration::from_secs(60),
            );

            let context = Context {
                elements: create_basic_context(),
            };

            let result = engine.render_page(&context, "layout"); // Only pass "layout" as the template name
            assert!(
                result.is_ok(),
                "Failed to render page, result: {:?}",
                result
            );

            let rendered_page = result.unwrap();
            println!("Rendered page: {}", rendered_page.trim());

            assert_eq!(
                rendered_page.trim(),
                "<html><body>Hello, World</body></html>"
            );
        }

        #[test]
        fn test_render_page_missing_file() {
            let mut engine =
                Engine::new("missing/path", Duration::from_secs(60));
            let context = Context {
                elements: HashMap::new(),
            };
            let result =
                engine.render_page(&context, "nonexistent_layout");
            assert!(matches!(result, Err(TemplateError::Io(_))));
        }
    }

    /// Tests for the `PageOptions` struct.
    mod page_options_tests {
        use ssg_template::PageOptions;

        #[test]
        fn test_page_options_new() {
            let options = PageOptions::new();
            assert!(options.elements.is_empty());
        }

        #[test]
        fn test_page_options_set_get() {
            let mut options = PageOptions::new();
            options.set("title".to_string(), "My Title".to_string());
            assert_eq!(
                options.get("title"),
                Some(&"My Title".to_string())
            );
            assert_eq!(options.get("non_existent"), None);
        }

        #[test]
        fn test_page_options_large_context() {
            let mut options = PageOptions::new();
            for i in 0..1000 {
                let key = format!("key{}", i);
                let value = format!("value{}", i);
                options.set(key, value); // Now we can pass the owned Strings
            }
            assert_eq!(
                options.get("key999"),
                Some(&"value999".to_string())
            );
            assert_eq!(options.get("key1000"), None);
        }
    }

    /// Edge case tests for template rendering.
    mod context_edge_cases_tests {
        use super::*;

        #[test]
        fn test_render_template_invalid_format() {
            let engine = create_engine();
            let context = create_basic_context();
            let template = "{greeting}, {name}!";
            assert_template_rendering(
                &engine,
                template,
                &context,
                Err(TemplateError::RenderError("".to_string())),
            );
        }

        #[test]
        fn test_render_template_invalid_syntax() {
            let engine = create_engine();
            let context = create_basic_context();
            let invalid_template = "Hello, {{name";
            assert_template_rendering(
                &engine,
                invalid_template,
                &context,
                Err(TemplateError::RenderError("".to_string())),
            );
        }

        #[test]
        fn test_render_large_template() {
            let engine = create_engine();
            let large_template = "Hello, {{name}}".repeat(1000);
            let context = create_basic_context();
            assert_template_rendering(
                &engine,
                &large_template,
                &context,
                Ok(&"Hello, World".repeat(1000)),
            );
        }

        #[test]
        fn test_render_template_empty_template() {
            let engine = create_engine();
            let context = create_basic_context();
            let empty_template = "";
            assert_template_rendering(
                &engine,
                empty_template,
                &context,
                Err(TemplateError::RenderError(
                    "Template is empty".to_string(),
                )),
            );
        }
    }
}

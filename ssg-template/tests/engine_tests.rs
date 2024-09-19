/// Unit tests for the `Engine` struct and its methods.
#[cfg(test)]
mod tests {
    use ssg_template::{Context, Engine, TemplateError};
    use std::collections::HashMap;
    use std::time::Duration;

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

    /// Tests for template rendering in the `Engine` struct.
    mod render_tests {
        use super::*;

        #[test]
        fn test_engine_render_template() {
            let engine = create_engine();
            let context = Context {
                elements: create_basic_context(),
            };
            let template = "{{greeting}}, {{name}}!";

            let result = engine
                .render_template(template, &context.elements)
                .unwrap();
            assert_eq!(result, "Hello, World!");
        }

        #[test]
        fn test_engine_render_template_unresolved_tags() {
            let engine = create_engine();
            let context: HashMap<String, String> = HashMap::new();
            let template = "{{greeting}}, {{name}}!";

            let result = engine.render_template(template, &context);
            assert!(matches!(
                result,
                Err(TemplateError::RenderError(_))
            ));
        }

        #[test]
        fn test_engine_render_empty_template() {
            let engine = create_engine();
            let context: HashMap<String, String> = HashMap::new();
            let template = "";

            let result = engine.render_template(template, &context);
            assert!(
                matches!(result, Err(TemplateError::RenderError(msg)) if msg == "Template is empty")
            );
        }

        #[test]
        fn test_engine_render_empty_context() {
            let engine = create_engine();
            let context: HashMap<String, String> = HashMap::new();
            let template = "{{greeting}}, {{name}}!";

            let result = engine.render_template(template, &context);
            assert!(matches!(
                result,
                Err(TemplateError::RenderError(_))
            ));
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

            let result =
                engine.render_template(template, &context).unwrap();
            assert_eq!(result, "& <script>alert('XSS')</script>");
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
        use std::fs::File;
        use std::io::Write;
        use tempfile::tempdir;

        #[test]
        fn test_engine_download_template_files() {
            let engine = create_engine();
            let url = "https://raw.githubusercontent.com/sebastienrousseau/shokunin/main/templates";
            let result = engine.download_template_files(url);
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
            let temp_dir = tempdir().unwrap();
            let layout_path = temp_dir.path().join("layout.html");

            let mut file = File::create(&layout_path).unwrap();
            writeln!(file, "<html><body>{{{{greeting}}}}, {{{{name}}}}</body></html>").unwrap();

            let mut engine = Engine::new(
                temp_dir.path().to_str().unwrap(),
                Duration::from_secs(60),
            );
            let context = Context {
                elements: create_basic_context(),
            };

            let result = engine.render_page(&context, "layout");
            assert!(result.is_ok());
            assert_eq!(
                result.unwrap().trim(),
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
            options.set("title", "My Title");
            assert_eq!(options.get("title"), Some(&"My Title"));
            assert_eq!(options.get("non_existent"), None);
        }

        #[test]
        fn test_page_options_large_context() {
            let mut options = PageOptions::new();
            let mut keys = Vec::new();
            let mut values = Vec::new();
            for i in 0..1000 {
                let key = format!("key{}", i);
                let value = format!("value{}", i);
                keys.push(key);
                values.push(value);
            }
            for i in 0..1000 {
                options.set(&keys[i], &values[i]);
            }

            assert_eq!(options.get("key999"), Some(&"value999"));
            assert_eq!(options.get("key1000"), None); // Key not present
        }
    }

    /// Edge case tests for template rendering.
    mod context_edge_cases_tests {
        use super::*;

        #[test]
        fn test_render_template_invalid_format() {
            let engine = create_engine();
            let context = create_basic_context();
            let template = "{greeting}, {name}!"; // Invalid format (single curly braces)

            let result = engine.render_template(template, &context);
            assert!(matches!(
                result,
                Err(TemplateError::RenderError(_))
            ));
        }

        #[test]
        fn test_render_template_invalid_syntax() {
            let engine = create_engine();
            let context = create_basic_context();
            let invalid_template = "Hello, {{name"; // Missing closing braces

            let result =
                engine.render_template(invalid_template, &context);
            assert!(matches!(
                result,
                Err(TemplateError::RenderError(_))
            ));
        }

        #[test]
        fn test_render_large_template() {
            let engine = create_engine();
            let large_template = "Hello, {{name}}".repeat(1000); // Large template with repetitive pattern
            let context = create_basic_context();

            let result =
                engine.render_template(&large_template, &context);
            assert!(result.is_ok());
            assert!(result.unwrap().contains("Hello, World"));
        }

        #[test]
        fn test_render_template_empty_template() {
            let engine = create_engine();
            let context = create_basic_context();
            let empty_template = ""; // Empty template

            let result =
                engine.render_template(empty_template, &context);
            assert!(matches!(
                result,
                Err(TemplateError::RenderError(_))
            ));
        }
    }
}

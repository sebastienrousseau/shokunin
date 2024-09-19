/// Unit tests for the `Engine` struct and its methods.
#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    /// Tests for template rendering in the `Engine` struct.
    mod render_tests {

        use ssg_template::{Engine, TemplateError};
        use std::collections::HashMap;
        use std::time::Duration;

        /// Test rendering a template with a valid context.
        #[test]
        fn test_engine_render_template() {
            let engine =
                Engine::new("dummy/path", Duration::from_secs(60));
            let mut context = HashMap::new();
            context.insert("name".to_string(), "World".to_string());
            context.insert("greeting".to_string(), "Hello".to_string());

            let template = "{{greeting}}, {{name}}!";
            let result =
                engine.render_template(template, &context).unwrap();
            assert_eq!(result, "Hello, World!");
        }

        /// Test rendering a template with unresolved tags.
        #[test]
        fn test_engine_render_template_unresolved_tags() {
            let engine =
                Engine::new("dummy/path", Duration::from_secs(60));
            let context: HashMap<String, String> = HashMap::new();

            let template = "{{greeting}}, {{name}}!";
            let result = engine.render_template(template, &context);
            assert!(matches!(
                result,
                Err(TemplateError::RenderError(_))
            ));
        }

        /// Test rendering an empty template.
        #[test]
        fn test_engine_render_empty_template() {
            let engine =
                Engine::new("dummy/path", Duration::from_secs(60));
            let context: HashMap<String, String> = HashMap::new();

            let template = "";
            let result = engine.render_template(template, &context);
            assert!(
                matches!(result, Err(TemplateError::RenderError(msg)) if msg == "Template is empty")
            );
        }

        /// Test rendering a template with an empty context.
        #[test]
        fn test_engine_render_empty_context() {
            let engine =
                Engine::new("dummy/path", Duration::from_secs(60));
            let context: HashMap<String, String> = HashMap::new();

            let template = "{{greeting}}, {{name}}!";
            let result = engine.render_template(template, &context);
            assert!(matches!(
                result,
                Err(TemplateError::RenderError(_))
            ));
        }

        /// Test rendering a template with special characters in the context.
        #[test]
        fn test_engine_render_special_characters_in_context() {
            let engine =
                Engine::new("dummy/path", Duration::from_secs(60));
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

        /// Test rendering with a large context and template.
        #[test]
        fn test_engine_large_context() {
            let engine =
                Engine::new("dummy/path", Duration::from_secs(60));
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
        use ssg_template::{Context, Engine, TemplateError};
        use std::time::Duration;

        /// Test downloading template files from a URL.
        ///
        /// Note: This test may fail if there is no internet connection or the URL is unreachable.
        #[test]
        fn test_engine_download_template_files() {
            let engine =
                Engine::new("dummy/path", Duration::from_secs(60));
            let url = "https://raw.githubusercontent.com/sebastienrousseau/shokunin/main/templates";
            let result = engine.download_template_files(url);
            assert!(result.is_ok());
        }

        /// Test rendering with an invalid template path.
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
    }
}

mod page_options_tests {

    use ssg_template::PageOptions;

    /// Test `PageOptions::new` to ensure it initializes an empty HashMap.
    #[test]
    fn test_page_options_new() {
        let options = PageOptions::new();
        assert!(options.elements.is_empty());
    }

    /// Test `PageOptions::set` and `PageOptions::get` to ensure they behave as expected.
    #[test]
    fn test_page_options_set_get() {
        let mut options = PageOptions::new();
        options.set("title", "My Title");
        assert_eq!(options.get("title"), Some(&"My Title"));
        assert_eq!(options.get("non_existent"), None);
    }
}

mod file_operations_tests {
    use ssg_template::{Context, Engine, TemplateError};
    use std::collections::HashMap;
    use std::fs::File;
    use std::io::Write;
    use std::time::Duration;
    use tempfile::tempdir;

    /// Test `render_page` with a valid template path.
    #[test]
    fn test_render_page_valid_path() {
        // Create a temporary directory
        let temp_dir = tempdir().unwrap();
        let layout_path = temp_dir.path().join("layout.html");

        // Create a mock layout file in the temporary directory
        let mut file = File::create(&layout_path).unwrap();
        writeln!(
            file,
            "<html><body>{{{{greeting}}}}, {{{{name}}}}</body></html>"
        )
        .unwrap();

        // Initialize engine with the temporary directory
        let mut engine = Engine::new(
            temp_dir.path().to_str().unwrap(),
            Duration::from_secs(60),
        );

        // Prepare the context
        let mut elements = HashMap::new();
        elements.insert("greeting".to_string(), "Hello".to_string());
        elements.insert("name".to_string(), "World".to_string());
        let context = Context { elements };

        // Render the page with the mock layout
        let result = engine.render_page(&context, "layout");
        assert!(result.is_ok());

        // Trim the result and expected output to avoid newline mismatches
        let unwrapped_result = result.unwrap();
        let rendered_output = unwrapped_result.trim();
        let expected_output = "<html><body>Hello, World</body></html>";

        assert_eq!(rendered_output, expected_output);
    }

    /// Test `render_page` with an invalid template path.
    #[test]
    fn test_render_page_invalid_path() {
        let mut engine =
            Engine::new("invalid/path", Duration::from_secs(60));
        let context = Context {
            elements: HashMap::new(),
        };
        let result = engine.render_page(&context, "nonexistent_layout");
        assert!(matches!(result, Err(TemplateError::Io(_))));
    }

    /// Test `render_page` when the file is missing.
    #[test]
    fn test_render_page_missing_file() {
        let mut engine =
            Engine::new("missing/path", Duration::from_secs(60));
        let context = Context {
            elements: HashMap::new(),
        };
        let result = engine.render_page(&context, "nonexistent_layout");
        assert!(matches!(result, Err(TemplateError::Io(_))));
    }
}

mod context_edge_cases_tests {

    use ssg_template::{Engine, TemplateError};
    use std::collections::HashMap;
    use std::time::Duration;

    /// Test rendering a template with an empty context.
    #[test]
    fn test_render_template_empty_context() {
        let engine = Engine::new("dummy/path", Duration::from_secs(60));
        let context: HashMap<String, String> = HashMap::new();

        let template = "{{greeting}}, {{name}}!";
        let result = engine.render_template(template, &context);
        assert!(matches!(result, Err(TemplateError::RenderError(_))));
    }

    /// Test rendering a template with special characters in the context.
    #[test]
    fn test_render_template_special_characters() {
        let engine = Engine::new("dummy/path", Duration::from_secs(60));
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
}

mod additional_tests {

    use ssg_template::{Context, Engine, PageOptions, TemplateError};
    use std::time::Duration;
    use std::{collections::HashMap, fs::File};
    use tempfile::tempdir;

    /// Test rendering a template with an invalid format.
    #[test]
    fn test_engine_render_template_invalid_format() {
        let engine = Engine::new("dummy/path", Duration::from_secs(60));
        let mut context = HashMap::new();
        context.insert("name".to_string(), "World".to_string());
        context.insert("greeting".to_string(), "Hello".to_string());

        // Invalid format: single curly braces instead of double
        let template = "{greeting}, {name}!";
        let result = engine.render_template(template, &context);
        assert!(matches!(result, Err(TemplateError::RenderError(_))));
    }

    /// Test rendering a page with an empty layout file.
    #[test]
    fn test_render_page_empty_layout_file() {
        // Create a temporary directory
        let temp_dir = tempdir().unwrap();
        let layout_path = temp_dir.path().join("layout.html");

        // Create an empty layout file in the temporary directory
        File::create(&layout_path).unwrap();

        // Initialize engine with the temporary directory
        let mut engine = Engine::new(
            temp_dir.path().to_str().unwrap(),
            Duration::from_secs(60),
        );

        // Prepare the context
        let mut elements = HashMap::new();
        elements.insert("greeting".to_string(), "Hello".to_string());
        elements.insert("name".to_string(), "World".to_string());
        let context = Context { elements };

        // Render the page with the empty layout
        let result = engine.render_page(&context, "layout");
        assert!(matches!(result, Err(TemplateError::RenderError(_))));
    }

    /// Test rendering a page where the layout directory has permission errors.
    #[test]
    fn test_render_page_permission_error() {
        // Simulate a directory with permission issues
        let temp_dir = tempdir().unwrap();
        let layout_path = temp_dir.path().join("layout.html");

        // Create the layout file, but simulate a permission error by not allowing writes
        File::create(&layout_path).unwrap();
        let mut engine = Engine::new(
            "/restricted/directory",
            Duration::from_secs(60),
        );

        let mut elements = HashMap::new();
        elements.insert("greeting".to_string(), "Hello".to_string());
        elements.insert("name".to_string(), "World".to_string());
        let context = Context { elements };

        let result = engine.render_page(&context, "layout");
        assert!(matches!(result, Err(TemplateError::Io(_))));
    }

    /// Test `PageOptions` with a large context.
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

    /// Test rendering a template with an invalid context data type (e.g., integer values).
    #[test]
    fn test_render_template_invalid_context_data_type() {
        let engine = Engine::new("templates/", Duration::from_secs(60));
        let template = "Hello, {{name}}!";
        let mut invalid_context = HashMap::new();
        invalid_context.insert("name".to_string(), "World".to_string()); // Valid
        invalid_context.insert("number".to_string(), "42".to_string()); // Invalid if expecting specific types

        let result = engine.render_template(template, &invalid_context);
        assert!(result.is_ok());
    }

    /// Test render_template error handling with invalid template syntax.
    #[test]
    fn test_render_template_invalid_template_syntax() {
        let engine = Engine::new("templates/", Duration::from_secs(60));
        let invalid_template = "Hello, {{name"; // Missing closing braces
        let mut context = HashMap::new();
        context.insert("name".to_string(), "World".to_string());

        let result = engine.render_template(invalid_template, &context);
        assert!(matches!(result, Err(TemplateError::RenderError(_))));
    }

    /// Test large template rendering.
    #[test]
    fn test_render_large_template() {
        let engine = Engine::new("templates/", Duration::from_secs(60));
        let large_template = "Hello, {{name}}".repeat(1000); // Large template with repetitive pattern
        let mut context = HashMap::new();
        context.insert("name".to_string(), "World".to_string());

        let result = engine.render_template(&large_template, &context);
        assert!(result.is_ok());
        assert!(result.unwrap().contains("Hello, World"));
    }

    /// Test PageOptions invalid set with unexpected data types.
    #[test]
    fn test_page_options_invalid_set() {
        let mut options = PageOptions::new();

        // Try setting invalid values (simulate, as PageOptions expects strings)
        options.set("key1", "value1");
        options.set("key2", "value2");

        assert_eq!(options.get("key1"), Some(&"value1"));
        assert_eq!(options.get("key3"), None); // Ensure invalid key does not exist
    }

    /// Test empty template rendering.
    #[test]
    fn test_render_template_empty_template() {
        let engine = Engine::new("templates/", Duration::from_secs(60));
        let empty_template = ""; // Empty template
        let mut context = HashMap::new();
        context.insert("name".to_string(), "World".to_string());

        let result = engine.render_template(empty_template, &context);
        assert!(matches!(result, Err(TemplateError::RenderError(_))));
    }
}

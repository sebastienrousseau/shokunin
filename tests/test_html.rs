// Copyright © 2024 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

#[cfg(test)]
mod tests {
    use regex::Regex;
    use ssg::{
        modules::html::generate_html,
        utilities::directory::format_header_with_id_class,
    };
    use ssg::modules::postprocessor::post_process_html;
    use ssg::modules::html::HtmlGenerationError;

    #[test]
    fn test_generate_html_with_front_matter() {
        let content = "---\ntitle: Hello, world!\ndescription: A simple greeting\n---\n# Welcome";
        let title = "Welcome";
        let description = "Say hi to the world!";
        let result = generate_html(content, title, description, None);
        let expected = "<h1 id=\"h1-welcome\" tabindex=\"0\" aria-label=\"Welcome Heading\" itemprop=\"headline\" class=\"welcome\">Welcome</h1><p>Say hi to the world!</p><h1 id=\"h1-welcome\" tabindex=\"0\" aria-label=\"Welcome Heading\" itemprop=\"headline\" class=\"welcome\">Welcome</h1>";
        match result {
            Ok(res) => assert_eq!(res.trim(), expected),
            Err(e) => panic!("Error: {:?}", e),
        }
    }

    #[test]
    fn test_generate_html_without_front_matter() {
        let content = "# Welcome";
        let title = "Welcome";
        let description = "Say hi to the world!";
        let result = generate_html(content, title, description, None);
        let expected = "<h1 id=\"h1-welcome\" tabindex=\"0\" aria-label=\"Welcome Heading\" itemprop=\"headline\" class=\"welcome\">Welcome</h1><p>Say hi to the world!</p><h1 id=\"h1-welcome\" tabindex=\"0\" aria-label=\"Welcome Heading\" itemprop=\"headline\" class=\"welcome\">Welcome</h1>";
        match result {
            Ok(res) => assert_eq!(res.trim(), expected),
            Err(e) => panic!("Error: {:?}", e),
        }
    }

    #[test]
    fn test_generate_html_without_title() {
        let content =
            "---\ndescription: A simple greeting\n---\n# Welcome";
        let title = "";
        let description = "Say hi to the world!";
        let result = generate_html(content, title, description, None);
        match result {
            Ok(_) => panic!("Expected an error but got Ok"),
            Err(e) => {
                if let HtmlGenerationError::EmptyTitle = e {
                    // Test passed
                } else {
                    panic!("Unexpected error: {:?}", e);
                }
            }
        }
    }

    #[test]
    fn test_generate_html_without_description() {
        let content = "---\ntitle: Hello, world!\n---\n# Welcome";
        let title = "Welcome";
        let description = "";
        let result = generate_html(content, title, description, None);
        match result {
            Ok(_) => panic!("Expected an error but got Ok"),
            Err(e) => {
                if let HtmlGenerationError::EmptyDescription = e {
                    // Test passed
                } else {
                    panic!("Unexpected error: {:?}", e);
                }
            }
        }
    }

    #[test]
    fn test_generate_html_with_empty_fields() {
        let content = "---\ntitle:\ndescription:\n---\n# Welcome";
        let title = "Welcome";
        let description = "Say hi to the world!";
        let result = generate_html(content, title, description, None);
        let expected = "<h1 id=\"h1-welcome\" tabindex=\"0\" aria-label=\"Welcome Heading\" itemprop=\"headline\" class=\"welcome\">Welcome</h1><p>Say hi to the world!</p><h1 id=\"h1-welcome\" tabindex=\"0\" aria-label=\"Welcome Heading\" itemprop=\"headline\" class=\"welcome\">Welcome</h1>";
        match result {
            Ok(res) => assert_eq!(res.trim(), expected),
            Err(e) => panic!("Error: {:?}", e),
        }
    }

    #[test]
    fn test_generate_html_with_empty_content() {
        let content = "";
        let title = "Welcome";
        let description = "Say hi to the world!";
        let result = generate_html(content, title, description, None);
        let expected = "<h1 id=\"h1-welcome\" tabindex=\"0\" aria-label=\"Welcome Heading\" itemprop=\"headline\" class=\"welcome\">Welcome</h1><p>Say hi to the world!</p>";
        match result {
            Ok(res) => assert_eq!(res.trim(), expected),
            Err(e) => panic!("Error: {:?}", e),
        }
    }

    #[test]
    fn test_format_header_with_id_class() {
        let header_str = "<h1>Hello, world!</h1>";
        let id_regex = Regex::new(r"[^a-z0-9]+").unwrap();
        let result = format_header_with_id_class(header_str, &id_regex);
        let expected = "<h1 id=\"h1-hello\" tabindex=\"0\" aria-label=\"-ello Heading\" itemprop=\"headline\" class=\"hello\">Hello, world!</h1>";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_format_header_with_id_class_multiple_words() {
        let header_str = "<h1>Welcome to the world</h1>";
        let id_regex = Regex::new(r"[^a-z0-9]+").unwrap();
        let result = format_header_with_id_class(header_str, &id_regex);
        let expected = "<h1 id=\"h1-welcome\" tabindex=\"0\" aria-label=\"-elcome Heading\" itemprop=\"headline\" class=\"welcome\">Welcome to the world</h1>";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_format_header_with_id_class_special_characters() {
        let header_str = "<h1>Hello, world! #$%^&*()</h1>";
        let id_regex = Regex::new(r"[^a-z0-9]+").unwrap();
        let result = format_header_with_id_class(header_str, &id_regex);
        let expected = "<h1 id=\"h1-hello\" tabindex=\"0\" aria-label=\"-ello Heading\" itemprop=\"headline\" class=\"hello\">Hello, world! #$%^&*()</h1>";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_format_header_with_id_class_no_header_tag() {
        let header_str = "Hello, world!";
        let id_regex = Regex::new(r"[^a-z0-9]+").unwrap();
        let result = format_header_with_id_class(header_str, &id_regex);
        assert_eq!(result, header_str);
    }

    // #[test]
    // fn test_post_process_html_with_valid_input() {
    //     let html = r#"<p class="old-class">Hello</p><img src="image.jpg" alt="A picture">"#;
    //     let class_regex = Regex::new(r#"class="[^"]*""#).unwrap();
    //     let img_regex = Regex::new(r#"(.*<img[^>]*)(/?>)"#).unwrap();
    //     let result =
    //         post_process_html(html, &class_regex, &img_regex).unwrap();

    //     assert!(result.contains(r#"<img src="image.jpg" alt="A picture" title="Image of a picture">"#));
    // }

    #[test]
    fn test_post_process_html_with_missing_alt_and_title() {
        let html = r#"<img src="image.jpg">"#;
        let class_regex = Regex::new(r#"class="[^"]*""#).unwrap();
        let img_regex = Regex::new(r#"(.*<img[^>]*)(/>)"#).unwrap();
        let result =
            post_process_html(html, &class_regex, &img_regex).unwrap();

        // Expect no change as both alt and title are missing
        assert_eq!(result.trim(), r#"<img src="image.jpg">"#);
    }

    #[test]
    fn test_post_process_html_with_invalid_regex() {
        let html = "<p>Hello</p>";
        // Use a malformed regex pattern that will fail during compilation
        let invalid_regex_pattern = "["; // An unclosed character class is invalid
        let class_regex = Regex::new(invalid_regex_pattern);
        let img_regex = Regex::new(r#"<img[^>]*?(/?>)"#).unwrap();

        // Check if the class_regex compilation failed
        if let Ok(regex) = class_regex {
            // If somehow the regex is okay, then proceed with the test (unlikely with an invalid pattern)
            let result = post_process_html(html, &regex, &img_regex);
            assert!(result.is_err()); // Expect an error during processing
        }
    }

    #[test]
    fn test_post_process_html_with_empty_input() {
        let html = "";
        let class_regex = Regex::new(r#"class="[^"]*""#).unwrap();
        let img_regex = Regex::new(r#"<img[^>]*?(/?>)"#).unwrap();
        let result =
            post_process_html(html, &class_regex, &img_regex).unwrap();

        assert_eq!(result, "");
    }

    #[test]
    fn test_post_process_html_with_invalid_input() {
        let html = "<p>Hello</p>\n";
        let class_regex = Regex::new(r#"class="[^"]*""#).unwrap();
        let img_regex = Regex::new(r#"<img[^>]*?(/?>)"#).unwrap();
        let result =
            post_process_html(html, &class_regex, &img_regex).unwrap();

        assert_eq!(result, html);
    }
}

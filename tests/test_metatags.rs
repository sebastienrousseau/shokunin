// Copyright © 2024 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

#[cfg(test)]
mod tests {
    use regex::Regex;
    use ssg::modules::metatags::generate_metatags;

    // Test general functionality of the generate_metatags function
    #[test]
    fn test_generate_metatags_general() {
        let test_cases = [(
            vec![
                ("description".to_string(), "A blog about Rust programming.".to_string()),
            ],
            "<meta name=\"description\" content=\"A blog about Rust programming.\">",
        ),
        (
            vec![],
            "",
        ),
        (
            vec![("description".to_string(), "My site description".to_string())],
            "<meta name=\"description\" content=\"My site description\">",
        ),
        (
            vec![
                ("description".to_string(), "My site description".to_string()),
                ("keywords".to_string(), "rust,web,ssg".to_string()),
                ("author".to_string(), "John Doe".to_string()),
            ],
            "<meta name=\"description\" content=\"My site description\">\n<meta name=\"keywords\" content=\"rust,web,ssg\">\n<meta name=\"author\" content=\"John Doe\">",
        ),
        (
            vec![
                ("description".to_string(), "My site description".to_string()),
                ("keywords".to_string(), "rust,web,ssg rust,web,ssg".to_string()),
                ("author".to_string(), "John Doe".to_string()),
            ],
            "<meta name=\"description\" content=\"My site description\">\n<meta name=\"keywords\" content=\"rust,web,ssg rust,web,ssg\">\n<meta name=\"author\" content=\"John Doe\">",
        )];

        for (input_metadata, expected_output) in test_cases.iter() {
            let result = generate_metatags(input_metadata);
            assert_eq!(
                result, *expected_output,
                "Mismatch in generated meta tags for general tests"
            );
        }
    }

    // Test for ensuring meta tag order stability
    #[test]
    fn test_generate_metatags_order_stability() {
        let input_metadata = vec![
            ("author".to_string(), "John Doe".to_string()),
            (
                "description".to_string(),
                "A blog about Rust programming.".to_string(),
            ),
            ("keywords".to_string(), "Rust, programming".to_string()),
        ];
        let expected_output = "<meta name=\"author\" content=\"John Doe\">\n<meta name=\"description\" content=\"A blog about Rust programming.\">\n<meta name=\"keywords\" content=\"Rust, programming\">";

        let result = generate_metatags(&input_metadata);

        assert_eq!(result, expected_output, "Generated meta tags do not maintain the input order for order stability tests");

        let regex =
            Regex::new(r#"<meta name="([^"]+)" content="[^"]*">"#)
                .unwrap();
        let found_tags = regex
            .captures_iter(&result)
            .map(|cap| cap[1].to_string())
            .collect::<Vec<_>>();
        let expected_tags = input_metadata
            .iter()
            .map(|(name, _)| name.clone())
            .collect::<Vec<_>>();

        assert_eq!(
            found_tags, expected_tags,
            "Meta tags are not in the expected order"
        );
    }

    // Test basic functionality with a single metadata entry
    #[test]
    fn test_generate_metatags_basic() {
        let test_cases = vec![
            (
                vec![("description".to_string(), "A blog about Rust programming.".to_string())],
                "<meta name=\"description\" content=\"A blog about Rust programming.\">",
            ),
            // Add more basic cases as needed
        ];

        for (input_metadata, expected_output) in test_cases {
            let result = generate_metatags(&input_metadata);
            assert_eq!(
                result, expected_output,
                "Failed on basic metadata input"
            );
        }
    }

    // Test handling of empty and null cases
    #[test]
    fn test_generate_metatags_edge_cases() {
        let test_cases = vec![
            (vec![], ""),
            (
                vec![("".to_string(), "".to_string())],
                "<meta name=\"\" content=\"\">",
            ),
            // Add more edge cases as needed
        ];

        for (input_metadata, expected_output) in test_cases {
            let result = generate_metatags(&input_metadata);
            assert_eq!(
                result, expected_output,
                "Failed on edge case metadata input"
            );
        }
    }

    // Test meta tag order stability and tag order verification
    #[test]
    fn test_generate_metatags_order_stability_order_check() {
        let input_metadata = vec![
            ("author".to_string(), "John Doe".to_string()),
            (
                "description".to_string(),
                "A blog about Rust programming.".to_string(),
            ),
            ("keywords".to_string(), "Rust, programming".to_string()),
        ];
        let expected_order = ["author", "description", "keywords"];
        let result = generate_metatags(&input_metadata);
        let regex =
            Regex::new(r#"<meta name="([^"]+)" content="[^"]*">"#)
                .unwrap();
        let matches = regex
            .captures_iter(&result)
            .map(|cap| cap[1].to_string())
            .collect::<Vec<_>>();

        // Ensure all expected tags are present in the result
        assert_eq!(
            matches.len(),
            expected_order.len(),
            "Mismatch in number of generated meta tags"
        );

        // Verify the order of generated meta tags matches the input order
        for (expected, actual) in
            expected_order.iter().zip(matches.iter())
        {
            assert_eq!(
                expected, actual,
                "Meta tag order does not match expected order"
            );
        }
    }

    // Test behaviour with duplicate keys
    #[test]
    fn test_generate_metatags_duplicate_keys() {
        let input_metadata = vec![
            (
                "description".to_string(),
                "First description".to_string(),
            ),
            ("keywords".to_string(), "Rust, programming".to_string()),
            (
                "description".to_string(),
                "Second description".to_string(),
            ),
        ];
        let expected_output = "<meta name=\"description\" content=\"First description\">\n<meta name=\"keywords\" content=\"Rust, programming\">\n<meta name=\"description\" content=\"Second description\">";

        let result = generate_metatags(&input_metadata);

        assert_eq!(
            result, expected_output,
            "Generated meta tags should handle duplicate keys"
        );
    }
    // Test behaviour with long input values
    #[test]
    fn test_generate_metatags_long_input_values() {
        let long_description = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.".repeat(100);
        let input_metadata = vec![
            ("description".to_string(), long_description),
            ("keywords".to_string(), "Rust, programming".to_string()),
        ];
        let result = generate_metatags(&input_metadata);

        assert!(!result.is_empty(), "Generated meta tags should not be empty for long input values");
    }
    // Test behaviour with empty keys
    #[test]
    fn test_generate_metatags_empty_keys() {
        let input_metadata = vec![
            (
                "description".to_string(),
                "A blog about Rust programming.".to_string(),
            ),
            ("".to_string(), "Empty key".to_string()),
            ("keywords".to_string(), "Rust, programming".to_string()),
        ];
        let expected_output = "<meta name=\"description\" content=\"A blog about Rust programming.\">\n<meta name=\"\" content=\"Empty key\">\n<meta name=\"keywords\" content=\"Rust, programming\">";

        let result = generate_metatags(&input_metadata);

        assert_eq!(
            result, expected_output,
            "Generated meta tags should handle empty keys"
        );
    }
    // Test behaviour with empty values
    #[test]
    fn test_generate_metatags_empty_values() {
        let input_metadata = vec![
            (
                "description".to_string(),
                "A blog about Rust programming.".to_string(),
            ),
            ("keywords".to_string(), "".to_string()),
        ];
        let expected_output = "<meta name=\"description\" content=\"A blog about Rust programming.\">\n<meta name=\"keywords\" content=\"\">";

        let result = generate_metatags(&input_metadata);

        assert_eq!(
            result, expected_output,
            "Generated meta tags should handle empty values"
        );
    }
    // Test behaviour with whitespace handling
    #[test]
    fn test_generate_metatags_whitespace_handling() {
        let input_metadata = vec![
            (
                "description".to_string(),
                "  A blog about Rust programming.  ".to_string(),
            ),
            ("keywords".to_string(), " Rust, programming ".to_string()),
        ];
        let expected_output = "<meta name=\"description\" content=\"A blog about Rust programming.\">\n<meta name=\"keywords\" content=\"Rust, programming\">";

        let result = generate_metatags(&input_metadata);

        assert_eq!(result, expected_output, "Generated meta tags should handle whitespace appropriately");
    }
}

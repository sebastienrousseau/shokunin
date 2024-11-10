// Copyright © 2024 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

#[cfg(test)]
mod tests {
    use staticdatagen::modules::plaintext::generate_plain_text;

    // Test for generating plain text with bold and italics
    #[test]
    fn test_generate_plain_text_with_bold_and_italics() {
        // Given Markdown content with bold and italics
        let content = "**Bold** and *italics*";

        // When generating plain text
        let result = generate_plain_text(
            content,
            "Title",
            "Description",
            "Author",
            "Creator",
            "Keywords",
        );

        // Assert that the result is successful
        assert!(result.is_ok());

        // Retrieve the generated plain text
        let (plain_text, _, _, _, _, _) = result.unwrap();

        // Assert that the generated plain text is correct
        assert_eq!(plain_text, "Bold and italics");
    }

    // Test for generating plain text with an image tag
    #[test]
    fn test_generate_plain_text_with_image_tag() {
        // Given Markdown content with an image tag
        let content =
            "Some text with an image <img src=\"example.jpg\" />";

        // When generating plain text
        let result = generate_plain_text(
            content,
            "Title",
            "Description",
            "Author",
            "Creator",
            "Keywords",
        );

        // Assert that the result is successful
        assert!(result.is_ok());

        // Retrieve the generated plain text
        let (plain_text, _, _, _, _, _) = result.unwrap();

        // Assert that the generated plain text is correct
        assert_eq!(plain_text, "Some text with an image");
    }

    // Test for generating plain text with paragraphs
    #[test]
    fn test_generate_plain_text_with_paragraphs() {
        // Given Markdown content with paragraphs
        let content = "\
Header 1

This is a paragraph.

This is another paragraph.";

        // When generating plain text
        let result = generate_plain_text(
            content,
            "Title",
            "Description",
            "Author",
            "Creator",
            "Keywords",
        );

        // Assert that the result is successful
        assert!(result.is_ok());

        // Retrieve the generated plain text
        let (plain_text, _, _, _, _, _) = result.unwrap();

        // Assert that the generated plain text is correct
        assert_eq!(
            plain_text,
            "Header 1This is a paragraph.This is another paragraph."
        );
    }

    // Test for generating plain text with a link reference
    #[test]
    fn test_generate_plain_text_with_link_reference() {
        // Given Markdown content with a link reference
        let content = "\
            Some text [link][1].

            [1]: https://example.com";

        // When generating plain text
        let result = generate_plain_text(
            content,
            "Title",
            "Description",
            "Author",
            "Creator",
            "Keywords",
        );

        // Assert that the result is successful
        assert!(result.is_ok());

        // Retrieve the generated plain text
        let (plain_text, _, _, _, _, _) = result.unwrap();

        // Assert that the generated plain text is correct
        assert_eq!(
            plain_text,
            "Some text [ link ] [ 1 ] .[1]: https://example.com"
        );
    }
}

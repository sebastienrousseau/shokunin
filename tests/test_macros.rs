// Copyright © 2024 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

#[cfg(test)]
mod tests {
    use quick_xml::Writer;
    use rlg::{log_format::LogFormat, log_level::LogLevel};
    use ssg::{macro_check_directory, macro_cleanup_directories, macro_create_directories, macro_execute_and_log, macro_generate_metatags, macro_generate_rss, macro_generate_tags_from_fields, macro_generate_tags_from_list, macro_log_complete, macro_log_error, macro_log_info, macro_log_start, macro_metadata_option, macro_set_rss_data_fields, macro_write_element, macros::shell_macros::CommandExecutor, models::data::RssData, modules::metatags::{generate_custom_meta_tags, load_metatags}, utilities::escape::escape_html_entities};
    use std::path::Path;
    use std::{collections::HashMap, io::Cursor};

    #[test]
    fn test_macro_check_directory_existing_directory() {
        // Arrange
        let temp_dir = tempfile::tempdir().unwrap();

        // Act
        macro_check_directory!(temp_dir.path(), "temp_dir");

        // Assert
        assert!(temp_dir.path().is_dir());
    }

    #[test]
    fn test_macro_check_directory_nonexistent_directory() {
        // Arrange
        let temp_dir = tempfile::tempdir().unwrap();
        let new_dir = temp_dir.path().join("new_dir");

        // Act
        macro_check_directory!(&new_dir, "new_dir");

        // Assert
        assert!(new_dir.is_dir());
    }

    #[test]
    fn test_macro_cleanup_directories() {
        // Arrange
        let dir1 = Path::new("dir1");
        let dir2 = Path::new("dir2");

        // Act
        macro_cleanup_directories!(dir1, dir2);

        // No assertions, the test ensures no errors occur during cleanup
    }

    #[test]
    fn test_macro_create_directories() {
        // Arrange
        let temp_dir = tempfile::tempdir().unwrap();
        let dir1 = temp_dir.path().join("dir1");
        let dir2 = temp_dir.path().join("dir2");

        // Act
        macro_create_directories!(&dir1, &dir2).unwrap();

        // Assert
        assert!(dir1.exists());
        assert!(dir2.exists());

        // Cleanup
        std::fs::remove_dir(&dir1).unwrap();
        std::fs::remove_dir(&dir2).unwrap();
    }

    #[test]
    fn test_macro_metadata_option_existing_key() {
        // Arrange
        let mut metadata = HashMap::new();
        metadata.insert("key", "value");

        // Act
        let value = macro_metadata_option!(metadata, "key");

        // Assert
        assert_eq!(value, "value");
    }

    #[test]
    fn test_macro_write_element_empty_value() {
        // Arrange
        let mut writer = Writer::new(Cursor::new(Vec::new()));

        // Act
        macro_write_element!(writer, "tag", "").unwrap();

        // Assert
        let result = writer.into_inner().into_inner();
        let expected = Vec::<u8>::new();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_macro_write_element_nonempty_value() {
        // Arrange
        let mut writer = Writer::new(Cursor::new(Vec::new()));

        // Act
        macro_write_element!(writer, "tag", "value").unwrap();

        // Assert
        let result = writer.into_inner().into_inner();
        let expected = b"<tag>value</tag>".to_vec();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_macro_generate_metatags() {
        // Test with multiple key-value pairs
        let metatags = macro_generate_metatags!(
            "description",
            "This is a description",
            "keywords",
            "rust,macros,metatags"
        );
        assert!(metatags.contains("description"));
        assert!(metatags.contains("keywords"));

        // Validate escaping/encoding of values
        let metatags = macro_generate_metatags!(
            "description",
            "<This is a description>",
            "keywords",
            "&rust,macros,metatags&"
        );
        assert!(metatags.contains("&lt;This is a description&gt;"));
        assert!(metatags.contains("&amp;rust,macros,metatags&amp;"));

        // Pass invalid types for keys and values (compile-time check)
        // e.g., macro_generate_metatags!(description, 123, keywords, true);
    }

    #[test]
    fn test_macro_generate_tags_from_list() {
        // Arrange
        let mut metadata = HashMap::new();
        metadata.insert(
            String::from("description"),
            String::from("This is a description"),
        );
        metadata.insert(
            String::from("keywords"),
            String::from("rust,macros,metatags"),
        );

        let tag_names = &["description", "keywords"];

        // Act
        let html_meta_tags =
            macro_generate_tags_from_list!(tag_names, &metadata);

        // Assert
        assert!(html_meta_tags.contains("This is a description"));
        assert!(html_meta_tags.contains("rust,macros,metatags"));
    }

    #[test]
    fn test_macro_generate_tags_from_fields() {
        // Arrange
        let mut metadata = HashMap::new();
        metadata.insert(
            "description".to_string(),
            "This is a description".to_string(),
        );
        metadata.insert(
            "keywords".to_string(),
            "rust,macros,metatags".to_string(),
        );

        // Test different field mappings
        let html_meta_tags = macro_generate_tags_from_fields!(
            _metadata, metadata,
            "description" => description, "keywords" => keywords
        );

        // Validate output tags
        assert!(html_meta_tags.contains("This is a description"));
        assert!(html_meta_tags.contains("rust,macros,metatags"));
        assert!(html_meta_tags.contains("<meta name=\"description\" content=\"This is a description\">"));
        assert!(html_meta_tags.contains(
            "<meta name=\"keywords\" content=\"rust,macros,metatags\">"
        ));

        // Pass fields not in metadata
        let html_meta_tags = macro_generate_tags_from_fields!(
            _metadata, metadata,
            "author" => author, "created" => created
        );
        assert!(!html_meta_tags.contains("author"));
        assert!(!html_meta_tags.contains("created"));
    }

    #[test]
    fn test_macro_log_info() {
        // Arrange
        let level = LogLevel::INFO;
        let component = "TestComponent";
        let description = "Test description";
        let format = LogFormat::CLF;

        // Act
        let log =
            macro_log_info!(&level, component, description, &format);

        // Assert
        assert_eq!(log.level, level);
        assert_eq!(log.component, component);
        assert_eq!(log.description, description);
        assert_eq!(log.format, format);
    }

    #[test]
fn test_macro_generate_rss() -> Result<(), Box<dyn std::error::Error>> {
    use quick_xml::Writer;
    use std::io::Cursor;
    use ssg::macro_generate_rss;

    // Create an instance of RssData
    let rss_data = RssData {
        atom_link: "http://example.com/rss".to_string(),
        author: "joesmith@example.com (Joe Smith)".to_string(),
        category: "Test Category".to_string(),
        copyright: "2024".to_string(),
        description: "Test Description".to_string(),
        docs: "http://example.com/rss".to_string(),
        generator: "RSS Generator".to_string(),
        image: "http://example.com/image.jpg".to_string(),
        item_guid: "http://example.com/item".to_string(),
        item_description: "Test Item Description".to_string(),
        item_link: "http://example.com/item".to_string(),
        item_pub_date: "Wed, 02 Oct 2002 15:00:00 +0200".to_string(),
        item_title: "Test Item Title".to_string(),
        language: "en-us".to_string(),
        last_build_date: "Wed, 02 Oct 2002 15:00:00 +0200".to_string(),
        link: "http://example.com".to_string(),
        managing_editor: "John Doe".to_string(),
        pub_date: "Wed, 02 Oct 2002 15:00:00 +0200".to_string(),
        title: "Test Title".to_string(),
        ttl: "60".to_string(),
        webmaster: "joesmith@example.com (Joe Smith)".to_string(),
    };

    // Create a Writer instance with a Cursor
    let mut writer = Writer::new(Cursor::new(Vec::new()));

    // Call the macro
    #[allow(clippy::question_mark)]
    if let Err(err) = macro_generate_rss!(&mut writer, &rss_data) {
        return Err(err);
    }

    // Convert the writer into bytes and then to a UTF-8 String for trimming
    let result_bytes = writer.into_inner().into_inner();
    let result_string = String::from_utf8(result_bytes)?;
    let trimmed_result = result_string.trim();

    // Define the expected XML as a string and trim it
    let expected_str = "<?xml version=\"1.0\" encoding=\"utf-8\"?><rss version=\"2.0\" xmlns:atom=\"http://www.w3.org/2005/Atom\"><channel><atom:link href=\"http://example.com/rss\" rel=\"self\" type=\"application/rss+xml\" /><title>Test Title</title><link>http://example.com</link><description>Test Description</description><language>en-us</language><pubDate>Wed, 02 Oct 2002 15:00:00 +0200</pubDate><lastBuildDate>Wed, 02 Oct 2002 15:00:00 +0200</lastBuildDate><docs>http://example.com/rss</docs><generator>RSS Generator</generator><managingEditor>John Doe</managingEditor><webMaster>joesmith@example.com (Joe Smith)</webMaster><category>Test Category</category><ttl>60</ttl><image><url>http://example.com/image.jpg</url><title>Test Title</title><link>http://example.com</link></image><item><author>joesmith@example.com (Joe Smith)</author><description>Test Item Description</description><guid>http://example.com/item</guid><link>http://example.com/item</link><pubDate>Wed, 02 Oct 2002 15:00:00 +0200</pubDate><title>Test Item Title</title></item></channel></rss>".trim();
    let trimmed_expected = expected_str.trim();

    let normalized_actual = trimmed_result.replace(" />", "/>");
    let normalized_expected = trimmed_expected.replace(" />", "/>");

    // Convert trimmed strings back to byte arrays for comparison
    assert_eq!(normalized_actual, normalized_expected, "The XML outputs do not match.");


    Ok(())
}



    #[test]
    fn test_macro_set_rss_data_fields() {
        let mut rss_data = RssData::default();
        macro_set_rss_data_fields!(rss_data, title, "Test Title");
        macro_set_rss_data_fields!(rss_data, link, "http://example.com");
        macro_set_rss_data_fields!(rss_data, description, "Test Description");

        assert_eq!(rss_data.title, "Test Title");
        assert_eq!(rss_data.link, "http://example.com");
        assert_eq!(rss_data.description, "Test Description");
    }
    #[test]
    fn test_command_executor_new_default_interpreter() {
        let executor = CommandExecutor::new::<&str>(None);
        let command = executor.command;
        assert_eq!(command.get_program(), "sh");
    }
    #[test]
    fn test_command_executor_new_custom_interpreter() {
        let interpreter = "bash";
        let executor = CommandExecutor::new(Some(interpreter));
        let command = executor.command;
        assert_eq!(command.get_program(), interpreter);
    }

    #[test]
    fn test_command_executor_execute_success() {
        let mut executor = CommandExecutor::new(None::<&str>);
        executor.command("echo hello");
        let result = executor.execute();
        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(String::from_utf8_lossy(&output.stdout), "hello\n");
    }

    

    #[test]
    fn test_macro_execute_and_log_success() {
        // Test successful command execution
        let result = macro_execute_and_log!(
            "echo hello", "example_pkg", "list_directory",
            "Listing directory contents...", "Listing directory completed successfully.",
            "Listing directory failed.", Some("output"), Some("sh")
        );
        assert!(result.is_ok());
    }


    #[test]
    fn test_macro_log_start() {
        let message = "Listing directory contents...";
        // No need to test the actual logging output, just check if the macro compiles
        macro_log_start!(operation, message);
    }

    #[test]
    fn test_macro_log_complete() {
        let message = "Listing directory completed successfully.";
        // No need to test the actual logging output, just check if the macro compiles
        macro_log_complete!(operation, message);
    }

    #[test]
    fn test_macro_log_error() {
        let message = "Listing directory failed.";
        // No need to test the actual logging output, just check if the macro compiles
        macro_log_error!(operation, message);
    }
}

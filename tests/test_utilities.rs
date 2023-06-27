#[cfg(test)]
mod tests {

    use comrak::ComrakOptions;
    use quick_xml::Writer;
    use regex::Regex;
    use ssg::utilities::{
        backup_file, cleanup_directory, create_comrak_options,
        create_directory, directory, extract_front_matter,
        find_html_files, format_header_with_id_class, minify_html,
        minify_html_files, move_output_directory, to_title_case,
        update_class_attributes, write_element, write_minified_html,
    };
    use std::io::Cursor;
    use std::{
        error::Error,
        fs::{self, File},
        io::{self, Read, Write},
        path::{Path, PathBuf},
    };
    use tempfile::tempdir;

    #[test]
    fn test_directory_existing_directory() {
        let temp = tempdir().unwrap();
        let temp_path = temp.path().to_owned();
        let name = "existing_directory";
        let result = directory(&temp_path, name);
        assert!(
            result.is_ok(),
            "Should not return an error for an existing directory"
        );
    }

    #[test]
    fn test_directory_existing_file() {
        let temp = tempdir().unwrap();
        let file_path = temp.path().join("file.txt");
        fs::write(&file_path, "Some content").unwrap();

        let name = "existing_file";
        let result = directory(&file_path, name);
        assert!(
            result.is_err(),
            "Should return an error for an existing file"
        );
    }

    #[test]
    fn test_directory_create_directory() {
        let temp = tempdir().unwrap();
        let new_dir = temp.path().join("new_directory");
        let name = "new_directory";
        let result = directory(&new_dir, name);
        assert!(
            result.is_ok(),
            "Should not return an error for a newly created directory"
        );
        assert!(
            new_dir.exists() && new_dir.is_dir(),
            "New directory should be created"
        );
    }

    #[test]
    fn test_directory_create_nested_directory() {
        let temp = tempdir().unwrap();
        let new_nested_dir =
            temp.path().join("nested").join("directory");
        let name = "nested_directory";
        let result = directory(&new_nested_dir, name);
        assert!(
            result.is_ok(),
            "Should not return an error for a newly created nested directory"
        );
        assert!(
            new_nested_dir.exists() && new_nested_dir.is_dir(),
            "New nested directory should be created"
        );
    }

    #[test]
    fn test_move_output_directory() -> std::io::Result<()> {
        // Setup: Create a dummy output directory and a dummy file inside it.
        let out_dir = Path::new("temp_out_dir");
        let dummy_file = out_dir.join("dummy_file.txt");
        fs::create_dir_all(out_dir)?;
        fs::File::create(&dummy_file)?;

        // Call the function to test.
        let site_name = "My Test Site";
        move_output_directory(site_name, out_dir)?;

        // Check that the output directory has been moved to the expected location.
        let public_dir = Path::new("public");
        let expected_new_dir =
            public_dir.join(site_name.replace(' ', "_"));
        assert!(
            expected_new_dir.exists(),
            "The directory was not moved to the expected location."
        );

        // Check that the dummy file still exists at the new location.
        let expected_new_file = expected_new_dir.join("dummy_file.txt");
        assert!(expected_new_file.exists(), "The contents of the directory were not preserved during the move.");

        // Cleanup: Remove the public directory that was created during the test.
        fs::remove_dir_all(public_dir)?;

        Ok(())
    }

    #[test]
    fn test_minify_html_files() -> std::io::Result<()> {
        // Setup: Create a dummy output directory and a dummy HTML file inside it.
        let out_dir = Path::new("tests/test_output");
        let dummy_file = out_dir.join("dummy_file.html");
        fs::create_dir_all(out_dir)?;
        let mut file = fs::File::create(&dummy_file)?;
        file.write_all(
            b"<html>\n<body>\nHello, World!\n</body>\n</html>",
        )?;

        // Call the function to test.
        minify_html_files(out_dir)?;

        // Check that the HTML file still exists at the original location.
        assert!(
            dummy_file.exists(),
            "The original HTML file does not exist."
        );

        // Check that the HTML file has been minified.
        // This assumes that your minify_html function simply removes newline characters.
        // Update this to match the actual behavior of your minify_html function.
        let minified_contents = fs::read_to_string(&dummy_file)?;
        assert_eq!(
            minified_contents,
            "<html><body>Hello, World!</body></html>"
        );

        // Cleanup: Remove the temp output directory that was created during the test.
        fs::remove_dir_all(out_dir)?;

        Ok(())
    }

    #[test]
    fn test_find_html_files() -> std::io::Result<()> {
        // Setup: Create a directory with a few .html files and a subdirectory
        // with a few more .html files.
        let dir = Path::new("temp_dir");
        let sub_dir = dir.join("sub_dir");

        fs::create_dir_all(&sub_dir)?;

        let file_paths = vec![
            dir.join("file1.html"),
            dir.join("file2.html"),
            sub_dir.join("file3.html"),
            sub_dir.join("file4.html"),
        ];

        for file_path in &file_paths {
            let mut file = File::create(file_path)?;
            file.write_all(b"<html></html>")?;
        }

        // Call the function to test.
        let found_html_files = find_html_files(dir)?;

        // Check that the function found all of the .html files.
        let mut expected_html_files: Vec<PathBuf> = file_paths.clone();
        expected_html_files.sort_unstable();
        let mut found_html_files_sorted = found_html_files.clone();
        found_html_files_sorted.sort_unstable();

        assert_eq!(
            found_html_files_sorted,
            expected_html_files,
            "The function did not find all of the expected .html files."
        );

        // Cleanup: Remove the directory that was created during the test.
        fs::remove_dir_all(dir)?;

        Ok(())
    }
    #[test]
    fn test_minify_html() -> io::Result<()> {
        // Setup: Create a dummy HTML file.
        let file_path = Path::new("temp_html_file.html");
        let mut file = File::create(&file_path)?;
        write!(file, "<!DOCTYPE html>\n<html>\n<head>\n    <title>Test Page</title>\n</head>\n<body>\n    <h1>Hello, world!</h1>\n</body>\n</html>")?;

        // Call the function to test.
        let minified_content = minify_html(file_path)?;

        // Check that the HTML has been minified.
        // This check will depend on the exact settings you're using in your minify function.
        // Here, we're just checking that the minified content is shorter than the original content.
        let original_content = fs::read_to_string(file_path)?;
        assert!(
            minified_content.len() < original_content.len(),
            "The HTML was not minified."
        );

        // Cleanup: Remove the HTML file that was created during the test.
        fs::remove_file(file_path)?;

        Ok(())
    }
    #[test]
    fn test_backup_file() -> std::io::Result<()> {
        // Setup: Create a dummy file.
        let file_path = Path::new("temp_file.txt");
        let mut file = fs::File::create(&file_path)?;
        file.write_all(b"Some text for the dummy file")?;

        // Call the function to test.
        let backup_path = backup_file(&file_path)?;

        // Check that the backup file was created and that it has the expected content.
        assert!(backup_path.exists(), "Backup file was not created.");
        let backup_content = fs::read_to_string(&backup_path)?;
        assert_eq!(
            backup_content, "Some text for the dummy file",
            "Backup file content does not match original file content."
        );

        // Cleanup: Remove the original and backup file.
        fs::remove_file(file_path)?;
        fs::remove_file(backup_path)?;

        Ok(())
    }
    #[test]
    fn test_write_minified_html() -> std::io::Result<()> {
        // Setup: Define a dummy file path and minified HTML.
        let file_path = Path::new("temp_minified_html_file.html");
        let minified_html = "<!DOCTYPE html><html><head><title>Test</title></head><body><p>Hello, World!</p></body></html>";

        // Call the function to test.
        write_minified_html(file_path, minified_html)?;

        // Check that the file now exists.
        assert!(file_path.exists(), "The file was not created.");

        // Check that the file contains the expected minified HTML.
        let mut file = fs::File::open(file_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        assert_eq!(
            contents, minified_html,
            "The file does not contain the expected minified HTML."
        );

        // Cleanup: Remove the file that was created during the test.
        fs::remove_file(file_path)?;

        Ok(())
    }

    #[test]
    fn test_cleanup_directory_success() -> Result<(), Box<dyn Error>> {
        // Setup: Create some dummy directories with dummy files in them.
        let dir1 = Path::new("temp_dir1");
        let dummy_file1 = dir1.join("dummy_file1.txt");
        fs::create_dir_all(dir1)?;
        fs::File::create(&dummy_file1)?;

        let dir2 = Path::new("temp_dir2");
        let dummy_file2 = dir2.join("dummy_file2.txt");
        fs::create_dir_all(dir2)?;
        fs::File::create(&dummy_file2)?;

        let directories: Vec<&Path> = vec![dir1, dir2];

        // Call the function to test.
        cleanup_directory(&directories)?;

        // Check that the directories no longer exist.
        assert!(!dir1.exists(), "The directory1 was not cleaned up.");
        assert!(!dir2.exists(), "The directory2 was not cleaned up.");

        Ok(())
    }

    #[test]
    fn test_cleanup_directory_ignore_nonexistent(
    ) -> Result<(), Box<dyn Error>> {
        // Setup: Use a path that doesn't exist.
        let nonexistent_dir = Path::new("nonexistent_dir");
        let directories: Vec<&Path> = vec![nonexistent_dir];

        // Call the function to test.
        cleanup_directory(&directories)?;

        // Check that we didn't get an error even though the directory doesn't exist.
        assert!(
            !nonexistent_dir.exists(),
            "The nonexistent directory was somehow created."
        );

        Ok(())
    }
    #[test]
    fn test_create_directory() -> Result<(), Box<dyn Error>> {
        // Setup: Define some directories to test with.
        let dir1 = Path::new("tests/temp_dir1");
        let dir2 = Path::new("tests/temp_dir2");
        let dir3 = Path::new("tests/temp_dir3");

        // Create dir1 and dir2, but leave dir3 non-existent for now.
        fs::create_dir(dir1)?;
        fs::create_dir(dir2)?;

        let dirs = vec![dir1, dir2, dir3];

        // Call the function to test.
        create_directory(&dirs)?;

        // Check that all directories exist.
        assert!(dir1.exists(), "dir1 should still exist.");
        assert!(dir2.exists(), "dir2 should still exist.");
        assert!(dir3.exists(), "dir3 should have been created.");

        // Cleanup: Remove the directories that were created during the test.
        fs::remove_dir(dir1)?;
        fs::remove_dir(dir2)?;
        fs::remove_dir(dir3)?;

        Ok(())
    }
    #[test]
    fn test_write_element() -> Result<(), Box<dyn std::error::Error>> {
        let mut writer = Writer::new(Cursor::new(Vec::new()));
        let name = "element";
        let value = "value";

        write_element(&mut writer, name, value)?;

        let result =
            String::from_utf8(writer.into_inner().into_inner())?;
        assert_eq!(result, "<element>value</element>");

        Ok(())
    }

    #[test]
    fn test_write_element_empty_value(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut writer = Writer::new(Cursor::new(Vec::new()));
        let name = "element";
        let value = "";

        write_element(&mut writer, name, value)?;

        let result =
            String::from_utf8(writer.into_inner().into_inner())?;
        assert_eq!(result, "");

        Ok(())
    }

    #[test]
    fn test_write_element_special_chars(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut writer = Writer::new(Cursor::new(Vec::new()));
        let name = "element";
        let value = "<>&\"'";

        write_element(&mut writer, name, value)?;

        let result =
            String::from_utf8(writer.into_inner().into_inner())?;
        assert_eq!(result, "<element><>&\"'</element>");

        Ok(())
    }

    #[test]
    fn to_title_case_single_word() {
        assert_eq!(to_title_case("hello"), "Hello");
        assert_eq!(to_title_case("WORLD"), "WORLD");
        assert_eq!(to_title_case("rust"), "Rust");
    }

    #[test]
    fn to_title_case_multiple_words() {
        assert_eq!(to_title_case("hello world"), "Hello World");
        assert_eq!(to_title_case("HELLO WORLD"), "HELLO WORLD");
        assert_eq!(to_title_case("hello WORLD"), "Hello WORLD");
        assert_eq!(
            to_title_case("Rust programming language"),
            "Rust Programming Language"
        );
    }

    #[test]
    fn to_title_case_empty_string() {
        assert_eq!(to_title_case(""), "");
    }

    #[test]
    fn to_title_case_only_spaces() {
        assert_eq!(to_title_case(""), "");
    }

    #[test]
    fn to_title_case_leading_trailing_spaces() {
        assert_eq!(to_title_case(" hello "), " Hello ");
        assert_eq!(to_title_case(" hello world "), " Hello World ");
    }

    #[test]
    fn test_format_header_with_id_class() {
        let id_regex = Regex::new(r"[^a-z0-9]+").unwrap();

        let test_cases = vec![
            (
                "<h1>This is a test</h1>",
                "<h1 id=\"h1-this-is-a-test\" class=\"h1-this-is-a-test\">This is a test</h1>"
            ),
            (
                "<h2>Another Test</h2>",
                "<h2 id=\"h2-another-test\" class=\"h2-another-test\">Another Test</h2>"
            ),
            (
                "<h3>Test with special characters!@#$%^&*</h3>",
                "<h3 id=\"h3-test-with-special-characters\" class=\"h3-test-with-special-characters\">Test with special characters!@#$%^&*</h3>"
            ),
            (
                "<h1>Test with multiple     spaces</h1>",
                "<h1 id=\"h1-test-with-multiple-spaces\" class=\"h1-test-with-multiple-spaces\">Test with multiple     spaces</h1>"
            ),
            (
                "<h1>Test_with_underscores</h1>",
                "<h1 id=\"h1-test-with-underscores\" class=\"h1-test-with-underscores\">Test_with_underscores</h1>"
            ),
            (
                "<h1>Test-with-dashes</h1>",
                "<h1 id=\"h1-test-with-dashes\" class=\"h1-test-with-dashes\">Test-with-dashes</h1>"
            ),
        ];

        for (input, expected_output) in test_cases {
            assert_eq!(
                format_header_with_id_class(input, &id_regex),
                expected_output,
            );
        }
    }

    #[test]
    fn test_extract_front_matter_yaml() {
        let content = "---\ntitle: Hello\n---\nThis is the content.";
        assert_eq!(
            extract_front_matter(content),
            "This is the content."
        );
    }

    #[test]
    fn test_extract_front_matter_toml() {
        let content = "+++\ntitle = 'Hello'\n+++\nThis is the content.";
        assert_eq!(
            extract_front_matter(content),
            "This is the content."
        );
    }

    #[test]
    fn test_extract_front_matter_json() {
        let content =
            "{\n\"title\": \"Hello\"\n}\nThis is the content.";
        assert_eq!(
            extract_front_matter(content),
            "\nThis is the content."
        );
    }

    #[test]
    fn test_extract_front_matter_none() {
        let content = "This is the content.";
        assert_eq!(
            extract_front_matter(content),
            "This is the content."
        );
    }

    #[test]
    fn test_extract_front_matter_incomplete_yaml() {
        let content = "---\ntitle: Hello\nThis is the content.";
        assert_eq!(extract_front_matter(content), "");
    }

    #[test]
    fn test_extract_front_matter_incomplete_toml() {
        let content = "+++\ntitle = 'Hello'\nThis is the content.";
        assert_eq!(extract_front_matter(content), "");
    }

    #[test]
    fn test_extract_front_matter_incomplete_json() {
        let content = "{\n\"title\": \"Hello\"\nThis is the content.";
        assert_eq!(extract_front_matter(content), "");
    }

    #[test]
    fn test_create_comrak_options() {
        let options = create_comrak_options();

        assert_eq!(options.extension.autolink, true);
        assert_eq!(options.extension.description_lists, true);
        assert_eq!(options.extension.footnotes, true);
        assert_eq!(
            options.extension.front_matter_delimiter,
            Some("---".to_owned())
        );
        assert_eq!(options.extension.header_ids, Some("".to_string()));
        assert_eq!(options.extension.strikethrough, true);
        assert_eq!(options.extension.superscript, true);
        assert_eq!(options.extension.table, true);
        assert_eq!(options.extension.tagfilter, true);
        assert_eq!(options.extension.tasklist, true);
        assert_eq!(options.parse.smart, true);
        assert_eq!(options.render.github_pre_lang, true);
        assert_eq!(options.render.hardbreaks, false);
        assert_eq!(options.render.unsafe_, true);
    }

    #[test]
    fn test_default_comrak_options() {
        let options = ComrakOptions::default();

        assert_eq!(options.extension.autolink, false);
        assert_eq!(options.extension.description_lists, false);
        assert_eq!(options.extension.footnotes, false);
        assert_eq!(options.extension.front_matter_delimiter, None);
        assert_eq!(options.extension.header_ids, None);
        assert_eq!(options.extension.strikethrough, false);
        assert_eq!(options.extension.superscript, false);
        assert_eq!(options.extension.table, false);
        assert_eq!(options.extension.tagfilter, false);
        assert_eq!(options.extension.tasklist, false);
        assert_eq!(options.parse.smart, false);
        assert_eq!(options.render.github_pre_lang, false);
        assert_eq!(options.render.hardbreaks, false);
        assert_eq!(options.render.unsafe_, false);
    }

    #[test]
    fn test_update_class_attributes_no_class() {
        let line = "<img src=\"test.png\" alt=\"test\" />";
        let class_regex =
            Regex::new(r"\.class=&quot;([^&]*)&quot;").unwrap();
        let img_regex = Regex::new(r"(img[^>]*)\s?/>").unwrap();

        assert_eq!(
            update_class_attributes(line, &class_regex, &img_regex),
            String::from(line)
        );
    }

    #[test]
    fn test_update_class_attributes_with_class() {
        let line = "<img src=\"test.png\" alt=\"test\" class=\"test_class\" />";
        let class_regex =
            Regex::new(r"\.class=&quot;([^&]*)&quot;").unwrap();
        let img_regex = Regex::new(r"(img[^>]*)\s?/>").unwrap();

        assert_eq!(
            update_class_attributes(line, &class_regex, &img_regex),
            String::from(line)
        );
    }

    #[test]
    fn test_update_class_attributes_error_handling() {
        let line = "<img src=\"test.png\" alt=\"test\" />";
        let class_regex =
            Regex::new(r"\.class=&quot;([^&]*)&quot;").unwrap();
        let img_regex = Regex::new(r"(img[^>]*)\s?/>").unwrap();

        assert_eq!(
            update_class_attributes(
                ".class=&quot;non_matching_class&quot;",
                &class_regex,
                &img_regex
            ),
            String::from(".class=&quot;non_matching_class&quot;")
        );

        assert_eq!(
            update_class_attributes(line, &class_regex, &img_regex),
            String::from(line)
        );
    }
}

#[cfg(test)]
mod tests {
    use ssg::{file::File, generate_navigation};

    #[test]
    fn test_generate_navigation() {
        let expected_output =
            "<li><a href=\"file1.html\">file1</a></li>";
        let files = vec![File::new(
            "file1.md",
            "Content1".to_string(),
            "Content1".to_string(),
        )];
        let actual_output = generate_navigation(&files[..]);
        assert_eq!(actual_output, expected_output);
    }

    #[test]
    fn test_generate_navigation_with_empty_files() {
        let files: Vec<File> = Vec::new();
        let expected_output = "";
        assert_eq!(generate_navigation(&files), expected_output);
    }
}

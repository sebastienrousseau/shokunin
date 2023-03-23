#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;

    use ssg::create_output_directory;

    #[test]
    fn test_create_output_directory_with_existing_directory() {
        let out_dir = "tests/test_content/output1";
        assert!(create_output_directory(Path::new(out_dir)).is_ok());
        fs::remove_dir_all(out_dir).unwrap();
    }

    #[test]
    fn test_create_output_directory_with_missing_directory() {
        let out_dir = "tests/test_content/nonexistent_output";
        assert!(create_output_directory(Path::new(out_dir)).is_ok());
        fs::remove_dir_all(out_dir).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use ssg::read_files;

    #[test]
    fn test_read_files_with_existing_files() {
        let src_dir = "tests/test_content/";
        assert!(read_files(Path::new(src_dir)).is_ok());
    }

    #[test]
    fn test_read_files_with_missing_directory() {
        let src_dir = "tests/test_data/nonexistent_content";
        assert!(read_files(Path::new(src_dir)).is_err());
    }
}

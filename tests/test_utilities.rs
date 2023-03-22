#[cfg(test)]
mod tests {
    use ssg::utilities::directory;
    use std::fs;
    use std::path::PathBuf;
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
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use ssg::delete_previous_directory;

    #[test]
    fn test_delete_previous_directory_success() {
        let out_dir = Path::new("tests/test_data/out");
        let site_name = "Test Site";
        let result = delete_previous_directory(out_dir, site_name);
        if let Err(ref error) = result {
            eprintln!("{}", error);
        }
        assert!(result.is_err());
    }

    #[test]
    fn test_delete_previous_directory_with_missing_out_dir() {
        let out_dir =
            Path::new("tests/test_content/nonexistent_out_dir");
        let site_name = "Test Site";
        assert!(delete_previous_directory(out_dir, site_name).is_err());
    }

    #[test]
    fn test_delete_previous_directory_with_missing_site_dir() {
        let out_dir = Path::new("tests/test_content/out");
        let site_name = "nonexistent_site";
        assert!(delete_previous_directory(out_dir, site_name).is_err());
    }
}

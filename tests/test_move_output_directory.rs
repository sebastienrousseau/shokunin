#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;

    use ssg::move_output_directory;

    #[test]
    fn test_move_output_directory() {
        fs::create_dir("tests/test_output").unwrap();
        let out_dir = Path::new("tests/test_output");
        let site_name = "Test Site";
        let public_dir = Path::new("public");
        let expected_dir = public_dir.join("Test_Site");
        assert!(move_output_directory(out_dir, site_name).is_ok());
        assert!(expected_dir.exists());
        assert!(out_dir.exists() == false);
        assert!(public_dir.join(site_name).exists() == false);
        fs::remove_dir("public/Test_Site").unwrap();
    }
}

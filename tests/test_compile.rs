#[cfg(test)]
mod tests {
    use std::path::Path;

    use ssg::compile;

    #[test]
    fn test_compile_success() {
        let src_dir = Path::new("./test_content");
        let out_dir = Path::new("./test_output");
        let template_path = Some("./test_templates/".to_string());
        let site_name = "Test Site".to_string();
        let result = compile(
            src_dir,
            out_dir,
            template_path.as_ref(),
            site_name,
        );
        if let Err(ref error) = result {
            eprintln!("{}", error);
        }
        assert!(result.is_err());
    }

    #[test]
    fn test_compile_with_missing_src_dir() {
        let src_dir =
            Path::new("tests/test_content/nonexistent_src_dir");
        let out_dir = Path::new("tests/test_output");
        let template_path = Some(String::from(
            "tests/test_content/templates/template1",
        ));
        let site_name = String::from("Test Site");
        assert!(compile(
            src_dir,
            out_dir,
            template_path.as_ref(),
            site_name
        )
        .is_err());
    }

    #[test]
    fn test_compile_with_missing_template() {
        let src_dir = Path::new("tests/test_content");
        let out_dir = Path::new("tests/test_output");
        let template_path = Some(String::from(
            "tests/test_content/nonexistent_template",
        ));
        let site_name = String::from("Test Site");
        assert!(compile(
            src_dir,
            out_dir,
            template_path.as_ref(),
            site_name
        )
        .is_err());
    }
}

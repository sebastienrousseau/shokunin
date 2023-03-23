#[cfg(test)]
mod tests {
    use ssg::create_template_directory;

    #[test]
    fn test_create_template_directory_with_existing_template() {
        let template_path =
            Some("tests/test_templates/template.html".to_string());
        assert!(
            create_template_directory(template_path.as_ref()).is_ok()
        );
    }

    #[test]
    fn test_create_template_directory_with_missing_template() {
        let template_path =
            Some("tests/test_data/nonexistent_template".to_string());
        let result = create_template_directory(template_path.as_ref());
        if let Err(ref error) = result {
            eprintln!("{}", error);
        }
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_template_directory_with_no_template() {
        let template_path = None;
        assert!(
            create_template_directory(template_path.as_ref()).is_ok()
        );
    }
}

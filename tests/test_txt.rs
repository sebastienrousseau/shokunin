#[cfg(test)]
mod tests {
    use ssg::modules::txt::create_txt_data;
    use std::collections::HashMap;

    #[test]
    fn test_create_txt_data_with_permalink() {
        let mut metadata = HashMap::new();
        metadata.insert(
            "permalink".to_string(),
            "https://example.com".to_string(),
        );

        let txt_data = create_txt_data(&metadata);

        assert_eq!(txt_data.permalink, "https://example.com");
    }

    #[test]
    fn test_create_txt_data_with_missing_permalink() {
        let metadata = HashMap::new(); // Empty metadata

        let txt_data = create_txt_data(&metadata);

        assert_eq!(txt_data.permalink, "");
    }
}

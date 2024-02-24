#[cfg(test)]
mod tests {
    use ssg::modules::human::create_human_data;
    use std::collections::HashMap;

    #[test]
    fn test_create_human_data_with_all_fields() {
        let mut metadata = HashMap::new();
        metadata.insert(
            "author_location".to_string(),
            "Location".to_string(),
        );
        metadata.insert(
            "author_twitter".to_string(),
            "@twitter_handle".to_string(),
        );
        metadata.insert(
            "author_website".to_string(),
            "https://example.com".to_string(),
        );
        metadata.insert("author".to_string(), "John Doe".to_string());
        metadata.insert(
            "site_components".to_string(),
            "Components".to_string(),
        );
        metadata.insert(
            "site_last_updated".to_string(),
            "2023-01-01".to_string(),
        );
        metadata.insert(
            "site_software".to_string(),
            "Software".to_string(),
        );
        metadata.insert(
            "site_standards".to_string(),
            "Standards".to_string(),
        );
        metadata
            .insert("thanks".to_string(), "Contributors".to_string());

        let human_data = create_human_data(&metadata);

        assert_eq!(human_data.author_location, "Location");
        assert_eq!(human_data.author_twitter, "@twitter_handle");
        assert_eq!(human_data.author_website, "https://example.com");
        assert_eq!(human_data.author, "John Doe");
        assert_eq!(human_data.site_components, "Components");
        assert_eq!(human_data.site_last_updated, "2023-01-01");
        assert_eq!(human_data.site_software, "Software");
        assert_eq!(human_data.site_standards, "Standards");
        assert_eq!(human_data.thanks, "Contributors");
    }

    #[test]
    fn test_create_human_data_with_missing_fields() {
        let metadata = HashMap::new(); // Empty metadata

        let human_data = create_human_data(&metadata);

        assert_eq!(human_data.author_location, "");
        assert_eq!(human_data.author_twitter, "");
        assert_eq!(human_data.author_website, "");
        assert_eq!(human_data.author, "");
        assert_eq!(human_data.site_components, "");
        assert_eq!(human_data.site_last_updated, "");
        assert_eq!(human_data.site_software, "");
        assert_eq!(human_data.site_standards, "");
        assert_eq!(human_data.thanks, "");
    }
}

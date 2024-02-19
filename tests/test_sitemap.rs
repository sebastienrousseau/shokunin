#[cfg(test)]
mod tests {
    use ssg::modules::sitemap::create_site_map_data;
    use std::collections::HashMap;

    #[test]
    fn test_create_site_map_data_with_all_fields() {
        let mut metadata = HashMap::new();
        metadata.insert("changefreq".to_string(), "daily".to_string());
        metadata.insert(
            "last_build_date".to_string(),
            "2023-01-01".to_string(),
        );
        metadata.insert(
            "permalink".to_string(),
            "https://example.com".to_string(),
        );

        let site_map_data = create_site_map_data(&metadata);

        assert_eq!(site_map_data.changefreq, "daily");
        assert_eq!(site_map_data.lastmod, "2023-01-01");
        assert_eq!(site_map_data.loc, "https://example.com");
    }

    #[test]
    fn test_create_site_map_data_with_missing_fields() {
        let metadata = HashMap::new(); // Empty metadata

        let site_map_data = create_site_map_data(&metadata);

        assert_eq!(site_map_data.changefreq, "");
        assert_eq!(site_map_data.lastmod, "");
        assert_eq!(site_map_data.loc, "");
    }
}

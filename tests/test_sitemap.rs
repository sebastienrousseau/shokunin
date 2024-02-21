#[cfg(test)]
mod tests {
    use serde_json;
    use ssg::models::data::SiteMapData;
    use ssg::modules::sitemap::create_site_map_data;
    use std::collections::HashMap;

    /// Tests the creation of SiteMapData with all expected fields provided.
    #[test]
    fn create_site_map_data_with_complete_metadata() {
        let mut metadata = HashMap::new();
        metadata.insert("changefreq".to_string(), "daily".to_string());
        metadata.insert("last_build_date".to_string(), "2024-02-20".to_string());
        metadata.insert("permalink".to_string(), "https://example.com".to_string());

        let site_map_data = create_site_map_data(&metadata);

        assert_eq!("daily", site_map_data.changefreq);
        assert_eq!("2024-02-20", site_map_data.lastmod);
        assert_eq!("https://example.com", site_map_data.loc);
    }

    /// Verifies that missing metadata fields result in default SiteMapData values.
    #[test]
    fn create_site_map_data_with_incomplete_metadata() {
        let metadata = HashMap::new(); // Empty metadata

        let site_map_data = create_site_map_data(&metadata);

        assert_eq!("", site_map_data.changefreq);
        assert_eq!("", site_map_data.lastmod);
        assert_eq!("", site_map_data.loc);
    }

    /// Checks handling of metadata when only the changefreq is provided.
    #[test]
    fn create_site_map_data_with_only_changefreq() {
        let mut metadata = HashMap::new();
        metadata.insert("changefreq".to_string(), "daily".to_string());

        let site_map_data = create_site_map_data(&metadata);

        assert_eq!("daily", site_map_data.changefreq);
        assert_eq!("", site_map_data.lastmod); // Expected default value
        assert_eq!("", site_map_data.loc); // Expected default value
    }

    /// Tests serialization and deserialization of SiteMapData for data integrity.
    #[test]
    fn serialize_and_deserialize_site_map_data() {
        let mut metadata = HashMap::new();
        metadata.insert("changefreq".to_string(), "daily".to_string());
        metadata.insert("last_build_date".to_string(), "2023-01-01".to_string());
        metadata.insert("permalink".to_string(), "https://example.com".to_string());

        let original = create_site_map_data(&metadata);
        let serialized = serde_json::to_string(&original).expect("Serialization failed");
        let deserialized: SiteMapData = serde_json::from_str(&serialized).expect("Deserialization failed");

        assert_eq!(original.changefreq, deserialized.changefreq);
        assert_eq!(original.lastmod, deserialized.lastmod);
        assert_eq!(original.loc, deserialized.loc);
    }
}

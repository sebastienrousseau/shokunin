// Copyright © 2024 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

#[cfg(test)]
mod tests {
    use staticdatagen::{
        models::data::SiteMapData,
        modules::sitemap::create_site_map_data,
    };
    use std::collections::HashMap;

    /// Helper function to create metadata with given optional values.
    fn setup_metadata(
        changefreq: Option<&str>,
        lastmod: Option<&str>,
        loc: Option<&str>,
    ) -> HashMap<String, String> {
        let mut metadata = HashMap::new();
        if let Some(cf) = changefreq {
            metadata.insert("changefreq".to_string(), cf.to_string());
        }
        if let Some(lm) = lastmod {
            metadata.insert("last_build_date".to_string(), lm.to_string());
        }
        if let Some(location) = loc {
            metadata.insert("permalink".to_string(), location.to_string());
        }
        metadata
    }

    /// Tests the creation of SiteMapData with all expected fields provided.
    #[test]
    fn create_site_map_data_with_complete_metadata() {
        let metadata = setup_metadata(
            Some("daily"),
            Some("2024-02-20"),
            Some("https://example.com"),
        );

        let site_map_data = create_site_map_data(&metadata);

        assert_eq!("daily", site_map_data.changefreq);
        assert_eq!("2024-02-20", site_map_data.lastmod);
        assert_eq!("https://example.com", site_map_data.loc);
    }

    /// Verifies that missing metadata fields result in default SiteMapData values.
    #[test]
    fn create_site_map_data_with_incomplete_metadata() {
        let metadata = setup_metadata(None, None, None); // Empty metadata

        let site_map_data = create_site_map_data(&metadata);

        assert_eq!("", site_map_data.changefreq);
        assert_eq!("", site_map_data.lastmod);
        assert_eq!("", site_map_data.loc);
    }

    /// Checks handling of metadata when only the changefreq is provided.
    #[test]
    fn create_site_map_data_with_only_changefreq() {
        let metadata = setup_metadata(Some("daily"), None, None);

        let site_map_data = create_site_map_data(&metadata);

        assert_eq!("daily", site_map_data.changefreq);
        assert_eq!("", site_map_data.lastmod); // Expected default value
        assert_eq!("", site_map_data.loc); // Expected default value
    }

    /// Tests serialization and deserialization of SiteMapData for data integrity.
    #[test]
    fn serialize_and_deserialize_site_map_data() {
        let metadata = setup_metadata(
            Some("daily"),
            Some("2023-01-01"),
            Some("https://example.com"),
        );

        let original = create_site_map_data(&metadata);
        let serialized = serde_json::to_string(&original)
            .expect("Serialization failed");
        let deserialized: SiteMapData = serde_json::from_str(&serialized)
            .expect("Deserialization failed");

        assert_eq!(original.changefreq, deserialized.changefreq);
        assert_eq!(original.lastmod, deserialized.lastmod);
        assert_eq!(original.loc, deserialized.loc);
    }
}

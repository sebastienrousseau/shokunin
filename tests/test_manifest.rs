#[cfg(test)]
mod tests {
    use ssg::modules::manifest::create_manifest_data;
    use std::collections::HashMap;

    #[test]
    fn test_create_manifest_data_with_valid_metadata() {
        let mut metadata = HashMap::new();
        metadata.insert("name".to_string(), "My Web App".to_string());
        metadata.insert("short_name".to_string(), "App".to_string());
        metadata.insert(
            "description".to_string(),
            "A cool web app".to_string(),
        );
        metadata.insert("icon".to_string(), "app-icon.svg".to_string());
        metadata
            .insert("theme-color".to_string(), "#00aabb".to_string());

        let manifest_data = create_manifest_data(&metadata);

        assert_eq!(manifest_data.name, "My Web App");
        assert_eq!(manifest_data.short_name, "App");
        assert_eq!(manifest_data.start_url, ".");
        assert_eq!(manifest_data.display, "standalone");
        assert_eq!(manifest_data.background_color, "#ffffff");
        assert_eq!(manifest_data.description, "A cool web app");
        assert_eq!(manifest_data.icons.len(), 1);
        assert_eq!(manifest_data.icons[0].src, "app-icon.svg");
        assert_eq!(manifest_data.icons[0].sizes, "512x512");
        assert_eq!(
            manifest_data.icons[0].icon_type,
            Some("image/svg+xml".to_string())
        );
        assert_eq!(
            manifest_data.icons[0].purpose,
            Some("any maskable".to_string())
        );
        assert_eq!(manifest_data.orientation, "portrait-primary");
        assert_eq!(manifest_data.scope, "/");
        assert_eq!(manifest_data.theme_color, "#00aabb");
    }

    #[test]
    fn test_create_manifest_data_with_missing_metadata() {
        let metadata = HashMap::new(); // Empty metadata

        let manifest_data = create_manifest_data(&metadata);

        assert_eq!(manifest_data.name, "");
        assert_eq!(manifest_data.short_name, "");
        assert_eq!(manifest_data.start_url, ".");
        assert_eq!(manifest_data.display, "standalone");
        assert_eq!(manifest_data.background_color, "#ffffff");
        assert_eq!(manifest_data.description, "");
        assert!(manifest_data.icons.is_empty());
        assert_eq!(manifest_data.orientation, "portrait-primary");
        assert_eq!(manifest_data.scope, "/");
        assert_eq!(manifest_data.theme_color, "");
    }
}

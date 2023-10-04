#[cfg(test)]
mod tests {
    use ssg::models::data::{FileData, PageData};
    use ssg::modules::tags::{create_tags_data, generate_tags, generate_tags_html};
    use std::collections::HashMap;

    #[test]
    fn test_generate_tags_with_tags_and_metadata() {
        let file_data = FileData {
            content: "This is a test with tags: tag1, tag2".to_string(),
            ..Default::default()
        };
        let mut metadata = HashMap::new();
        metadata.insert("tags".to_string(), "tag1, tag2".to_string());
        metadata.insert("title".to_string(), "Test Page".to_string());

        let tags_data_map = generate_tags(&file_data, &metadata);

        assert_eq!(tags_data_map.len(), 2);

        let tag1_data = tags_data_map.get("tag1").unwrap();
        assert_eq!(tag1_data.len(), 1);
        let tag1_entry = &tag1_data[0];
        assert_eq!(tag1_entry["title"], "Test Page");

        let tag2_data = tags_data_map.get("tag2").unwrap();
        assert_eq!(tag2_data.len(), 1);
        let tag2_entry = &tag2_data[0];
        assert_eq!(tag2_entry["title"], "Test Page");
    }

    #[test]
    fn test_generate_tags_with_no_tags_in_metadata() {
        let file_data = FileData {
            content: "This is a test".to_string(),
            ..Default::default()
        };
        let metadata = HashMap::new(); // No tags in metadata

        let tags_data_map = generate_tags(&file_data, &metadata);

        assert_eq!(tags_data_map.len(), 0);
    }

    #[test]
    fn test_create_tags_data_with_metadata() {
        let mut metadata = HashMap::new();
        metadata.insert("date".to_string(), "2021-09-04".to_string());
        metadata.insert("description".to_string(), "A sample description".to_string());

        let tags_data = create_tags_data(&metadata);

        assert_eq!(tags_data.dates, "2021-09-04");
        assert_eq!(tags_data.descriptions, "A sample description");
        assert_eq!(tags_data.titles, "");
        assert_eq!(tags_data.permalinks, "");
        assert_eq!(tags_data.keywords, "");
    }

    #[test]
    fn test_generate_tags_html_with_data() {
        let mut global_tags_data = HashMap::new();
        global_tags_data.insert(
            "tag1".to_string(),
            vec![
                PageData {
                    date: "2022-09-01".to_string(),
                    description: "Description 1".to_string(),
                    permalink: "/page1".to_string(),
                    title: "Page 1".to_string(),
                },
            ],
        );

        let global_tags_data = generate_tags_html(&global_tags_data);

        assert!(global_tags_data.contains("<h2 class=\"featured-tags\" id=\"h2-featured-tags\" tabindex=\"0\">Featured Tags (1)</h2>"));
        assert!(global_tags_data.contains("<h3 class=\"tag1\" id=\"h3-tag1\" tabindex=\"0\">Tag1 (1 Posts)</h3>"));
        assert!(global_tags_data.contains("<li>2022-09-01: <a href=\"/page1\">Page 1</a> - <strong>Description 1</strong></li>"));
    }

    
}

// Copyright © 2024 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

// Import necessary modules and types
#[cfg(test)]
mod tests {
    use staticdatagen::models::data::{
        CnameData, FileData, HumanData, HumansData, IconData,
        ManifestData, MetaTag, NewsData, NewsVisitOptions, RssData,
        SiteMapData, TxtData,
    };

    // Test cases for CnameData
    #[test]
    fn test_cname_data_default() {
        // Arrange
        let cname_data = CnameData::default();
        let expected_cname_data = CnameData {
            cname: String::default(),
        };
        // Act & Assert
        assert_eq!(cname_data, expected_cname_data);
    }

    #[test]
    fn test_cname_data_new() {
        // Arrange
        let cname_data = CnameData::new("example.com".to_string());
        // Act & Assert
        assert_eq!(cname_data.cname, "example.com");
    }

    // Test cases for FileData
    #[test]
    fn test_file_data_default() {
        // Arrange
        let file_data = FileData::default();
        let expected_file_data = FileData {
            cname: String::default(),
            content: String::default(),
            human: String::default(),
            json: String::default(),
            keyword: String::default(),
            name: String::default(),
            rss: String::default(),
            sitemap: String::default(),
            sitemap_news: String::default(),
            txt: String::default(),
        };
        // Act & Assert
        assert_eq!(file_data, expected_file_data);
    }

    #[test]
    fn test_file_data_new() {
        // Arrange
        let file_data = FileData::new(
            "file.txt".to_string(),
            "Content".to_string(),
        );
        // Act & Assert
        assert_eq!(file_data.name, "file.txt");
        assert_eq!(file_data.content, "Content");
    }

    // Test cases for IconData
    #[test]
    fn test_icon_data_default() {
        // Arrange
        let icon_data = IconData::default();
        let expected_icon_data = IconData {
            src: String::default(),
            sizes: String::default(),
            icon_type: None,
            purpose: None,
        };
        // Act & Assert
        assert_eq!(icon_data, expected_icon_data);
    }

    #[test]
    fn test_icon_data_new() {
        // Arrange
        let icon_data =
            IconData::new("icon.png".to_string(), "32x32".to_string());
        // Act & Assert
        assert_eq!(icon_data.src, "icon.png");
        assert_eq!(icon_data.sizes, "32x32");
    }

    // Test cases for ManifestData
    #[test]
    fn test_manifest_options_default() {
        // Arrange
        let manifest_options = ManifestData::default();
        let expected_manifest_options = ManifestData {
            background_color: String::default(),
            description: String::default(),
            display: String::default(),
            icons: Vec::new(),
            name: String::default(),
            orientation: String::default(),
            scope: String::default(),
            short_name: String::default(),
            start_url: String::default(),
            theme_color: String::default(),
        };
        // Act & Assert
        assert_eq!(manifest_options, expected_manifest_options);
    }

    #[test]
    fn test_manifest_data_new() {
        // Arrange
        let manifest_data = ManifestData::new();
        // Act & Assert
        assert_eq!(manifest_data.background_color, "");
        assert_eq!(manifest_data.description, "");
        assert_eq!(manifest_data.display, "");
        assert_eq!(manifest_data.icons, Vec::<IconData>::new());
        assert_eq!(manifest_data.name, "");
        assert_eq!(manifest_data.orientation, "");
        assert_eq!(manifest_data.scope, "");
        assert_eq!(manifest_data.short_name, "");
        assert_eq!(manifest_data.start_url, "");
        assert_eq!(manifest_data.theme_color, "");
    }

    // Test cases for SiteMapData
    #[test]
    fn test_sitemap_data_default() {
        // Arrange
        let sitemap_data = SiteMapData::default();
        let expected_sitemap_data = SiteMapData {
            loc: String::default(),
            lastmod: String::default(),
            changefreq: String::default(),
        };
        // Act & Assert
        assert_eq!(sitemap_data, expected_sitemap_data);
    }

    #[test]
    fn test_sitemap_data_new() {
        // Arrange
        let sitemap_data = SiteMapData::new(
            "example.com".to_string(),
            "2023-01-01".to_string(),
            "daily".to_string(),
        );
        // Act & Assert
        assert_eq!(sitemap_data.loc, "example.com");
        assert_eq!(sitemap_data.lastmod, "2023-01-01");
        assert_eq!(sitemap_data.changefreq, "daily");
    }

    // Test cases for TxtData
    #[test]
    fn test_txt_data_default() {
        // Arrange
        let txt_data = TxtData::default();
        let expected_txt_data = TxtData {
            permalink: String::default(),
        };
        // Act & Assert
        assert_eq!(txt_data, expected_txt_data);
    }

    #[test]
    fn test_txt_data_new() {
        // Arrange
        let txt_data = TxtData::new("example.com".to_string());
        // Act & Assert
        assert_eq!(txt_data.permalink, "example.com");
    }

    // Test cases for IconData (duplicate)
    #[test]
    fn test_icon_data() {
        // Arrange
        let icon_data =
            IconData::new("icon.png".to_string(), "32x32".to_string());
        // Act & Assert
        assert_eq!(icon_data.src, "icon.png");
        assert_eq!(icon_data.sizes, "32x32");
        assert_eq!(icon_data.icon_type, None);
        assert_eq!(icon_data.purpose, None);
    }

    // Test cases for NewsData
    #[test]
    fn test_news_data_create_default() {
        // Arrange
        let news_data = NewsData::create_default();
        // Act & Assert
        assert_eq!(news_data.news_genres, "");
        assert_eq!(news_data.news_keywords, "");
        assert_eq!(news_data.news_language, "");
        assert_eq!(news_data.news_image_loc, "");
        assert_eq!(news_data.news_loc, "");
        assert_eq!(news_data.news_publication_date, "");
        assert_eq!(news_data.news_publication_name, "");
        assert_eq!(news_data.news_title, "");
    }

    #[test]
    fn test_news_data_new() {
        // Arrange
        let news_data = NewsData::new(NewsData {
            news_genres: "News".to_string(),
            news_keywords: "Keyword".to_string(),
            news_language: "English".to_string(),
            news_image_loc: "/images/news.jpg".to_string(),
            news_loc: "/news".to_string(),
            news_publication_date: "2024-05-01".to_string(),
            news_publication_name: "Daily News".to_string(),
            news_title: "Breaking News!".to_string(),
        });
        // Act & Assert
        assert_eq!(news_data.news_genres, "News");
        assert_eq!(news_data.news_keywords, "Keyword");
        assert_eq!(news_data.news_language, "English");
        assert_eq!(news_data.news_image_loc, "/images/news.jpg");
        assert_eq!(news_data.news_loc, "/news");
        assert_eq!(news_data.news_publication_date, "2024-05-01");
        assert_eq!(news_data.news_publication_name, "Daily News");
        assert_eq!(news_data.news_title, "Breaking News!");
    }

    // Test cases for NewsVisitOptions
    #[test]
    fn test_news_visit_options_new() {
        // Arrange
        let news_visit_options = NewsVisitOptions {
            base_url: "example.com",
            news_genres: "News",
            news_keywords: "Keyword",
            news_language: "English",
            news_publication_date: "2024-05-01",
            news_publication_name: "Daily News",
            news_title: "Breaking News!",
        };
        // Act & Assert
        assert_eq!(news_visit_options.base_url, "example.com");
        assert_eq!(news_visit_options.news_genres, "News");
        assert_eq!(news_visit_options.news_keywords, "Keyword");
        assert_eq!(news_visit_options.news_language, "English");
        assert_eq!(
            news_visit_options.news_publication_date,
            "2024-05-01"
        );
        assert_eq!(
            news_visit_options.news_publication_name,
            "Daily News"
        );
        assert_eq!(news_visit_options.news_title, "Breaking News!");
    }

    // Test cases for HumanData
    #[test]
    fn test_human_data_new() {
        // Arrange
        let human_data = HumanData::new();
        // Act & Assert
        assert_eq!(human_data.author, None);
        assert_eq!(human_data.author_website, None);
        assert_eq!(human_data.author_twitter, None);
        assert_eq!(human_data.author_location, None);
        assert_eq!(human_data.thanks, None);
        assert_eq!(human_data.site_last_updated, None);
        assert_eq!(human_data.site_standards, None);
        assert_eq!(human_data.site_components, None);
        assert_eq!(human_data.site_software, None);
    }

    // Test cases for HumansData
    #[test]
    fn test_humans_data_new() {
        // Arrange
        let humans_data =
            HumansData::new("Author".to_string(), "Thanks".to_string());
        // Act & Assert
        assert_eq!(humans_data.author, "Author");
        assert_eq!(humans_data.author_website, "");
        assert_eq!(humans_data.author_twitter, "");
        assert_eq!(humans_data.author_location, "");
        assert_eq!(humans_data.thanks, "Thanks");
        assert_eq!(humans_data.site_last_updated, "");
        assert_eq!(humans_data.site_standards, "");
        assert_eq!(humans_data.site_components, "");
        assert_eq!(humans_data.site_software, "");
    }

    // Test cases for RssData
    #[test]
    fn test_rss_data_new() {
        // Arrange
        let mut rss_data = RssData::new();
        rss_data.set("atom_link", "atom_link");
        rss_data.set("author", "author");
        rss_data.set("category", "category");
        rss_data.set("copyright", "copyright");
        rss_data.set("description", "description");
        rss_data.set("docs", "docs");
        rss_data.set("generator", "generator");
        rss_data.set("image", "image");
        rss_data.set("item_guid", "item_guid");
        rss_data.set("item_description", "item_description");
        rss_data.set("item_link", "item_link");
        rss_data.set("item_pub_date", "item_pub_date");
        rss_data.set("item_title", "item_title");
        rss_data.set("language", "language");
        rss_data.set("last_build_date", "last_build_date");
        rss_data.set("link", "link");
        rss_data.set("managing_editor", "managing_editor");
        rss_data.set("pub_date", "pub_date");
        rss_data.set("title", "title");
        rss_data.set("ttl", "ttl");
        rss_data.set("webmaster", "webmaster");

        // Act & Assert
        assert_eq!(rss_data.atom_link, "atom_link");
        assert_eq!(rss_data.author, "author");
        assert_eq!(rss_data.category, "category");
        assert_eq!(rss_data.copyright, "copyright");
        assert_eq!(rss_data.description, "description");
        assert_eq!(rss_data.docs, "docs");
        assert_eq!(rss_data.generator, "generator");
        assert_eq!(rss_data.image, "image");
        assert_eq!(rss_data.item_guid, "item_guid");
        assert_eq!(rss_data.item_description, "item_description");
        assert_eq!(rss_data.item_link, "item_link");
        assert_eq!(rss_data.item_pub_date, "item_pub_date");
        assert_eq!(rss_data.item_title, "item_title");
        assert_eq!(rss_data.language, "language");
        assert_eq!(rss_data.last_build_date, "last_build_date");
        assert_eq!(rss_data.link, "link");
        assert_eq!(rss_data.managing_editor, "managing_editor");
        assert_eq!(rss_data.pub_date, "pub_date");
        assert_eq!(rss_data.title, "title");
        assert_eq!(rss_data.ttl, "ttl");
        assert_eq!(rss_data.webmaster, "webmaster");
    }

    // Test cases for MetaTag
    #[test]
    fn test_meta_tag_generate() {
        // Arrange
        let meta_tag =
            MetaTag::new("name".to_string(), "value".to_string());
        // Act & Assert
        assert_eq!(
            meta_tag.generate(),
            "<meta content=\"value\" name=\"name\">"
        );
    }

    #[test]
    fn test_meta_tag_generate_metatags() {
        // Arrange
        let meta_tags = vec![
            MetaTag::new("name1".to_string(), "value1".to_string()),
            MetaTag::new("name2".to_string(), "value2".to_string()),
        ];
        // Act & Assert
        assert_eq!(MetaTag::generate_metatags(&meta_tags), "<meta content=\"value1\" name=\"name1\"><meta content=\"value2\" name=\"name2\">");
    }
}

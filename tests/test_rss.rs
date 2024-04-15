// Copyright © 2024 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use quick_xml::{escape::escape, Writer};
    use ssg::{
        macro_write_element, models::data::RssData,
        modules::rss::generate_rss,
    };

    /// # RssData Tests
    ///
    /// These tests validate the behaviour of the `RssData` struct.
    mod rss_data_tests {
        use super::*;

        // Test the default constructor of RssData
        #[test]
        fn test_rss_options_new() {
            let options = RssData::new();

            assert_eq!(options.title, "");
            assert_eq!(options.link, "");
            assert_eq!(options.description, "");
            assert_eq!(options.generator, "");
            assert_eq!(options.language, "");
            assert_eq!(options.atom_link, "");
            assert_eq!(options.webmaster, "");
            assert_eq!(options.last_build_date, "");
            assert_eq!(options.pub_date, "");
            assert_eq!(options.item_title, "");
            assert_eq!(options.item_link, "");
            assert_eq!(options.item_guid, "");
            assert_eq!(options.item_description, "");
            assert_eq!(options.item_pub_date, "");
        }
    }

    /// # Rss Generation Tests
    ///
    /// These tests validate the generation of RSS feeds.
    mod rss_generation_tests {
        use super::*;

        // Test generating an RSS feed with default options
        #[test]
        fn test_generate_rss_with_default_options() {
            let options = RssData::new();
            let rss_result = generate_rss(&options);
            assert!(rss_result.is_ok());

            let rss_str = rss_result.unwrap();
            assert!(rss_str.contains(
                "<?xml version=\"1.0\" encoding=\"utf-8\"?>"
            ));
            assert!(rss_str.contains("<rss version=\"2.0\" xmlns:atom=\"http://www.w3.org/2005/Atom\">"));
            assert!(rss_str.contains("</rss>"));
            assert!(!rss_str.contains("<title></title>"));
            assert!(!rss_str.contains("<link></link>"));
            assert!(!rss_str.contains("<description></description>"));
        }

        // Test generating an RSS feed with some custom options
        #[test]
        fn test_generate_rss_with_custom_options() {
            let mut options = RssData::new();
            options.title = "My RSS Feed".to_string();
            options.link = "https://example.com".to_string();
            options.description =
                "A description of my RSS feed.".to_string();

            let rss_result = generate_rss(&options);
            assert!(rss_result.is_ok());

            let rss_str = rss_result.unwrap();
            assert!(rss_str.contains("<title>My RSS Feed</title>"));
            assert!(rss_str
                .contains("<link>https://example.com</link>"));
            assert!(rss_str.contains("<description>A description of my RSS feed.</description>"));
        }
    }

    /// # Rss Writer Tests
    ///
    /// These tests validate the behaviour of the RSS writer functions.
    mod rss_writer_tests {
        use super::*;

        // Test macro_write_element function
        #[test]
        fn test_macro_write_element(
        ) -> Result<(), Box<dyn std::error::Error>> {
            let mut writer = Writer::new(Cursor::new(Vec::new()));
            macro_write_element!(
                &mut writer,
                "testElement",
                escape("testValue")
            )?;
            let xml = writer.into_inner().into_inner();
            let xml_str = String::from_utf8(xml)?;
            assert_eq!(xml_str, "<testElement>testValue</testElement>");

            Ok(())
        }

        // Test generating an RSS feed
        #[test]
        fn test_generate_rss() {
            let options = RssData::new();
            let rss_str = generate_rss(&options);
            assert!(rss_str.is_ok());
        }

        // Test generating an RSS feed with empty title
        #[test]
        fn test_generate_rss_with_empty_title() {
            let mut options = RssData::new();
            options.title = "".to_string();

            let rss = generate_rss(&options);
            assert!(rss.is_ok());

            let rss_str = rss.unwrap();

            // check that the title is not in the feed
            assert!(!rss_str.contains("<title></title>"));
        }
    }

    /// # Rss Element Writer Tests
    ///
    /// These tests validate the behavior of individual RSS element writer functions.
    mod rss_element_writer_tests {
        use super::*;

        // Test generating an RSS feed with invalid URL
        #[test]
        fn test_generate_rss_with_invalid_url() {
            let mut options = RssData::new();
            options.link = "invalid-url".to_string();

            let rss_result = generate_rss(&options);
            assert!(rss_result.is_ok());

            let rss_str = rss_result.unwrap();

            // Check that the generated RSS feed contains the invalid URL value
            assert!(rss_str.contains("<link>invalid-url</link>"));
        }
    }
}

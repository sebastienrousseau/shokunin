#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use quick_xml::events::BytesEnd;
    use quick_xml::events::BytesStart;
    use quick_xml::events::Event;
    use quick_xml::Writer;
    use ssg::rss::generate_rss;
    use ssg::rss::RssOptions;
    use std::borrow::Cow;

    // Test the default constructor of RssOptions
    #[test]
    fn test_rss_options_new() {
        let options = RssOptions::new();

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

    // Test generating an RSS feed with default options
    #[test]
    fn test_generate_rss_with_default_options() {
        let options = RssOptions::new();
        let rss = generate_rss(&options);

        assert!(rss.is_ok());

        let rss_str = rss.unwrap();

        // check for some basic elements
        assert!(rss_str
            .contains("<?xml version=\"1.0\" encoding=\"utf-8\"?>"));
        assert!(rss_str.contains("<rss version=\"2.0\" xmlns:atom=\"http://www.w3.org/2005/Atom\">"));
        assert!(rss_str.contains("</rss>"));

        // check that the empty fields are not in the feed
        assert!(!rss_str.contains("<title></title>"));
        assert!(!rss_str.contains("<link></link>"));
        assert!(!rss_str.contains("<description></description>"));
    }

    // Test generating an RSS feed with some custom options
    #[test]
    fn test_generate_rss_with_custom_options() {
        let mut options = RssOptions::new();
        options.title = "My RSS Feed".to_string();
        options.link = "https://example.com".to_string();
        options.description =
            "A description of my RSS feed.".to_string();

        let rss = generate_rss(&options);

        assert!(rss.is_ok());

        let rss_str = rss.unwrap();

        // check for the custom elements
        assert!(rss_str.contains("<title>My RSS Feed</title>"));
        assert!(rss_str.contains("<link>https://example.com</link>"));
        assert!(rss_str.contains(
            "<description>A description of my RSS feed.</description>"
        ));
    }
    #[test]
    fn test_macro_write_element(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut writer = Writer::new(Cursor::new(Vec::new()));
        let result = ssg::macro_write_element!(
            &mut writer,
            "testElement",
            "testValue"
        )?;
        let xml = writer.into_inner().into_inner();
        let xml_str = String::from_utf8(xml)?;
        assert_eq!(xml_str, "<testElement>testValue</testElement>");
        assert_eq!(result, ());

        Ok(())
    }

    #[test]
    fn test_generate_rss() {
        let options = RssOptions::new();
        let rss_str = generate_rss(&options);
        assert!(rss_str.is_ok());
    }

    #[test]
    fn test_generate_rss_10000_items() {
        let mut options = RssOptions::new();
        for i in 0..10000 {
            options.item_title = format!("Item {}", i);
            options.item_link =
                format!("https://example.com/item{}", i);
            options.item_guid = format!("item{}", i);
            options.item_description = format!("This is item {}.", i);
            options.item_pub_date =
                "Wed, 20 May 2020 07:00:00 GMT".to_string();

            let rss = generate_rss(&options);

            assert!(rss.is_ok());

            let rss_str = rss.unwrap();

            assert!(
                rss_str.contains(&format!("<title>Item {}</title>", i))
            );
            assert!(rss_str.contains(&format!(
                "<link>https://example.com/item{}</link>",
                i
            )));
            assert!(
                rss_str.contains(&format!("<guid>item{}</guid>", i))
            );
            assert!(rss_str.contains(&format!(
                "<description>This is item {}.</description>",
                i
            )));
            assert!(rss_str.contains(
                "<pubDate>Wed, 20 May 2020 07:00:00 GMT</pubDate>"
            ));
        }
    }

    #[test]
    fn test_generate_rss_with_empty_title() {
        let mut options = RssOptions::new();
        options.title = "".to_string();

        let rss = generate_rss(&options);

        assert!(rss.is_ok());

        let rss_str = rss.unwrap();

        // check that the title is not in the feed
        assert!(!rss_str.contains("<title></title>"));
    }

    #[test]
    fn test_generate_rss_with_invalid_url() {
        let mut options = RssOptions::new();
        options.link = "invalid-url".to_string();

        let rss = generate_rss(&options);

        // Currently, generate_rss doesn't validate URLs,
        // so it will still return an Ok result even if the URL is invalid.
        assert!(rss.is_ok());
    }
}

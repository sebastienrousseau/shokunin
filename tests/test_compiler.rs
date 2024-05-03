// Copyright © 2024 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

#[cfg(test)]
mod tests {
    use std::error::Error;

    use ssg::{
        models::data::RssData,
        modules::{
            frontmatter::extract, html::generate_html,
            rss::generate_rss,
        },
    };

    #[test]
    fn test_extract_metadata() {
        let content = "---
title: My Title
description: My Description
---
";
        let metadata = extract(content);
        assert_eq!(metadata["title"], "My Title");
        assert_eq!(metadata["description"], "My Description");
    }

    #[test]
    fn test_extract_metadata_with_no_metadata() {
        let content = "This is some content with no frontmatter";
        let metadata = extract(content);
        assert!(metadata.is_empty());
    }

    #[test]
    fn test_generate_html() {
        let content = "## Hello, world!\n\nThis is a test.";
        let title = "My Page";
        let description = "This is a test page";
        let html_result =
            generate_html(content, title, description, None);

        match html_result {
            Ok(html) => {
                assert_eq!(
                html,
                "<h1 id=\"h1-my\" tabindex=\"0\" aria-label=\"My Heading\" itemprop=\"headline\" class=\"my\">My Page</h1><p>This is a test page</p><h2 id=\"h2-hello\" tabindex=\"0\" aria-label=\"Hello Heading\" itemprop=\"name\" class=\"hello\">Hello, world!</h2>\n<p>This is a test.</p>\n"
            );
            }
            Err(err) => {
                panic!("HTML generation failed with error: {:?}", err);
            }
        }
    }

    #[test]
    fn test_generate_rss() -> Result<(), Box<dyn Error>> {
        let options = RssData {
            author: "Me".to_string(),
            category: "Technology".to_string(),
            copyright: "© 2023-2024 My Company".to_string(),
            description: "Latest technology news".to_string(),
            docs: "https://example.com/rss/docs".to_string(),
            generator: "My RSS Generator".to_string(),
            image: "None".to_string(),
            language: "en".to_string(),
            last_build_date: "2023-06-29T12:00:00Z".to_string(),
            link: "https://example.com".to_string(),
            managing_editor: "editor@example.com".to_string(),
            pub_date: "2023-06-29T12:00:00Z".to_string(),
            title: "My RSS Feed".to_string(),
            ttl: "60".to_string(),
            webmaster: "webmaster@example.com".to_string(),
            atom_link: "https://example.com/rss/feed".to_string(),
            item_description: "Item description".to_string(),
            item_guid: "item-guid".to_string(),
            item_link: "https://example.com/item".to_string(),
            item_pub_date: "2023-06-29T12:00:00Z".to_string(),
            item_title: "Item title".to_string(),
        };

        const EXPECTED_RESULT: &str = "<?xml version=\"1.0\" encoding=\"utf-8\"?><rss version=\"2.0\" xmlns:atom=\"http://www.w3.org/2005/Atom\"><channel><title>My RSS Feed</title><link>https://example.com</link><description>Latest technology news</description><language>en</language><pubDate>2023-06-29T12:00:00Z</pubDate><lastBuildDate>2023-06-29T12:00:00Z</lastBuildDate><docs>https://example.com/rss/docs</docs><generator>My RSS Generator</generator><managingEditor>editor@example.com</managingEditor><webMaster>webmaster@example.com</webMaster><category>Technology</category><ttl>60</ttl><image><url>None</url><title>My RSS Feed</title><link>https://example.com</link></image><atom:link href=\"https://example.com/rss/feed\" rel=\"self\" type=\"application/rss+xml\"/><item><author>Me</author><description>Item description</description><guid>item-guid</guid><link>https://example.com/item</link><pubDate>2023-06-29T12:00:00Z</pubDate><title>Item title</title></item></channel></rss>";

        let result: Result<String, Box<dyn Error>> =
            generate_rss(&options);
        assert_eq!(result?, EXPECTED_RESULT);

        Ok(())
    }
}

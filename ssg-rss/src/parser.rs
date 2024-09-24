// Copyright © 2024 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

use crate::data::{RssData, RssItem};
use crate::error::{Result, RssError};
use crate::version::RssVersion;
use quick_xml::events::Event;
use quick_xml::name::QName;
use quick_xml::Reader;
use std::str::FromStr;

/// Parses an RSS feed from XML content.
///
/// This function takes XML content as input and parses it into an `RssData` struct.
/// It supports parsing RSS versions 0.90, 0.91, 0.92, 1.0, and 2.0.
///
/// # Arguments
///
/// * `content` - A string slice containing the XML content of the RSS feed.
///
/// # Returns
///
/// * `Ok(RssData)` - The parsed RSS data if successful.
/// * `Err(RssError)` - An error if parsing fails.
///
pub fn parse_rss(content: &str) -> Result<RssData> {
    let mut reader = Reader::from_str(content);

    let mut rss_data = RssData::new();
    let mut buf: Vec<u8> = Vec::new();
    let mut in_channel = false;
    let mut in_item = false;
    let mut current_item = RssItem::new();
    let mut current_element = String::new();

    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e)) => {
                let name = e.name();
                if name == QName(b"rss") {
                    if let Some(version_attr) =
                        e.attributes().find(|attr| {
                            attr.as_ref().unwrap().key
                                == QName(b"version")
                        })
                    {
                        // Store the value in a variable with a longer lifetime
                        let version_value =
                            version_attr.unwrap().value.clone();

                        // Handling UTF-8 error when parsing the version attribute
                        let version_str =
                            std::str::from_utf8(&version_value)
                                .map_err(|_e| RssError::InvalidInput)?; // Corrected the error handling

                        // Handling version string parsing errors
                        rss_data.version =
                            RssVersion::from_str(version_str)
                                .map_err(|_e| RssError::InvalidInput)?; // Using InvalidInput to handle version parse errors
                    }
                } else if name == QName(b"channel") {
                    in_channel = true;
                } else if name == QName(b"item") {
                    in_item = true;
                    current_item = RssItem::new();
                }
                current_element =
                    String::from_utf8_lossy(name.as_ref()).into_owned();
            }
            Ok(Event::End(ref e)) => {
                let name = e.name();
                if name == QName(b"channel") {
                    in_channel = false;
                } else if name == QName(b"item") {
                    in_item = false;
                    rss_data.add_item(current_item.clone());
                }
                current_element.clear();
            }
            Ok(Event::Text(e)) => {
                let text = e
                    .unescape()
                    .map_err(RssError::XmlParseError)?
                    .into_owned();
                if in_channel && !in_item {
                    parse_channel_element(
                        &mut rss_data,
                        &current_element,
                        &text,
                    )?;
                } else if in_item {
                    parse_item_element(
                        &mut current_item,
                        &current_element,
                        &text,
                    )?;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(RssError::XmlParseError(e)),
            _ => {}
        }
        buf.clear();
    }

    Ok(rss_data)
}

fn parse_channel_element(
    rss_data: &mut RssData,
    element: &str,
    text: &str,
) -> Result<()> {
    match element {
        "title" => rss_data.title = text.to_string(),
        "link" => rss_data.link = text.to_string(),
        "description" => rss_data.description = text.to_string(),
        "language" => rss_data.language = text.to_string(),
        "copyright" => rss_data.copyright = text.to_string(),
        "managingEditor" => rss_data.managing_editor = text.to_string(),
        "webMaster" => rss_data.webmaster = text.to_string(),
        "pubDate" => rss_data.pub_date = text.to_string(),
        "lastBuildDate" => rss_data.last_build_date = text.to_string(),
        "category" => rss_data.category = text.to_string(),
        "generator" => rss_data.generator = text.to_string(),
        "docs" => rss_data.docs = text.to_string(),
        "ttl" => rss_data.ttl = text.to_string(),
        _ => {
            // return Err(RssError::UnknownElement(format!(
            //     "Unknown channel element: {}",
            //     element
            // )))
        }
    }
    Ok(())
}

fn parse_item_element(
    item: &mut RssItem,
    element: &str,
    text: &str,
) -> Result<()> {
    match element {
        "title" => item.title = text.to_string(),
        "link" => item.link = text.to_string(),
        "description" => item.description = text.to_string(),
        "author" => item.author = text.to_string(),
        "guid" => item.guid = text.to_string(),
        "pubDate" => item.pub_date = text.to_string(),
        _ => {
            // return Err(RssError::UnknownElement(format!(
            //     "Unknown item element: {}",
            //     element
            // )))
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_rss_2_0() {
        let xml_content = r#"
        <?xml version="1.0" encoding="UTF-8"?>
        <rss version="2.0">
          <channel>
            <title>My RSS Feed</title>
            <link>https://example.com</link>
            <description>A sample RSS feed</description>
            <language>en-us</language>
            <item>
              <title>First Post</title>
              <link>https://example.com/first-post</link>
              <description>This is my first post</description>
              <guid>https://example.com/first-post</guid>
              <pubDate>Mon, 01 Jan 2023 00:00:00 GMT</pubDate>
            </item>
          </channel>
        </rss>
        "#;

        let rss_data = parse_rss(xml_content).unwrap();
        assert_eq!(rss_data.version, RssVersion::RSS2_0);
        assert_eq!(rss_data.title, "My RSS Feed");
        assert_eq!(rss_data.link, "https://example.com");
        assert_eq!(rss_data.description, "A sample RSS feed");
        assert_eq!(rss_data.language, "en-us");
        assert_eq!(rss_data.items.len(), 1);

        let item = &rss_data.items[0];
        assert_eq!(item.title, "First Post");
        assert_eq!(item.link, "https://example.com/first-post");
        assert_eq!(item.description, "This is my first post");
        assert_eq!(item.guid, "https://example.com/first-post");
        assert_eq!(item.pub_date, "Mon, 01 Jan 2023 00:00:00 GMT");
    }

    #[test]
    fn test_parse_rss_1_0() {
        let xml_content = r#"
        <?xml version="1.0" encoding="UTF-8"?>
        <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#" xmlns="http://purl.org/rss/1.0/">
          <channel rdf:about="https://example.com/rss">
            <title>My RSS 1.0 Feed</title>
            <link>https://example.com</link>
            <description>A sample RSS 1.0 feed</description>
            <items>
              <rdf:Seq>
                <rdf:li rdf:resource="https://example.com/first-post" />
              </rdf:Seq>
            </items>
          </channel>
          <item rdf:about="https://example.com/first-post">
            <title>First Post</title>
            <link>https://example.com/first-post</link>
            <description>This is my first post in RSS 1.0</description>
          </item>
        </rdf:RDF>
        "#;

        let rss_data = parse_rss(xml_content).unwrap();
        assert_eq!(rss_data.title, "My RSS 1.0 Feed");
        assert_eq!(rss_data.link, "https://example.com");
        assert_eq!(rss_data.description, "A sample RSS 1.0 feed");
        assert_eq!(rss_data.items.len(), 1);

        let item = &rss_data.items[0];
        assert_eq!(item.title, "First Post");
        assert_eq!(item.link, "https://example.com/first-post");
        assert_eq!(
            item.description,
            "This is my first post in RSS 1.0"
        );
    }

    #[test]
    fn test_parse_invalid_xml() {
        let invalid_xml = r#"
        <?xml version="1.0" encoding="UTF-8"?>
        <rss version="2.0">
          <channel>
            <title>Invalid Feed</title>
            <link>https://example.com</link>
            <description>This XML is invalid
          </channel>
        </rss>
        "#;

        assert!(parse_rss(invalid_xml).is_err());
    }

    #[test]
    fn test_parse_unknown_version() {
        let unknown_version_xml = r#"
        <?xml version="1.0" encoding="UTF-8"?>
        <rss version="3.0">
          <channel>
            <title>Unknown Version Feed</title>
            <link>https://example.com</link>
            <description>This feed has an unknown version</description>
          </channel>
        </rss>
        "#;

        assert!(parse_rss(unknown_version_xml).is_err());
    }
}

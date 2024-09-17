// Copyright © 2024 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

use crate::error::{Result, RssError};
use crate::models::data::RssData;
use quick_xml::events::BytesText;
use quick_xml::{
    events::{BytesDecl, BytesEnd, BytesStart, Event},
    Writer,
};
use std::io::Cursor;

fn write_element<W: std::io::Write>(
    writer: &mut Writer<W>,
    name: &str,
    content: &str,
) -> Result<()> {
    writer.write_event(Event::Start(BytesStart::new(name)))?;
    writer.write_event(Event::Text(BytesText::new(content)))?;
    writer.write_event(Event::End(BytesEnd::new(name)))?;
    Ok(())
}

/// Generates an RSS feed from the given `RssData` struct.
///
/// This function creates a complete RSS feed in XML format based on the data contained in the provided `RssData`.
/// It generates the feed by creating a series of XML elements corresponding to the fields of the `RssData`,
/// and writing them to a `Writer` object.
///
/// # Arguments
///
/// * `options` - A reference to a `RssData` struct containing the RSS feed data.
///
/// # Returns
///
/// * `Ok(String)` - The generated RSS feed as a string if successful.
/// * `Err(RssError)` - An error if RSS generation fails.
///
pub fn generate_rss(options: &RssData) -> Result<String> {
    let mut writer = Writer::new(Cursor::new(Vec::new()));

    // Write the XML declaration.
    writer.write_event(Event::Decl(BytesDecl::new(
        "1.0",
        Some("utf-8"),
        None,
    )))?;

    // Start the `rss` element.
    let mut rss_start = BytesStart::new("rss");
    rss_start.push_attribute(("version", "2.0"));
    rss_start
        .push_attribute(("xmlns:atom", "http://www.w3.org/2005/Atom"));
    writer.write_event(Event::Start(rss_start))?;

    // Start the `channel` element.
    writer.write_event(Event::Start(BytesStart::new("channel")))?;

    // Write the elements.
    write_elements(&mut writer, options)?;

    // Write the `image` element.
    write_image_element(&mut writer, options)?;

    // Write the `atom:link` element.
    write_atom_link_element(&mut writer, options)?;

    // Write the `item` element.
    write_item_element(&mut writer, options)?;

    // End the `channel` element.
    writer.write_event(Event::End(BytesEnd::new("channel")))?;

    // End the `rss` element.
    writer.write_event(Event::End(BytesEnd::new("rss")))?;

    let xml = writer.into_inner().into_inner();
    String::from_utf8(xml).map_err(RssError::from)
}

/// Write the specified elements to the writer.
fn write_elements<W: std::io::Write>(
    writer: &mut Writer<W>,
    options: &RssData,
) -> Result<()> {
    write_element(writer, "title", &options.title)?;
    write_element(writer, "link", &options.link)?;
    write_element(writer, "copyright", &options.copyright)?;
    write_element(writer, "description", &options.description)?;
    write_element(writer, "language", &options.language)?;
    write_element(writer, "pubDate", &options.pub_date)?;
    write_element(writer, "lastBuildDate", &options.last_build_date)?;
    write_element(writer, "docs", &options.docs)?;
    write_element(writer, "generator", &options.generator)?;
    write_element(writer, "managingEditor", &options.managing_editor)?;
    write_element(writer, "webMaster", &options.webmaster)?;
    println!("webMaster: {}", options.webmaster);
    write_element(writer, "category", &options.category)?;
    write_element(writer, "ttl", &options.ttl)?;
    Ok(())
}

fn write_image_element<W: std::io::Write>(
    writer: &mut Writer<W>,
    options: &RssData,
) -> Result<()> {
    writer.write_event(Event::Start(BytesStart::new("image")))?;
    write_element(writer, "url", &options.image)?;
    write_element(writer, "title", &options.title)?;
    write_element(writer, "link", &options.link)?;
    writer.write_event(Event::End(BytesEnd::new("image")))?;
    Ok(())
}

fn write_item_element<W: std::io::Write>(
    writer: &mut Writer<W>,
    options: &RssData,
) -> Result<()> {
    writer.write_event(Event::Start(BytesStart::new("item")))?;
    write_element(writer, "author", &options.author)?;
    write_element(writer, "description", &options.item_description)?;
    write_element(writer, "guid", &options.item_guid)?;
    write_element(writer, "link", &options.item_link)?;
    write_element(writer, "pubDate", &options.item_pub_date)?;
    write_element(writer, "title", &options.item_title)?;
    writer.write_event(Event::End(BytesEnd::new("item")))?;
    Ok(())
}

/// Write the Atom link element to the writer.
fn write_atom_link_element<W: std::io::Write>(
    writer: &mut Writer<W>,
    options: &RssData,
) -> Result<()> {
    let mut atom_link_start = BytesStart::new("atom:link");
    atom_link_start
        .push_attribute(("href", options.atom_link.as_str()));
    atom_link_start.push_attribute(("rel", "self"));
    atom_link_start.push_attribute(("type", "application/rss+xml"));
    writer.write_event(Event::Empty(atom_link_start))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_rss() {
        let rss_data = RssData {
            title: "Test RSS Feed".to_string(),
            link: "https://example.com".to_string(),
            copyright: "Copyright © 2023 Example Inc.".to_string(),
            description: "A test RSS feed".to_string(),
            language: "en-US".to_string(),
            pub_date: "Mon, 01 Jan 2023 00:00:00 GMT".to_string(),
            last_build_date: "Mon, 01 Jan 2023 00:00:00 GMT"
                .to_string(),
            docs: "https://example.com/rss/docs".to_string(),
            generator: "ssg-rss".to_string(),
            managing_editor: "editor@example.com".to_string(),
            webmaster: "webmaster@example.com".to_string(),
            category: "Technology".to_string(),
            ttl: "60".to_string(),
            image: "https://example.com/image.png".to_string(),
            atom_link: "https://example.com/feed.xml".to_string(),
            author: "John Doe".to_string(),
            item_description: "A test item".to_string(),
            item_guid: "https://example.com/item1".to_string(),
            item_link: "https://example.com/item1".to_string(),
            item_pub_date: "Mon, 01 Jan 2023 00:00:00 GMT".to_string(),
            item_title: "Test Item".to_string(),
        };

        let result = generate_rss(&rss_data);
        assert!(result.is_ok());

        let rss_feed = result.unwrap();
        assert!(rss_feed.contains("<rss version=\"2.0\" xmlns:atom=\"http://www.w3.org/2005/Atom\">"));
        assert!(rss_feed.contains("<title>Test RSS Feed</title>"));
        assert!(rss_feed.contains("<link>https://example.com</link>"));
        assert!(rss_feed
            .contains("<description>A test RSS feed</description>"));
        assert!(rss_feed.contains("<item>"));
        assert!(rss_feed.contains("<title>Test Item</title>"));
    }
}

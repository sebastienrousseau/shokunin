// Copyright © 2024 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

use crate::data::RssData;
use crate::error::{Result, RssError};
use quick_xml::events::{
    BytesDecl, BytesEnd, BytesStart, BytesText, Event,
};
use quick_xml::Writer;
use std::io::Cursor;

/// Writes an XML element with the given name and content.
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
/// # Example
///
/// ```
/// use ssg_rss::{RssData, generate_rss};
///
/// let rss_data = RssData::new()
///     .title("My Blog")
///     .link("https://myblog.com")
///     .description("A blog about Rust programming");
///
/// match generate_rss(&rss_data) {
///     Ok(rss_feed) => println!("{}", rss_feed),
///     Err(e) => eprintln!("Error generating RSS: {}", e),
/// }
/// ```
pub fn generate_rss(options: &RssData) -> Result<String> {
    let mut writer = Writer::new(Cursor::new(Vec::new()));

    // Write the XML declaration
    writer.write_event(Event::Decl(BytesDecl::new(
        "1.0",
        Some("utf-8"),
        None,
    )))?;

    // Start the `rss` element
    let mut rss_start = BytesStart::new("rss");
    rss_start.push_attribute(("version", "2.0"));
    rss_start
        .push_attribute(("xmlns:atom", "http://www.w3.org/2005/Atom"));
    writer.write_event(Event::Start(rss_start))?;

    // Start the `channel` element
    writer.write_event(Event::Start(BytesStart::new("channel")))?;

    // Write the elements
    write_elements(&mut writer, options)?;

    // Write the `image` element
    write_image_element(&mut writer, options)?;

    // Write the `atom:link` element
    write_atom_link_element(&mut writer, options)?;

    // Write the `item` element
    write_item_element(&mut writer, options)?;

    // End the `channel` element
    writer.write_event(Event::End(BytesEnd::new("channel")))?;

    // End the `rss` element
    writer.write_event(Event::End(BytesEnd::new("rss")))?;

    let xml = writer.into_inner().into_inner();
    String::from_utf8(xml).map_err(RssError::from)
}

/// Writes the specified elements to the writer.
fn write_elements<W: std::io::Write>(
    writer: &mut Writer<W>,
    options: &RssData,
) -> Result<()> {
    let elements = [
        ("title", &options.title),
        ("link", &options.link),
        ("description", &options.description),
        ("language", &options.language),
        ("pubDate", &options.pub_date),
        ("lastBuildDate", &options.last_build_date),
        ("docs", &options.docs),
        ("generator", &options.generator),
        ("managingEditor", &options.managing_editor),
        ("webMaster", &options.webmaster),
        ("category", &options.category),
        ("ttl", &options.ttl),
    ];

    for (name, content) in elements.iter() {
        if !content.is_empty() {
            write_element(writer, name, content)?;
        }
    }

    Ok(())
}

/// Writes the image element to the writer.
fn write_image_element<W: std::io::Write>(
    writer: &mut Writer<W>,
    options: &RssData,
) -> Result<()> {
    if !options.image.is_empty() {
        writer.write_event(Event::Start(BytesStart::new("image")))?;
        write_element(writer, "url", &options.image)?;
        write_element(writer, "title", &options.title)?;
        write_element(writer, "link", &options.link)?;
        writer.write_event(Event::End(BytesEnd::new("image")))?;
    }
    Ok(())
}

/// Writes the item element to the writer.
fn write_item_element<W: std::io::Write>(
    writer: &mut Writer<W>,
    options: &RssData,
) -> Result<()> {
    writer.write_event(Event::Start(BytesStart::new("item")))?;

    let item_elements = [
        ("title", &options.item_title),
        ("link", &options.item_link),
        ("description", &options.item_description),
        ("author", &options.author),
        ("guid", &options.item_guid),
        ("pubDate", &options.item_pub_date),
    ];

    for (name, content) in item_elements.iter() {
        if !content.is_empty() {
            write_element(writer, name, content)?;
        }
    }

    writer.write_event(Event::End(BytesEnd::new("item")))?;
    Ok(())
}

/// Writes the Atom link element to the writer.
fn write_atom_link_element<W: std::io::Write>(
    writer: &mut Writer<W>,
    options: &RssData,
) -> Result<()> {
    if !options.atom_link.is_empty() {
        let mut atom_link_start = BytesStart::new("atom:link");
        atom_link_start
            .push_attribute(("href", options.atom_link.as_str()));
        atom_link_start.push_attribute(("rel", "self"));
        atom_link_start.push_attribute(("type", "application/rss+xml"));
        writer.write_event(Event::Empty(atom_link_start))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_rss() {
        let rss_data = RssData::new()
            .title("Test RSS Feed")
            .link("https://example.com")
            .description("A test RSS feed")
            .language("en-US")
            .pub_date("Mon, 01 Jan 2023 00:00:00 GMT")
            .last_build_date("Mon, 01 Jan 2023 00:00:00 GMT")
            .docs("https://example.com/rss/docs")
            .generator("ssg-rss")
            .managing_editor("editor@example.com")
            .webmaster("webmaster@example.com")
            .category("Technology")
            .ttl("60")
            .image("https://example.com/image.png")
            .atom_link("https://example.com/feed.xml")
            .author("John Doe")
            .item_description("A test item")
            .item_guid("https://example.com/item1")
            .item_link("https://example.com/item1")
            .item_pub_date("Mon, 01 Jan 2023 00:00:00 GMT")
            .item_title("Test Item");

        let result = generate_rss(&rss_data);
        assert!(result.is_ok());

        let rss_feed = result.unwrap();
        assert!(rss_feed.contains(r#"<rss version="2.0" xmlns:atom="http://www.w3.org/2005/Atom">"#));
        assert!(rss_feed.contains("<title>Test RSS Feed</title>"));
        assert!(rss_feed.contains("<link>https://example.com</link>"));
        assert!(rss_feed
            .contains("<description>A test RSS feed</description>"));
        assert!(rss_feed.contains("<language>en-US</language>"));
        assert!(rss_feed.contains("<item>"));
        assert!(rss_feed.contains("<title>Test Item</title>"));
        assert!(rss_feed.contains(r#"<atom:link href="https://example.com/feed.xml" rel="self" type="application/rss+xml"/>"#));
    }

    #[test]
    fn test_generate_rss_missing_fields() {
        let rss_data = RssData::new()
            .title("Test RSS Feed")
            .link("https://example.com");

        let result = generate_rss(&rss_data);
        assert!(result.is_ok());

        let rss_feed = result.unwrap();
        assert!(rss_feed.contains("<title>Test RSS Feed</title>"));
        assert!(rss_feed.contains("<link>https://example.com</link>"));
        assert!(!rss_feed.contains("<description>"));
        assert!(!rss_feed.contains("<language>"));
    }
}

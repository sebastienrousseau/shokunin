// Copyright © 2024 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! RSS Feed Generator module
//!
//! This module provides functionality to generate RSS feeds from structured data.
//! It uses the `quick-xml` crate for efficient XML writing and handles
//! different versions of RSS, including 0.90, 0.91, 0.92, 1.0, and 2.0.

use crate::data::{RssData, RssItem};
use crate::error::{Result, RssError};
use crate::version::RssVersion;
use quick_xml::events::{
    BytesDecl, BytesEnd, BytesStart, BytesText, Event,
};
use quick_xml::Writer;
use std::io::Cursor;

const XML_VERSION: &str = "1.0";
const XML_ENCODING: &str = "utf-8";

/// Sanitizes the content by removing invalid XML characters.
///
/// # Arguments
///
/// * `content` - A string slice containing the content to be sanitized.
///
/// # Returns
///
/// A `String` with invalid XML characters removed.
fn sanitize_content(content: &str) -> String {
    content
        .chars()
        .filter(|&c| {
            c != '\u{0000}' && !c.is_control()
                || c == '\t'
                || c == '\n'
                || c == '\r'
        })
        .collect()
}

/// Writes an XML element with the given name and content.
///
/// # Arguments
///
/// * `writer` - A mutable reference to the XML writer.
/// * `name` - The name of the XML element.
/// * `content` - The content of the XML element.
///
/// # Returns
///
/// A `Result` indicating success or failure of the write operation.
fn write_element<W: std::io::Write>(
    writer: &mut Writer<W>,
    name: &str,
    content: &str,
) -> Result<()> {
    let sanitized_content = sanitize_content(content);
    writer.write_event(Event::Start(BytesStart::new(name)))?;
    writer
        .write_event(Event::Text(BytesText::new(&sanitized_content)))?;
    writer.write_event(Event::End(BytesEnd::new(name)))?;
    Ok(())
}

/// Helper function to validate required fields.
///
/// This function checks if the given field is empty and returns an error if it is.
///
/// # Arguments
///
/// * `field` - The field value to check.
/// * `field_name` - The name of the field (for error reporting).
fn validate_field(field: &str, field_name: &str) -> Result<()> {
    if field.is_empty() {
        return Err(RssError::MissingField(field_name.to_string()));
    }
    Ok(())
}

/// Generates an RSS feed from the given `RssData` struct.
///
/// This function creates a complete RSS feed in XML format based on the data contained in the provided `RssData`.
/// It generates the feed according to the RSS version set in the `RssData`.
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
/// use ssg_rss::{RssData, generate_rss, RssVersion};
///
/// let rss_data = RssData::new(None)
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
    // Validate required fields
    validate_field(&options.title, "title")?;
    validate_field(&options.link, "link")?;
    validate_field(&options.description, "description")?;

    let mut writer = Writer::new(Cursor::new(Vec::new()));

    write_xml_declaration(&mut writer)?;

    match options.version {
        RssVersion::RSS0_90 => {
            write_rss_channel_0_90(&mut writer, options)?
        }
        RssVersion::RSS0_91 => {
            write_rss_channel_0_91(&mut writer, options)?
        }
        RssVersion::RSS0_92 => {
            write_rss_channel_0_92(&mut writer, options)?
        }
        RssVersion::RSS1_0 => {
            write_rss_channel_1_0(&mut writer, options)?
        }
        RssVersion::RSS2_0 => {
            write_rss_channel_2_0(&mut writer, options)?
        }
    }

    let xml = writer.into_inner().into_inner();
    String::from_utf8(xml).map_err(RssError::from)
}

/// Writes the XML declaration to the writer.
fn write_xml_declaration<W: std::io::Write>(
    writer: &mut Writer<W>,
) -> Result<()> {
    Ok(writer.write_event(Event::Decl(BytesDecl::new(
        XML_VERSION,
        Some(XML_ENCODING),
        None,
    )))?)
}

/// Writes the RSS 0.90 channel element and its contents.
///
/// # Parameters
///
/// * `writer`: A mutable reference to a `quick_xml::Writer` instance.
/// * `options`: A reference to an `RssData` instance, containing the RSS feed options.
///
/// # Returns
///
/// * `Result<()>`: Returns `Ok(())` if the RSS 0.90 channel elements are successfully written.
fn write_rss_channel_0_90<W: std::io::Write>(
    writer: &mut Writer<W>,
    options: &RssData,
) -> Result<()> {
    let mut rss_start = BytesStart::new("rss");
    rss_start.push_attribute(("version", "0.90"));
    writer.write_event(Event::Start(rss_start))?;

    writer.write_event(Event::Start(BytesStart::new("channel")))?;

    write_channel_elements(writer, options)?;
    write_items(writer, options)?;

    writer.write_event(Event::End(BytesEnd::new("channel")))?;
    writer.write_event(Event::End(BytesEnd::new("rss")))?;

    Ok(())
}

/// Writes the RSS 0.91 channel element and its contents.
///
/// # Parameters
///
/// * `writer`: A mutable reference to a `quick_xml::Writer` instance.
/// * `options`: A reference to an `RssData` instance, containing the RSS feed options.
///
/// # Returns
///
/// * `Result<()>`: Returns `Ok(())` if the RSS 0.91 channel elements are successfully written.
fn write_rss_channel_0_91<W: std::io::Write>(
    writer: &mut Writer<W>,
    options: &RssData,
) -> Result<()> {
    let mut rss_start = BytesStart::new("rss");
    rss_start.push_attribute(("version", "0.91"));
    writer.write_event(Event::Start(rss_start))?;

    writer.write_event(Event::Start(BytesStart::new("channel")))?;

    write_channel_elements(writer, options)?;
    write_items(writer, options)?;

    writer.write_event(Event::End(BytesEnd::new("channel")))?;
    writer.write_event(Event::End(BytesEnd::new("rss")))?;

    Ok(())
}

/// Writes the RSS 0.92 channel element and its contents.
///
/// # Parameters
///
/// * `writer`: A mutable reference to a `quick_xml::Writer` instance.
/// * `options`: A reference to an `RssData` instance, containing the RSS feed options.
///
/// # Returns
///
/// * `Result<()>`: Returns `Ok(())` if the RSS 0.92 channel elements are successfully written.
fn write_rss_channel_0_92<W: std::io::Write>(
    writer: &mut Writer<W>,
    options: &RssData,
) -> Result<()> {
    let mut rss_start = BytesStart::new("rss");
    rss_start.push_attribute(("version", "0.92"));
    writer.write_event(Event::Start(rss_start))?;

    writer.write_event(Event::Start(BytesStart::new("channel")))?;

    write_channel_elements(writer, options)?;
    write_items(writer, options)?;

    writer.write_event(Event::End(BytesEnd::new("channel")))?;
    writer.write_event(Event::End(BytesEnd::new("rss")))?;

    Ok(())
}

/// Writes the RSS 1.0 channel element and its contents.
///
/// This function is used specifically for RSS 1.0 feeds (RDF-based).
///
/// # Parameters
///
/// * `writer`: A mutable reference to a `quick_xml::Writer` instance.
/// * `options`: A reference to an `RssData` instance, containing the RSS feed options.
///
/// # Returns
///
/// * `Result<()>`: Returns `Ok(())` if the RSS 1.0 channel elements are successfully written.
fn write_rss_channel_1_0<W: std::io::Write>(
    writer: &mut Writer<W>,
    options: &RssData,
) -> Result<()> {
    let mut rdf_start = BytesStart::new("rdf:RDF");
    rdf_start.push_attribute((
        "xmlns:rdf",
        "http://www.w3.org/1999/02/22-rdf-syntax-ns#",
    ));
    rdf_start.push_attribute(("xmlns", "http://purl.org/rss/1.0/"));
    writer.write_event(Event::Start(rdf_start))?;

    writer.write_event(Event::Start(BytesStart::new("channel")))?;

    write_channel_elements(writer, options)?;
    write_items(writer, options)?;

    writer.write_event(Event::End(BytesEnd::new("channel")))?;
    writer.write_event(Event::End(BytesEnd::new("rdf:RDF")))?;

    Ok(())
}

/// Writes the RSS 2.0 channel element and its contents.
///
/// This function is used specifically for RSS 2.0 feeds.
///
/// # Parameters
///
/// * `writer`: A mutable reference to a `quick_xml::Writer` instance.
/// * `options`: A reference to an `RssData` instance, containing the RSS feed options.
///
/// # Returns
///
/// * `Result<()>`: Returns `Ok(())` if the RSS 2.0 channel elements are successfully written.
fn write_rss_channel_2_0<W: std::io::Write>(
    writer: &mut Writer<W>,
    options: &RssData,
) -> Result<()> {
    let mut rss_start = BytesStart::new("rss");
    rss_start.push_attribute(("version", "2.0"));
    rss_start
        .push_attribute(("xmlns:atom", "http://www.w3.org/2005/Atom"));
    writer.write_event(Event::Start(rss_start))?;

    writer.write_event(Event::Start(BytesStart::new("channel")))?;

    // Write the channel elements
    write_channel_elements(writer, options)?;

    // Add the image element if it exists
    write_image_element(writer, options)?;

    // Write the atom link element
    write_atom_link_element(writer, options)?;

    // Write the items
    write_items(writer, options)?;

    writer.write_event(Event::End(BytesEnd::new("channel")))?;
    writer.write_event(Event::End(BytesEnd::new("rss")))?;

    Ok(())
}

/// Writes the channel elements to the writer.
///
/// This function takes a mutable reference to a `quick_xml::Writer` instance and an `RssData` instance.
/// It writes the channel elements to the XML feed, including title, link, description, language,
/// pubDate, lastBuildDate, docs, generator, managingEditor, webMaster, category, and ttl.
///
/// # Parameters
///
/// * `writer`: A mutable reference to a `quick_xml::Writer` instance.
/// * `options`: A reference to an `RssData` instance, containing the RSS feed options.
///
/// # Returns
///
/// * `Result<()>`: Returns `Ok(())` if the RSS channel elements are successfully written.
fn write_channel_elements<W: std::io::Write>(
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
///
/// This function is used for RSS 2.0 feeds.
///
/// # Parameters
///
/// * `writer`: A mutable reference to a `quick_xml::Writer` instance.
/// * `options`: A reference to an `RssData` instance, containing the RSS feed options.
///
/// # Returns
///
/// * `Result<()>`: Returns `Ok(())` if the image element is successfully written.
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

/// Writes the item elements to the RSS feed.
///
/// This function is shared between both RSS 1.0 and 2.0 feeds.
///
/// # Parameters
///
/// * `writer`: A mutable reference to a `quick_xml::Writer` instance.
/// * `options`: A reference to an `RssData` instance, containing the RSS feed items.
///
/// # Returns
///
/// * `Result<()>`: Returns `Ok(())` if the item elements are successfully written to the XML feed.
fn write_items<W: std::io::Write>(
    writer: &mut Writer<W>,
    options: &RssData,
) -> Result<()> {
    for item in &options.items {
        write_item(writer, item)?;
    }
    Ok(())
}

/// Writes a single item element to the RSS feed.
///
/// # Parameters
///
/// * `writer`: A mutable reference to a `quick_xml::Writer` instance.
/// * `item`: A reference to an `RssItem` instance, containing the item's details.
///
/// # Returns
///
/// * `Result<()>`: Returns `Ok(())` if the item elements are successfully written to the XML feed.
fn write_item<W: std::io::Write>(
    writer: &mut Writer<W>,
    item: &RssItem,
) -> Result<()> {
    writer.write_event(Event::Start(BytesStart::new("item")))?;

    let item_elements = [
        ("title", &item.title),
        ("link", &item.link),
        ("description", &item.description),
        ("guid", &item.guid),
        ("pubDate", &item.pub_date),
        ("author", &item.author),
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
///
/// This function is used for RSS 2.0 feeds.
///
/// # Parameters
///
/// * `writer`: A mutable reference to a `quick_xml::Writer` where the Atom link element will be written.
/// * `options`: A reference to an `RssData` instance containing the Atom link information.
///
/// # Returns
///
/// * `Result<()>`: Returns `Ok(())` if the Atom link element is successfully written to the XML feed.
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
    use quick_xml::events::Event;
    use quick_xml::Reader;

    fn assert_xml_element(xml: &str, element: &str, expected: &str) {
        let mut reader = Reader::from_str(xml);
        let mut found = false;

        loop {
            match reader.read_event() {
                Ok(Event::Start(ref e))
                    if e.name().as_ref() == element.as_bytes() =>
                {
                    match reader.read_event() {
                        Ok(Event::Text(e)) => {
                            assert_eq!(e.unescape().unwrap(), expected);
                            found = true;
                            break;
                        }
                        _ => continue,
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => panic!(
                    "Error at position {}: {:?}",
                    reader.buffer_position(),
                    e
                ),
                _ => (),
            }
        }
        assert!(
            found,
            "Element '{}' not found or doesn't match expected content",
            element
        );
    }

    #[test]
    fn test_generate_rss_minimal() {
        let rss_data = RssData::new(None)
            .title("Minimal Feed")
            .link("https://example.com")
            .description("A minimal RSS feed");

        let result = generate_rss(&rss_data);
        assert!(result.is_ok());

        let rss_feed = result.unwrap();
        assert!(rss_feed.contains(r#"<rss version="2.0" xmlns:atom="http://www.w3.org/2005/Atom">"#));
        assert_xml_element(&rss_feed, "title", "Minimal Feed");
        assert_xml_element(&rss_feed, "link", "https://example.com");
        assert_xml_element(
            &rss_feed,
            "description",
            "A minimal RSS feed",
        );
    }

    #[test]
    fn test_generate_rss_full() {
        let mut rss_data = RssData::new(None)
            .title("Full Feed")
            .link("https://example.com")
            .description("A full RSS feed")
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
            .atom_link("https://example.com/feed.xml");

        rss_data.add_item(
            RssItem::new()
                .title("Test Item")
                .link("https://example.com/item1")
                .description("A test item")
                .guid("https://example.com/item1")
                .pub_date("Mon, 01 Jan 2023 00:00:00 GMT")
                .author("John Doe"),
        );

        let result = generate_rss(&rss_data);
        assert!(result.is_ok());

        let rss_feed = result.unwrap();
        assert!(rss_feed.contains(r#"<rss version="2.0" xmlns:atom="http://www.w3.org/2005/Atom">"#));
        assert_xml_element(&rss_feed, "title", "Full Feed");
        assert_xml_element(&rss_feed, "link", "https://example.com");
        assert_xml_element(&rss_feed, "description", "A full RSS feed");
        assert_xml_element(&rss_feed, "language", "en-US");
        assert_xml_element(
            &rss_feed,
            "pubDate",
            "Mon, 01 Jan 2023 00:00:00 GMT",
        );
        assert!(rss_feed.contains("<item>"));
        assert_xml_element(&rss_feed, "author", "John Doe");
        assert_xml_element(
            &rss_feed,
            "guid",
            "https://example.com/item1",
        );
    }

    #[test]
    fn test_generate_rss_empty_fields() {
        let rss_data = RssData::new(None)
            .title("Empty Fields Feed")
            .link("https://example.com")
            .description("An RSS feed with some empty fields")
            .language("")
            .pub_date("")
            .last_build_date("");

        let result = generate_rss(&rss_data);
        assert!(result.is_ok());

        let rss_feed = result.unwrap();
        assert_xml_element(&rss_feed, "title", "Empty Fields Feed");
        assert_xml_element(&rss_feed, "link", "https://example.com");
        assert_xml_element(
            &rss_feed,
            "description",
            "An RSS feed with some empty fields",
        );
        assert!(!rss_feed.contains("<language>"));
        assert!(!rss_feed.contains("<pubDate>"));
        assert!(!rss_feed.contains("<lastBuildDate>"));
    }

    #[test]
    fn test_generate_rss_special_characters() {
        let rss_data = RssData::new(None)
            .title("Special & Characters")
            .link("https://example.com/special?param=value")
            .description("Feed with <special> & \"characters\"");

        let result = generate_rss(&rss_data);
        assert!(result.is_ok());

        let rss_feed = result.unwrap();
        assert_xml_element(&rss_feed, "title", "Special & Characters");
        assert_xml_element(
            &rss_feed,
            "link",
            "https://example.com/special?param=value",
        );
        assert_xml_element(
            &rss_feed,
            "description",
            "Feed with <special> & \"characters\"",
        );
    }

    #[test]
    fn test_generate_rss_multiple_items() {
        let mut rss_data = RssData::new(None)
            .title("Multiple Items Feed")
            .link("https://example.com")
            .description("An RSS feed with multiple items");

        for i in 1..=3 {
            rss_data.add_item(
                RssItem::new()
                    .title(format!("Item {}", i))
                    .link(format!("https://example.com/item{}", i))
                    .description(format!("Description for item {}", i))
                    .guid(format!("https://example.com/item{}", i))
                    .pub_date(format!(
                        "Mon, 0{} Jan 2023 00:00:00 GMT",
                        i
                    )),
            );
        }

        let result = generate_rss(&rss_data);
        assert!(result.is_ok());

        let rss_feed = result.unwrap();
        assert_xml_element(&rss_feed, "title", "Multiple Items Feed");

        for i in 1..=3 {
            assert!(rss_feed
                .contains(&format!("<title>Item {}</title>", i)));
            assert!(rss_feed.contains(&format!(
                "<link>https://example.com/item{}</link>",
                i
            )));
            assert!(rss_feed.contains(&format!(
                "<description>Description for item {}</description>",
                i
            )));
            assert!(rss_feed.contains(&format!(
                "<guid>https://example.com/item{}</guid>",
                i
            )));
            assert!(rss_feed.contains(&format!(
                "<pubDate>Mon, 0{} Jan 2023 00:00:00 GMT</pubDate>",
                i
            )));
        }
    }

    #[test]
    fn test_generate_rss_invalid_xml_characters() {
        let rss_data = RssData::new(None)
            .title("Invalid XML \u{0001} Characters")
            .link("https://example.com")
            .description(
                "Description with invalid \u{0000} characters",
            );

        let result = generate_rss(&rss_data);
        assert!(result.is_ok());

        let rss_feed = result.unwrap();
        assert!(!rss_feed.contains('\u{0000}'));
        assert!(!rss_feed.contains('\u{0001}'));
        assert_xml_element(
            &rss_feed,
            "title",
            "Invalid XML  Characters",
        );
        assert_xml_element(
            &rss_feed,
            "description",
            "Description with invalid  characters",
        );
    }

    #[test]
    fn test_generate_rss_long_content() {
        let long_description = "a".repeat(10000);
        let rss_data = RssData::new(None)
            .title("Long Content Feed")
            .link("https://example.com")
            .description(&long_description);

        let result = generate_rss(&rss_data);
        assert!(result.is_ok());

        let rss_feed = result.unwrap();
        assert_xml_element(&rss_feed, "title", "Long Content Feed");
        assert_xml_element(&rss_feed, "description", &long_description);
    }

    #[test]
    fn test_sanitize_content() {
        let input =
            "Text with \u{0000}null\u{0001} and \u{0008}backspace";
        let sanitized = sanitize_content(input);
        assert_eq!(sanitized, "Text with null and backspace");

        let input_with_newlines = "Text with \nnewlines\r\nand\ttabs";
        let sanitized_newlines = sanitize_content(input_with_newlines);
        assert_eq!(sanitized_newlines, input_with_newlines);
    }

    #[test]
    fn test_generate_rss_with_author() {
        let mut rss_data = RssData::new(None)
            .title("Feed with Author")
            .link("https://example.com")
            .description("An RSS feed with author information");

        rss_data.add_item(
            RssItem::new()
                .title("Authored Item")
                .link("https://example.com/item")
                .description("An item with an author")
                .author("John Doe"),
        );

        let result = generate_rss(&rss_data);
        assert!(result.is_ok());

        let rss_feed = result.unwrap();
        assert!(rss_feed.contains("<author>John Doe</author>"));
    }

    #[test]
    fn test_generate_rss_0_90() {
        let rss_data = RssData::new(Some(RssVersion::RSS0_90))
            .title("RSS 0.90 Feed")
            .link("https://example.com")
            .description("RSS 0.90 feed description");

        let result = generate_rss(&rss_data);
        assert!(result.is_ok());

        let rss_feed = result.unwrap();
        assert!(rss_feed.contains(r#"<rss version="0.90">"#));
        assert_xml_element(&rss_feed, "title", "RSS 0.90 Feed");
        assert_xml_element(&rss_feed, "link", "https://example.com");
        assert_xml_element(
            &rss_feed,
            "description",
            "RSS 0.90 feed description",
        );
    }

    #[test]
    fn test_generate_rss_0_91() {
        let rss_data = RssData::new(Some(RssVersion::RSS0_91))
            .title("RSS 0.91 Feed")
            .link("https://example.com")
            .description("RSS 0.91 feed description");

        let result = generate_rss(&rss_data);
        assert!(result.is_ok());

        let rss_feed = result.unwrap();
        assert!(rss_feed.contains(r#"<rss version="0.91">"#));
        assert_xml_element(&rss_feed, "title", "RSS 0.91 Feed");
        assert_xml_element(&rss_feed, "link", "https://example.com");
        assert_xml_element(
            &rss_feed,
            "description",
            "RSS 0.91 feed description",
        );
    }
    #[test]
    fn test_generate_rss_0_92() {
        let rss_data = RssData::new(Some(RssVersion::RSS0_92))
            .title("RSS 0.92 Feed")
            .link("https://example.com")
            .description("RSS 0.92 feed description");

        let result = generate_rss(&rss_data);
        assert!(result.is_ok());

        let rss_feed = result.unwrap();
        assert!(rss_feed.contains(r#"<rss version="0.92">"#));
        assert_xml_element(&rss_feed, "title", "RSS 0.92 Feed");
        assert_xml_element(&rss_feed, "link", "https://example.com");
        assert_xml_element(
            &rss_feed,
            "description",
            "RSS 0.92 feed description",
        );
    }

    #[test]
    fn test_generate_rss_1_0() {
        let rss_data = RssData::new(Some(RssVersion::RSS1_0))
            .title("RSS 1.0 Feed")
            .link("https://example.com")
            .description("RSS 1.0 feed description");

        let result = generate_rss(&rss_data);
        assert!(result.is_ok());

        let rss_feed = result.unwrap();
        assert!(rss_feed.contains(r#"<rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#" xmlns="http://purl.org/rss/1.0/">"#));
        assert_xml_element(&rss_feed, "title", "RSS 1.0 Feed");
        assert_xml_element(&rss_feed, "link", "https://example.com");
        assert_xml_element(
            &rss_feed,
            "description",
            "RSS 1.0 feed description",
        );
    }

    #[test]
    fn test_generate_rss_with_custom_version() {
        let rss_data = RssData::new(Some(RssVersion::RSS2_0))
            .title("Custom RSS Version")
            .link("https://example.com")
            .description("Custom RSS feed for version 2.0");

        let result = generate_rss(&rss_data);
        assert!(result.is_ok());

        let rss_feed = result.unwrap();
        assert!(rss_feed.contains(r#"<rss version="2.0" xmlns:atom="http://www.w3.org/2005/Atom">"#));
        assert_xml_element(&rss_feed, "title", "Custom RSS Version");
        assert_xml_element(&rss_feed, "link", "https://example.com");
        assert_xml_element(
            &rss_feed,
            "description",
            "Custom RSS feed for version 2.0",
        );
    }

    #[test]
    fn test_generate_rss_no_version_defaults_to_2_0() {
        let rss_data = RssData::new(None)
            .title("No Version Feed")
            .link("https://example.com")
            .description("RSS feed with no version defaults to 2.0");

        let result = generate_rss(&rss_data);
        assert!(result.is_ok());

        let rss_feed = result.unwrap();
        assert!(rss_feed.contains(r#"<rss version="2.0" xmlns:atom="http://www.w3.org/2005/Atom">"#));
        assert_xml_element(&rss_feed, "title", "No Version Feed");
        assert_xml_element(&rss_feed, "link", "https://example.com");
        assert_xml_element(
            &rss_feed,
            "description",
            "RSS feed with no version defaults to 2.0",
        );
    }

    #[test]
    fn test_generate_rss_2_0_with_image() {
        let rss_data = RssData::new(Some(RssVersion::RSS2_0))
            .title("Feed with Image")
            .link("https://example.com")
            .description("RSS 2.0 feed with an image")
            .image("https://example.com/image.png");

        let result = generate_rss(&rss_data);
        assert!(result.is_ok());

        let rss_feed = result.unwrap();
        assert!(rss_feed.contains(r#"<rss version="2.0" xmlns:atom="http://www.w3.org/2005/Atom">"#));
        assert_xml_element(&rss_feed, "title", "Feed with Image");
        assert_xml_element(&rss_feed, "link", "https://example.com");
        assert_xml_element(
            &rss_feed,
            "description",
            "RSS 2.0 feed with an image",
        );
        assert!(rss_feed.contains("<image>"));
        assert_xml_element(
            &rss_feed,
            "url",
            "https://example.com/image.png",
        );
    }

    #[test]
    fn test_generate_rss_atom_link() {
        let rss_data = RssData::new(Some(RssVersion::RSS2_0))
            .title("Feed with Atom Link")
            .link("https://example.com")
            .description("RSS 2.0 feed with atom link")
            .atom_link("https://example.com/feed.xml");

        let result = generate_rss(&rss_data);
        assert!(result.is_ok());

        let rss_feed = result.unwrap();
        assert!(rss_feed.contains(r#"<rss version="2.0" xmlns:atom="http://www.w3.org/2005/Atom">"#));
        assert!(rss_feed.contains(r#"<atom:link href="https://example.com/feed.xml" rel="self" type="application/rss+xml"/>"#));
    }
}

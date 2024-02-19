// Copyright © 2024 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

use crate::macro_write_element;
use crate::models::data::RssData;
use quick_xml::{
    events::{BytesDecl, BytesEnd, BytesStart, Event},
    Writer,
};
use std::io::Cursor;
use std::{borrow::Cow, error::Error};

/// Generates an RSS feed from the given `RssData` struct.
///
/// This function creates a complete RSS feed in XML format based on the data contained in the provided `RssData`.
/// It generates the feed by creating a series of XML elements corresponding to the fields of the `RssData`,
/// and writing them to a `Writer` object.
///
/// The generated RSS feed is returned as a `String`. If an error occurs during generation, it returns an error.
pub fn generate_rss(
    options: &RssData,
) -> Result<String, Box<dyn Error>> {
    // Create a new `Writer` instance.
    let mut writer = Writer::new(Cursor::new(Vec::new()));

    // Write the XML declaration.
    writer.write_event(Event::Decl(
        BytesDecl::new("1.0", Some("utf-8"), None)
    ))?;

    // Start the `rss` element.
    let mut rss_start = BytesStart::new("rss");
    rss_start.push_attribute(("version", "2.0"));
    rss_start
        .push_attribute(("xmlns:atom", "http://www.w3.org/2005/Atom"));
    writer.write_event(Event::Start(rss_start))?;

    // Start the `channel` element.
    writer.write_event(Event::Start(BytesStart::new("channel")))?;

    macro_write_element!(writer, "title", &options.title)?;
    macro_write_element!(writer, "link", &options.link)?;
    macro_write_element!(writer, "description", &options.description)?;
    macro_write_element!(writer, "language", &options.language)?;
    macro_write_element!(writer, "pubDate", &options.pub_date)?;
    macro_write_element!(
        writer,
        "lastBuildDate",
        &options.last_build_date
    )?;
    macro_write_element!(writer, "docs", &options.docs)?;
    macro_write_element!(writer, "generator", &options.generator)?;
    macro_write_element!(
        writer,
        "managingEditor",
        &options.managing_editor
    )?;
    macro_write_element!(writer, "webMaster", &options.webmaster)?;
    macro_write_element!(writer, "category", &options.category)?;
    macro_write_element!(writer, "ttl", &options.ttl)?;

    // Write the `image` element.
    writer.write_event(Event::Start(BytesStart::new("image")))?;
    macro_write_element!(writer, "url", &options.image)?;
    macro_write_element!(writer, "title", &options.title)?;
    macro_write_element!(writer, "link", &options.link)?;
    writer.write_event(Event::End(BytesEnd::new("image")))?;

    // Write the `atom:link` element.
    let mut atom_link_start =
        BytesStart::new(Cow::Borrowed("atom:link").into_owned());
    atom_link_start.push_attribute(
        ("href", options.atom_link.to_string().as_str())
    );
    atom_link_start.push_attribute(("rel", "self"));
    atom_link_start.push_attribute(("type", "application/rss+xml"));
    writer.write_event(Event::Empty(atom_link_start))?;

    // Write the `item` element.
    writer.write_event(Event::Start(BytesStart::new("item")))?;

    macro_write_element!(writer, "author", &options.author)?;
    macro_write_element!(
        writer,
        "description",
        &options.item_description
    )?;
    macro_write_element!(writer, "guid", &options.item_guid)?;
    macro_write_element!(writer, "link", &options.item_link)?;
    macro_write_element!(writer, "pubDate", &options.item_pub_date)?;
    macro_write_element!(writer, "title", &options.item_title)?;

    // End the `item` element.
    writer.write_event(Event::End(BytesEnd::new("item")))?;

    // End the `channel` element.
    writer.write_event(Event::End(BytesEnd::new("channel")))?;

    // End the `rss` element.
    writer.write_event(Event::End(BytesEnd::new("rss")))?;

    // Convert the XML to a string.
    let xml = writer.into_inner().into_inner();
    let rss_str = String::from_utf8(xml)?;

    // Return the RSS feed as a string.
    Ok(rss_str)
}

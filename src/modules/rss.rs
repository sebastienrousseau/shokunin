// Copyright © 2024 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

use crate::macro_write_element;
use crate::models::data::RssData;
use quick_xml::{
    escape::escape,
    events::{BytesDecl, BytesEnd, BytesStart, Event},
    Writer,
};
use std::error::Error;

/// Generates an RSS feed from the given `RssData` struct.
///
/// This function creates a complete RSS feed in XML format based on the data contained in the provided `RssData`.
/// It generates the feed by creating a series of XML elements corresponding to the fields of the `RssData`,
/// and writing them to a `Writer` object.
///
/// The generated RSS feed is returned as a `String`. If an error occurs during generation, it returns an error.
pub fn generate_rss(options: &RssData) -> Result<String, Box<dyn Error>> {
    let mut writer = Writer::new(Vec::new());

    // Write the XML declaration.
    writer.write_event(Event::Decl(BytesDecl::new("1.0", Some("utf-8"), None)))?;

    // Start the `rss` element.
    let mut rss_start = BytesStart::new("rss");
    rss_start.push_attribute(("version", "2.0"));
    rss_start.push_attribute(("xmlns:atom", "http://www.w3.org/2005/Atom"));
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

    let xml = writer.into_inner();
    let rss_str = String::from_utf8(xml)?;

    Ok(rss_str)
}

/// Write the specified elements to the writer.
pub fn write_elements<W: std::io::Write>(writer: &mut Writer<W>, options: &RssData) -> Result<(), Box<dyn Error>> {
    macro_write_element!(writer, "title", escape(&options.title))?;
    macro_write_element!(writer, "link", escape(&options.link))?;
    macro_write_element!(writer, "description", escape(&options.description))?;
    macro_write_element!(writer, "language", escape(&options.language))?;
    macro_write_element!(writer, "pubDate", escape(&options.pub_date))?;
    macro_write_element!(writer, "lastBuildDate", escape(&options.last_build_date))?;
    macro_write_element!(writer, "docs", escape(&options.docs))?;
    macro_write_element!(writer, "generator", escape(&options.generator))?;
    macro_write_element!(writer, "managingEditor", escape(&options.managing_editor))?;
    macro_write_element!(writer, "webMaster", escape(&options.webmaster))?;
    macro_write_element!(writer, "category", escape(&options.category))?;
    macro_write_element!(writer, "ttl", escape(&options.ttl))?;
    Ok(())
}

/// Write the image element to the writer.
pub fn write_image_element<W: std::io::Write>(writer: &mut Writer<W>, options: &RssData) -> Result<(), Box<dyn Error>> {
    writer.write_event(Event::Start(BytesStart::new("image")))?;
    macro_write_element!(writer, "url", &options.image)?;
    macro_write_element!(writer, "title", &options.title)?;
    macro_write_element!(writer, "link", &options.link)?;
    writer.write_event(Event::End(BytesEnd::new("image")))?;
    Ok(())
}

/// Write the Atom link element to the writer.
pub fn write_atom_link_element<W: std::io::Write>(writer: &mut Writer<W>, options: &RssData) -> Result<(), Box<dyn Error>> {
    let mut atom_link_start = BytesStart::new("atom:link");
    atom_link_start.push_attribute((
        "href",
        options.atom_link.to_string().as_str(),
    ));
    atom_link_start.push_attribute(("rel", "self"));
    atom_link_start.push_attribute(("type", "application/rss+xml"));
    writer.write_event(Event::Empty(atom_link_start))?;
    Ok(())
}

/// Write the `item` element.
pub fn write_item_element<W: std::io::Write>(writer: &mut Writer<W>, options: &RssData) -> Result<(), Box<dyn Error>> {
    writer.write_event(Event::Start(BytesStart::new("item")))?;
    macro_write_element!(writer, "author", escape(&options.author))?;
    macro_write_element!(writer, "description", escape(&options.item_description))?;
    macro_write_element!(writer, "guid", escape(&options.item_guid))?;
    macro_write_element!(writer, "link", escape(&options.item_link))?;
    macro_write_element!(writer, "pubDate", escape(&options.item_pub_date))?;
    macro_write_element!(writer, "title", escape(&options.item_title))?;
    writer.write_event(Event::End(BytesEnd::new("item")))?;
    Ok(())
}

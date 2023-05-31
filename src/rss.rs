// Copyright © 2023 Shokunin (職人) Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

use super::macro_write_element;
use quick_xml::{
    events::{BytesDecl, BytesEnd, BytesStart, Event},
    Writer,
};
use std::borrow::Cow;
use std::io::Cursor;

/// The `RssOptions` struct holds the options for an RSS feed.
///
/// # Fields
///
/// * `title`: The title of the RSS feed.
/// * `link`: The link to the RSS feed.
/// * `description`: The description of the RSS feed.
/// * `generator`: The generator of the RSS feed.
/// * `language`: The language of the RSS feed.
/// * `atom_link`: The atom link of the RSS feed.
/// * `webmaster`: The webmaster of the RSS feed.
/// * `last_build_date`: The last build date of the RSS feed.
/// * `pub_date`: The publication date of the RSS feed.
/// * `item_title`: The title of the RSS feed item.
/// * `item_link`: The link to the RSS feed item.
/// * `item_guid`: The GUID of the RSS feed item.
/// * `item_description`: The description of the RSS feed item.
/// * `item_pub_date`: The publication date of the RSS feed item.
///
#[derive(Debug, Default, PartialEq, Eq, Hash, Clone)]
pub struct RssOptions {
    /// The title of the RSS feed.
    pub title: String,
    /// The URL of the RSS feed.
    pub link: String,
    /// The description of the RSS feed.
    pub description: String,
    /// The generator of the RSS feed.
    pub generator: String,
    /// The language of the RSS feed.
    pub language: String,
    /// The link to the atom feed.
    pub atom_link: String,
    /// The webmaster of the RSS feed.
    pub webmaster: String,
    /// The last build date of the RSS feed.
    pub last_build_date: String,
    /// The publication date of the RSS feed.
    pub pub_date: String,
    /// The title of the RSS item.
    pub item_title: String,
    /// The link of the RSS item.
    pub item_link: String,
    /// The guid of the RSS item.
    pub item_guid: String,
    /// The description of the RSS item.
    pub item_description: String,
    /// The publication date of the RSS item.
    pub item_pub_date: String,
}

impl RssOptions {
    /// Creates a new `RssOptions` struct with default values.
    pub fn new() -> RssOptions {
        RssOptions::default()
    }
}

/// Generates an RSS feed from the given `RssOptions` struct.
pub fn generate_rss(
    options: &RssOptions,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut writer = Writer::new(Cursor::new(Vec::new()));
    writer.write_event(Event::Decl(BytesDecl::new(
        "1.0",
        Some("utf-8"),
        None,
    )))?;

    let mut rss_start = BytesStart::new("rss");
    rss_start.push_attribute(("version", "2.0"));
    rss_start
        .push_attribute(("xmlns:atom", "http://www.w3.org/2005/Atom"));
    writer.write_event(Event::Start(rss_start))?;

    writer.write_event(Event::Start(BytesStart::new("channel")))?;

    let channel_elements = [
        ("title", &options.title),
        ("link", &options.link),
        ("description", &options.description),
        ("generator", &options.generator),
        ("language", &options.language),
        ("lastBuildDate", &options.last_build_date),
        ("webMaster", &options.webmaster),
        ("pubDate", &options.pub_date),
    ];

    for &(element, value) in channel_elements.iter() {
        macro_write_element!(writer, element, value)?;
    }

    let mut atom_link_start = BytesStart::new("atom:link");
    atom_link_start
        .push_attribute(("href", options.atom_link.as_str()));
    atom_link_start.push_attribute(("rel", "self"));
    atom_link_start.push_attribute(("type", "application/rss+xml"));
    writer.write_event(Event::Empty(atom_link_start))?;

    writer.write_event(Event::Start(BytesStart::new("item")))?;

    let item_elements = [
        ("title", &options.item_title),
        ("link", &options.item_link),
        ("pubDate", &options.item_pub_date),
        ("guid", &options.item_guid),
        ("description", &options.item_description),
    ];

    for &(element, value) in item_elements.iter() {
        macro_write_element!(&mut writer, element, value)?;
    }

    writer.write_event(Event::End(BytesEnd::new("item")))?;
    writer.write_event(Event::End(BytesEnd::new("channel")))?;
    writer.write_event(Event::End(BytesEnd::new("rss")))?;

    let xml = writer.into_inner().into_inner();
    let rss_str = String::from_utf8(xml)?;

    Ok(rss_str)
}

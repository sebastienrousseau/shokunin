// Copyright © 2023 Shokunin (職人) Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

use super::macro_write_element;
use quick_xml::{
    events::{BytesDecl, BytesEnd, BytesStart, Event},
    Writer,
};
use std::borrow::Cow;
use std::io::Cursor;

/// The `RssOptions` struct holds all necessary options and data for an RSS feed.
///
/// This includes everything from metadata about the RSS feed itself, such as its title and language,
/// to information about individual items in the feed, such as their titles and publication dates.
///
/// The values contained in an instance of `RssOptions` can be used to generate a complete RSS feed in XML format.
#[derive(Debug, Default, PartialEq, Eq, Hash, Clone)]
pub struct RssOptions {
    /// The link to the atom feed.
    pub atom_link: String,
    /// The category of the RSS feed.
    pub category: String,
    /// The cloud of the RSS feed.
    pub cloud: String,
    /// The copyright notice for the content of the feed.
    pub copyright: String,
    /// The description of the RSS feed.
    pub description: String,
    /// The docs of the RSS feed.
    pub docs: String,
    /// The Enclosure of the RSS item. This is used for multimedia content.
    pub enclosure: String,
    /// The generator of the RSS feed.
    pub generator: String,
    /// The image of the RSS feed. This can be a URL pointing to an image file.
    pub image: String,
    /// The Guid of the RSS item. This is a unique identifier for the item.
    pub item_guid: String,
    /// The description of the RSS item.
    pub item_description: String,
    /// The link of the RSS item.
    pub item_link: String,
    /// The publication date of the RSS item.
    pub item_pub_date: String,
    /// The title of the RSS item.
    pub item_title: String,
    /// The language of the RSS feed.
    pub language: String,
    /// The last build date of the RSS feed.
    pub last_build_date: String,
    /// The URL of the RSS feed.
    pub link: String,
    /// The managing editor of the RSS feed.
    pub managing_editor: String,
    /// The publication date of the RSS feed.
    pub pub_date: String,
    /// The title of the RSS feed.
    pub title: String,
    /// Time To Live: the number of minutes the feed should be cached before refreshing.
    pub ttl: String,
    /// The webmaster of the RSS feed.
    pub webmaster: String,
}

impl RssOptions {
    /// Creates a new `RssOptions` struct with default values for all fields.
    ///
    /// This is a convenience function that makes it easy to create a new `RssOptions` without having to specify every field.
    /// Fields can then be set individually on the returned instance.
    pub fn new() -> RssOptions {
        RssOptions::default()
    }
}

/// Generates an RSS feed from the given `RssOptions` struct.
///
/// This function creates a complete RSS feed in XML format based on the data contained in the provided `RssOptions`.
/// It generates the feed by creating a series of XML elements corresponding to the fields of the `RssOptions`,
/// and writing them to a `Writer` object.
///
/// The generated RSS feed is returned as a `String`. If an error occurs during generation, it returns an error.
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
        ("category", &options.category),
        ("cloud", &options.cloud),
        ("copyright", &options.copyright),
        ("description", &options.description),
        ("docs", &options.docs),
        ("generator", &options.generator),
        ("image", &options.image),
        ("language", &options.language),
        ("lastBuildDate", &options.last_build_date),
        ("link", &options.link),
        ("managingEditor", &options.managing_editor),
        ("pubDate", &options.pub_date),
        ("title", &options.title),
        ("ttl", &options.ttl),
        ("webMaster", &options.webmaster),
    ];

    for &(element, value) in channel_elements.iter() {
        macro_write_element!(writer, element, value)?;
    }

    let mut atom_link_start =
        BytesStart::new(Cow::Borrowed("atom:link").into_owned());
    atom_link_start
        .push_attribute(("href", options.atom_link.as_str()));
    atom_link_start.push_attribute(("rel", "self"));
    atom_link_start.push_attribute(("type", "application/rss+xml"));
    writer.write_event(Event::Empty(atom_link_start))?;

    writer.write_event(Event::Start(BytesStart::new("item")))?;

    let item_elements = [
        ("description", &options.item_description),
        ("guid", &options.item_guid),
        ("link", &options.item_link),
        ("pubDate", &options.item_pub_date),
        ("title", &options.item_title),
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

// Copyright © 2024 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Contains macros related to directory operations.

//! This module provides macros for generating RSS feeds and setting fields for RSS data.
//!
//! It includes macros for generating RSS feeds from provided data using the quick_xml crate.
//! The `macro_generate_rss` macro generates a complete RSS feed in XML format based on the provided metadata values,
//! while the `macro_set_rss_data_fields` macro is used to set fields of the `RssData` struct.
//!
//! # `macro_generate_rss` Macro
//!
//! The `macro_generate_rss` macro generates a complete RSS feed in XML format based on the data contained in the provided `RssData`.
//! It dynamically generates XML elements for each field of the `RssData` using the provided metadata values and
//! writes them to the specified Writer instance.
//!
//! ## Arguments
//!
//! - `$writer`: The Writer instance to write the generated XML events.
//! - `$options`: The RssData instance containing the metadata values for generating the RSS feed.
//!
//! ## Returns
//!
//! Returns `Result<Writer<std::io::Cursor<Vec<u8>>>, Box<dyn Error>>` indicating success or an error if XML writing fails.
//!
//! # `macro_set_rss_data_fields` Macro
//!
//! The `macro_set_rss_data_fields` macro sets fields of the `RssData` struct.
//!
//! ## Arguments
//!
//! - `$rss_data`: The `RssData` struct to set fields for.
//! - `$field`: The field to set.
//! - `$value`: The value to set for the field.
//!
//! # Note
//!
//! This module assumes familiarity with the quick_xml crate and its usage. Users are encouraged
//! to review quick_xml documentation for a better understanding of how to work with XML generation in Rust.

/// # `macro_generate_rss` Macro
///
/// Generates an RSS feed from the given `RssData` struct.
///
/// This macro generates a complete RSS feed in XML format based on the data contained in the provided `RssData`.
/// It dynamically generates XML elements for each field of the `RssData` using the provided metadata values and
/// writes them to the specified Writer instance.
///
/// # Arguments
///
/// * `$writer` - The Writer instance to write the generated XML events.
/// * `$options` - The RssData instance containing the metadata values for generating the RSS feed.
///
/// # Returns
///
/// Returns `Result<Writer<std::io::Cursor<Vec<u8>>>, Box<dyn Error>>` indicating success or an error if XML writing fails.
///
#[macro_export]
macro_rules! macro_generate_rss {
    ($writer:expr, $options:expr) => {{
        use quick_xml::events::{BytesStart, BytesEnd, BytesDecl, Event};
        use quick_xml::Writer;
        use std::io::Cursor;

        // Create a Writer instance from the provided expression
        let mut writer = $writer;

        // Start building the RSS feed with XML declaration
        writer.write_event(Event::Decl(BytesDecl::new("1.0", Some("utf-8"), None)))
            .unwrap();

        // Start the <rss> tag
        let mut rss_start = BytesStart::new("rss");
        rss_start.push_attribute(("version", "2.0"));
        rss_start.push_attribute(("xmlns:atom", "http://www.w3.org/2005/Atom"));
        writer.write_event(Event::Start(rss_start)).unwrap();

        // Start the <channel> tag
        writer.write_event(Event::Start(BytesStart::new("channel"))).unwrap();

        // Atom link should be inside the channel, and it's a self-closing tag
        let mut atom_link_start = BytesStart::new("atom:link");
        atom_link_start.push_attribute(("href", $options.atom_link.as_str()));
        atom_link_start.push_attribute(("rel", "self"));
        atom_link_start.push_attribute(("type", "application/rss+xml"));
        writer.write_event(Event::Empty(atom_link_start)).unwrap(); // Use Event::Empty for self-closing tag

        // Write other channel elements
        macro_write_element!(writer, "title", &$options.title)?;
        macro_write_element!(writer, "link", &$options.link)?;
        macro_write_element!(writer, "description", &$options.description)?;
        macro_write_element!(writer, "language", &$options.language)?;
        macro_write_element!(writer, "pubDate", &$options.pub_date)?;
        macro_write_element!(writer, "lastBuildDate", &$options.last_build_date)?;
        macro_write_element!(writer, "docs", &$options.docs)?;
        macro_write_element!(writer, "generator", &$options.generator)?;
        macro_write_element!(writer, "managingEditor", &$options.managing_editor)?;
        macro_write_element!(writer, "webMaster", &$options.webmaster)?;
        macro_write_element!(writer, "category", &$options.category)?;
        macro_write_element!(writer, "ttl", &$options.ttl)?;

        // Write image element
        writer.write_event(Event::Start(BytesStart::new("image")))?;
        macro_write_element!(writer, "url", &$options.image)?;
        macro_write_element!(writer, "title", &$options.title)?;
        macro_write_element!(writer, "link", &$options.link)?;
        writer.write_event(Event::End(BytesEnd::new("image")))?;

        // Write item element
        writer.write_event(Event::Start(BytesStart::new("item")))?;
        macro_write_element!(writer, "author", &$options.author)?;
        macro_write_element!(writer, "description", &$options.item_description)?;
        macro_write_element!(writer, "guid", &$options.item_guid)?;
        macro_write_element!(writer, "link", &$options.item_link)?;
        macro_write_element!(writer, "pubDate", &$options.item_pub_date)?;
        macro_write_element!(writer, "title", &$options.item_title)?;
        writer.write_event(Event::End(BytesEnd::new("item")))?;

        // Close the <channel> and <rss> tags
        writer.write_event(Event::End(BytesEnd::new("channel")))?;
        writer.write_event(Event::End(BytesEnd::new("rss")))?;

        // Return the Writer instance
        Ok(writer)
    }};
}

/// # `macro_set_rss_data_fields` Macro
///
/// Sets fields of the `RssData` struct.
///
/// This macro sets the fields of the `RssData` struct with the provided values.
///
/// # Arguments
///
/// * `$rss_data` - The `RssData` struct to set fields for.
/// * `$field` - The field to set.
/// * `$value` - The value to set for the field.
///
#[macro_export]
macro_rules! macro_set_rss_data_fields {
    ($rss_data:expr, $field:ident, $value:expr) => {
        $rss_data.set(stringify!($field), $value);
    };
}

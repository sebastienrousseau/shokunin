// Copyright © 2024 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! This module provides macros for generating RSS feeds and setting fields for RSS data.
//!
//! It includes macros for generating RSS feeds from provided data using the quick_xml crate,
//! as well as macros for setting fields in the `RssData` struct.

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
/// # Example
///
/// ```text
/// use ssg_rss::{RssData, macro_generate_rss, macro_write_element};
/// use quick_xml::Writer;
/// use std::io::Cursor;
///
/// fn generate_rss() -> Result<(), Box<dyn std::error::Error>> {
///     let mut writer = Writer::new(Cursor::new(Vec::new()));
///     let options = RssData::new()
///         .title("My Blog")
///         .link("https://example.com")
///         .description("A blog about Rust");
///
///     let result: Result<Writer<Cursor<Vec<u8>>>, Box<dyn std::error::Error>> = macro_generate_rss!(writer, options);
///     assert!(result.is_ok());
///     Ok(())
/// }
/// ```text
/// generate_rss().unwrap();
/// ```
#[macro_export]
macro_rules! macro_generate_rss {
    ($writer:expr, $options:expr) => {{
        use quick_xml::events::{
            BytesDecl, BytesEnd, BytesStart, BytesText, Event,
        };

        let mut writer = $writer;

        writer.write_event(Event::Decl(BytesDecl::new(
            "1.0",
            Some("utf-8"),
            None,
        )))?;

        let mut rss_start = BytesStart::new("rss");
        rss_start.push_attribute(("version", "2.0"));
        rss_start.push_attribute((
            "xmlns:atom",
            "http://www.w3.org/2005/Atom",
        ));
        writer.write_event(Event::Start(rss_start))?;

        writer.write_event(Event::Start(BytesStart::new("channel")))?;

        macro_write_element!(writer, "title", &$options.title)?;
        macro_write_element!(writer, "link", &$options.link)?;
        macro_write_element!(
            writer,
            "description",
            &$options.description
        )?;
        macro_write_element!(writer, "language", &$options.language)?;
        macro_write_element!(writer, "pubDate", &$options.pub_date)?;
        macro_write_element!(
            writer,
            "lastBuildDate",
            &$options.last_build_date
        )?;
        macro_write_element!(writer, "docs", &$options.docs)?;
        macro_write_element!(writer, "generator", &$options.generator)?;
        macro_write_element!(
            writer,
            "managingEditor",
            &$options.managing_editor
        )?;
        macro_write_element!(writer, "webMaster", &$options.webmaster)?;
        macro_write_element!(writer, "category", &$options.category)?;
        macro_write_element!(writer, "ttl", &$options.ttl)?;

        // Write image element
        if !$options.image.is_empty() {
            writer
                .write_event(Event::Start(BytesStart::new("image")))?;
            macro_write_element!(writer, "url", &$options.image)?;
            macro_write_element!(writer, "title", &$options.title)?;
            macro_write_element!(writer, "link", &$options.link)?;
            writer.write_event(Event::End(BytesEnd::new("image")))?;
        }

        // Write atom:link
        if !$options.atom_link.is_empty() {
            let mut atom_link_start = BytesStart::new("atom:link");
            atom_link_start
                .push_attribute(("href", $options.atom_link.as_str()));
            atom_link_start.push_attribute(("rel", "self"));
            atom_link_start
                .push_attribute(("type", "application/rss+xml"));
            writer.write_event(Event::Empty(atom_link_start))?;
        }

        // Write item
        writer.write_event(Event::Start(BytesStart::new("item")))?;
        macro_write_element!(writer, "title", &$options.item_title)?;
        macro_write_element!(writer, "link", &$options.item_link)?;
        macro_write_element!(
            writer,
            "description",
            &$options.item_description
        )?;
        macro_write_element!(writer, "author", &$options.author)?;
        macro_write_element!(writer, "guid", &$options.item_guid)?;
        macro_write_element!(
            writer,
            "pubDate",
            &$options.item_pub_date
        )?;
        writer.write_event(Event::End(BytesEnd::new("item")))?;

        writer.write_event(Event::End(BytesEnd::new("channel")))?;
        writer.write_event(Event::End(BytesEnd::new("rss")))?;

        Ok(writer)
    }};
}

/// Writes an XML element with the given name and content.
///
/// This macro is used internally by the `macro_generate_rss` macro to write individual XML elements.
///
/// # Arguments
///
/// * `$writer` - The Writer instance to write the XML element.
/// * `$name` - The name of the XML element.
/// * `$content` - The content of the XML element.
///
/// # Example
///
/// ```
/// use ssg_rss::macro_write_element;
/// use quick_xml::Writer;
/// use std::io::Cursor;
/// use quick_xml::events::{BytesStart, BytesEnd, BytesText, Event};
///
/// fn _doctest_main_ssg_rss_src_macros_rs_153_0() -> Result<(), Box<dyn std::error::Error>> {
/// let mut writer = Writer::new(Cursor::new(Vec::new()));
/// macro_write_element!(writer, "title", "My Blog").unwrap();
///
/// Ok(())
/// }
/// ```
#[macro_export]
macro_rules! macro_write_element {
    ($writer:expr, $name:expr, $content:expr) => {{
        if !$content.is_empty() {
            $writer
                .write_event(Event::Start(BytesStart::new($name)))?;
            $writer
                .write_event(Event::Text(BytesText::new($content)))?;
            $writer.write_event(Event::End(BytesEnd::new($name)))?;
        }
        Ok::<(), quick_xml::Error>(())
    }};
}

/// Sets fields of the `RssData` struct.
///
/// This macro provides a convenient way to set multiple fields of an `RssData` struct in one go.
///
/// # Arguments
///
/// * `$rss_data` - The `RssData` struct to set fields for.
/// * `$($field:ident = $value:expr),+` - A comma-separated list of field-value pairs to set.
///
/// # Example
///
/// ```
/// use ssg_rss::{RssData, macro_set_rss_data_fields};
///
/// let mut rss_data = RssData::new(None);
/// macro_set_rss_data_fields!(rss_data,
///     title = "My Blog",
///     link = "https://example.com",
///     description = "A blog about Rust"
/// );
/// assert_eq!(rss_data.title, "My Blog");
/// assert_eq!(rss_data.link, "https://example.com");
/// assert_eq!(rss_data.description, "A blog about Rust");
/// ```
#[macro_export]
macro_rules! macro_set_rss_data_fields {
    ($rss_data:expr, $($field:ident = $value:expr),+ $(,)?) => {
        $rss_data = $rss_data $(.set(stringify!($field), $value))+
    };
}

/// # `macro_get_args` Macro
///
/// Retrieve a named argument from a `clap::ArgMatches` object.
///
/// ## Arguments
///
/// * `$matches` - A `clap::ArgMatches` object representing the parsed command-line arguments.
/// * `$name` - A string literal specifying the name of the argument to retrieve.
///
/// ## Behaviour
///
/// The `macro_get_args` macro retrieves the value of the named argument `$name` from the `$matches` object. If the argument is found and its value can be converted to `String`, the macro returns the value as a `Result<String, String>`. If the argument is not found or its value cannot be converted to `String`, an `Err` variant is returned with an error message indicating the omission of the required parameter.
///
/// The error message includes the name of the omitted parameter (`$name`) to assist with troubleshooting and providing meaningful feedback to users.
///
/// ## Notes
///
/// - This macro assumes the availability of the `clap` crate and the presence of a valid `ArgMatches` object.
/// - Make sure to adjust the code example by providing a valid `ArgMatches` object and replacing `"arg_name"` with the actual name of the argument you want to retrieve.
///
#[macro_export]
macro_rules! macro_get_args {
    ($matches:ident, $name:expr) => {
        $matches.get_one::<String>($name).ok_or(format!(
            "❌ Error: A required parameter was omitted. Add the required parameter. \"{}\".",
            $name
        ))?
    };
}

/// # `macro_metadata_option` Macro
///
/// Extracts an option value from metadata.
///
/// ## Usage
///
/// ```rust
/// use std::collections::HashMap;
/// use ssg_rss::macro_metadata_option;
///
/// let mut metadata = HashMap::new();
/// metadata.insert("key", "value");
/// let value = macro_metadata_option!(metadata, "key");
/// println!("{}", value);
/// ```
///
/// ## Arguments
///
/// * `$metadata` - A mutable variable that represents the metadata (of type `HashMap<String, String>` or any other type that supports the `get` and `cloned` methods).
/// * `$key` - A string literal that represents the key to search for in the metadata.
///
/// ## Behaviour
///
/// The `macro_metadata_option` macro is used to extract an option value from metadata. It takes a mutable variable representing the metadata and a string literal representing the key as arguments, and uses the `get` method of the metadata to find an option value with the specified key.
///
/// If the key exists in the metadata, the macro clones the value and returns it. If the key does not exist, it returns the default value for the type of the metadata values.
///
/// The macro is typically used in contexts where metadata is stored in a data structure that supports the `get` and `cloned` methods, such as a `HashMap<String, String>`.
///
/// # Example
///
/// ```
/// use std::collections::HashMap;
/// use ssg_rss::macro_metadata_option;
///
/// let mut metadata = HashMap::new();
/// metadata.insert("key".to_string(), "value".to_string());
/// let value = macro_metadata_option!(metadata, "key");
/// assert_eq!(value, "value");
/// ```
///
#[macro_export]
macro_rules! macro_metadata_option {
    ($metadata:ident, $key:expr) => {
        $metadata.get($key).cloned().unwrap_or_default()
    };
}

#[cfg(test)]
mod tests {
    use crate::RssData;
    use quick_xml::Writer;
    use std::collections::HashMap;
    use std::io::Cursor;

    #[test]
    fn test_macro_generate_rss() -> Result<(), quick_xml::Error> {
        let options = RssData::new(None)
            .title("Test RSS Feed")
            .link("https://example.com")
            .description("A test RSS feed");

        let writer = Writer::new(Cursor::new(Vec::new()));
        let result: Result<
            Writer<Cursor<Vec<u8>>>,
            Box<dyn std::error::Error>,
        > = macro_generate_rss!(writer, options);

        assert!(result.is_ok());
        let writer = result.unwrap();
        let content =
            String::from_utf8(writer.into_inner().into_inner())
                .unwrap();

        assert!(content.contains(r#"<rss version="2.0" xmlns:atom="http://www.w3.org/2005/Atom">"#));
        assert!(content.contains("<title>Test RSS Feed</title>"));
        assert!(content.contains("<link>https://example.com</link>"));
        assert!(content
            .contains("<description>A test RSS feed</description>"));

        Ok(())
    }

    /// Test generating an RSS feed using valid RSS data.
    /// Ensures that the macro generates valid XML elements for required fields.
    #[test]
    fn test_macro_generate_rss_valid_data(
    ) -> Result<(), quick_xml::Error> {
        let options = RssData::new(None)
            .title("Test RSS Feed")
            .link("https://example.com")
            .description("A test RSS feed");

        let writer = Writer::new(Cursor::new(Vec::new()));
        let result: Result<
            Writer<Cursor<Vec<u8>>>,
            Box<dyn std::error::Error>,
        > = macro_generate_rss!(writer, options);

        assert!(result.is_ok());
        let writer = result.unwrap();
        let content =
            String::from_utf8(writer.into_inner().into_inner())
                .unwrap();

        assert!(content.contains(r#"<rss version="2.0" xmlns:atom="http://www.w3.org/2005/Atom">"#));
        assert!(content.contains("<title>Test RSS Feed</title>"));
        assert!(content.contains("<link>https://example.com</link>"));
        assert!(content
            .contains("<description>A test RSS feed</description>"));

        Ok(())
    }

    /// Test generating an RSS feed with missing optional fields.
    /// Ensures that the macro still generates valid RSS but excludes the missing fields.
    #[test]
    fn test_macro_generate_rss_missing_fields(
    ) -> Result<(), quick_xml::Error> {
        let options = RssData::new(None)
            .title("Test RSS Feed")
            .link("https://example.com");

        let writer = Writer::new(Cursor::new(Vec::new()));
        let result: Result<
            Writer<Cursor<Vec<u8>>>,
            Box<dyn std::error::Error>,
        > = macro_generate_rss!(writer, options);

        assert!(result.is_ok());
        let writer = result.unwrap();
        let content =
            String::from_utf8(writer.into_inner().into_inner())
                .unwrap();

        assert!(content.contains("<title>Test RSS Feed</title>"));
        assert!(content.contains("<link>https://example.com</link>"));
        assert!(!content.contains("<description>")); // No description in this case

        Ok(())
    }

    /// Test setting multiple fields on an `RssData` struct using the macro.
    /// Ensures that all fields are set correctly and in order.
    #[test]
    fn test_macro_set_rss_data_fields(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut rss_data = RssData::new(None);
        macro_set_rss_data_fields!(
            rss_data,
            title = "My Blog",
            link = "https://example.com",
            description = "A blog about Rust"
        );

        assert_eq!(rss_data.title, "My Blog");
        assert_eq!(rss_data.link, "https://example.com");
        assert_eq!(rss_data.description, "A blog about Rust");

        Ok(())
    }

    /// Test metadata option macro when the key exists.
    /// Ensures the correct value is returned for a given key.
    #[test]
    fn test_macro_metadata_option_existing_key() {
        let mut metadata = HashMap::new();
        metadata.insert("author".to_string(), "John Doe".to_string());

        let value = macro_metadata_option!(metadata, "author");
        assert_eq!(value, "John Doe");
    }

    /// Test metadata option macro when the key is missing.
    /// Ensures that it returns an empty string or default value in case the key is not found.
    #[test]
    fn test_macro_metadata_option_missing_key() {
        let mut metadata = HashMap::new();
        metadata.insert("title".to_string(), "Rust Blog".to_string());

        let value = macro_metadata_option!(metadata, "author"); // Key "author" does not exist
        assert_eq!(value, ""); // Should return empty string by default
    }

    /// Test metadata option macro with an empty HashMap.
    /// Ensures it handles an empty metadata collection gracefully.
    #[test]
    fn test_macro_metadata_option_empty_metadata() {
        let metadata: HashMap<String, String> = HashMap::new();

        let value = macro_metadata_option!(metadata, "nonexistent_key");
        assert_eq!(value, "");
    }
}

// Copyright © 2024 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Contains macros related to directory operations.

//! Contains macros related to HTML generation.
//!
//! This module provides macros for generating HTML meta tags, writing XML elements, and generating HTML meta tags
//! from metadata HashMaps or field-value pairs.
//!
//! # Meta Tags Generation
//!
//! Meta tags are essential for defining metadata within HTML documents, such as page descriptions, keywords, and other
//! information relevant to search engines and web crawlers. The `macro_generate_metatags` macro allows generating
//! meta tags dynamically based on provided key-value pairs.
//!
//! # XML Element Writing
//!
//! XML elements are fundamental components of XML documents, including HTML, RSS feeds, and other structured data formats.
//! The `macro_write_element` macro facilitates writing XML elements to a specified writer, enabling the dynamic generation
//! of XML content in Rust applications.
//!
//! # Meta Tags Generation from Metadata
//!
//! In addition to generating meta tags from key-value pairs, this module provides macros for generating HTML meta tags
//! directly from metadata HashMaps or field-value pairs. These macros offer flexibility in constructing HTML meta tags
//! based on existing metadata structures or data models.
//!
//! # Example
//!
//! ```
//! use ssg::macro_generate_metatags;
//! use ssg::utilities::escape::escape_html_entities;
//!
//! let metatags = macro_generate_metatags!("description", "This is a description", "keywords", "rust,macros,metatags");
//! ```

/// Generates meta tags based on provided key-value pairs.
///
/// ## Usage
///
/// ```rust
/// use ssg::macro_generate_metatags;
/// use ssg::utilities::escape::escape_html_entities;
///
/// let metatags = macro_generate_metatags!("description", "This is a description", "keywords", "rust,macros,metatags");
/// ```
///
/// ## Arguments
///
/// * `($key:literal, $value:expr),*` - Pairs of a literal key and an expression value, each specified as `literal, expr`.
///
/// ## Behaviour
///
/// This macro generates meta tags using the provided keys and values. It takes pairs of literal keys and expression values and constructs HTML meta tags accordingly.
///
/// The pairs of keys and values are specified as `literal, expr` and separated by commas. For example, `macro_generate_metatags!("description", "This is a description", "keywords", "rust,macros,metatags")` will generate meta tags with the keys `description` and `keywords` and their corresponding values.
///
/// The macro internally creates a slice of tuples of the keys and values and passes it to the `generate_metatags` function. The function should return a string that represents the generated meta tags.
///
// #[macro_export]
// macro_rules! macro_generate_metatags {
//     ($($key:literal, $value:expr),* $(,)?) => {
//         $crate::modules::metatags::generate_metatags(&[ $(($key.to_owned(), $value.to_string())),* ])
//     };
// }
#[macro_export]
macro_rules! macro_generate_metatags {
    ($($key:literal, $value:expr),* $(,)?) => {
        $crate::modules::metatags::generate_metatags(&[ $(($key.to_owned(), escape_html_entities($value))),* ])
    };
}

/// Writes an XML element to the specified writer.
///
/// ## Usage
///
/// ```rust
/// use ssg::macro_write_element;
/// use std::io::Write;
/// use quick_xml::Writer;
///
/// let mut writer = Writer::new(Vec::new());
/// macro_write_element!(&mut writer, "title", "Hello, world!");
/// ```
///
/// ## Arguments
///
/// * `$writer:expr` - The writer instance to write the XML element to.
/// * `$name:expr` - The name of the XML element.
/// * `$value:expr` - The value of the XML element.
///
/// ## Behaviour
///
/// This macro writes an XML element with the specified name and value to the provided writer instance. It is primarily useful for generating XML documents, such as HTML or RSS feeds, dynamically.
///
#[macro_export]
macro_rules! macro_write_element {
    ($writer:expr, $name:expr, $value:expr) => {{
        use quick_xml::events::{
            BytesEnd, BytesStart, BytesText, Event,
        };
        use std::borrow::Cow;

        let result: Result<(), Box<dyn std::error::Error>> = (|| -> Result<(), Box<dyn std::error::Error>> {
            if !$value.is_empty() {
                let element_start = BytesStart::new($name);
                $writer.write_event(Event::Start(element_start.clone()))?;
                $writer.write_event(Event::Text(BytesText::from_escaped($value)))?;

                let element_end = BytesEnd::new::<Cow<'static, str>>(
                    std::str::from_utf8(element_start.name().as_ref()).unwrap().to_string().into(),
                );

                $writer.write_event(Event::End(element_end))?;
            }
            Ok(())
        })();

        result
    }};
}

/// Generates HTML meta tags based on a list of tag names and a metadata HashMap.
///
/// ## Usage
///
/// ```
/// use ssg::macro_generate_tags_from_list;
/// use ssg::modules::metatags::load_metatags;
/// use std::collections::HashMap;
///
/// // Create a new metadata hashmap
/// let mut metadata = HashMap::new();
///
/// // Insert String values into the hashmap
/// metadata.insert(String::from("description"), String::from("This is a description"));
/// metadata.insert(String::from("keywords"), String::from("rust,macros,metatags"));
///
/// // Define tag names
/// let tag_names = &["description", "keywords"];
///
/// // Call the macro with correct hashmap types
/// let html_meta_tags = macro_generate_tags_from_list!(tag_names, &metadata);
/// println!("{}", html_meta_tags);
/// ```
///
/// ## Arguments
///
/// * `$tag_names:expr` - An array slice containing the names of the tags to generate.
/// * `$metadata:expr` - The metadata HashMap containing the values for the tags.
///
/// ## Returns
///
/// Returns a string containing the HTML meta tags generated from the metadata HashMap.
///
#[macro_export]
macro_rules! macro_generate_tags_from_list {
    ($tag_names:expr, $metadata:expr) => {
        load_metatags($tag_names, $metadata)
    };
}

/// Generates HTML meta tags based on field-value pairs from a metadata HashMap.
///
/// ## Usage
///
/// ```
/// use ssg::macro_generate_tags_from_fields;
/// use ssg::modules::metatags::generate_custom_meta_tags;
/// use std::collections::HashMap;
///
/// // Create a new metadata hashmap
/// let mut metadata = HashMap::new();
///
///
/// // Insert String values into the hashmap
/// metadata.insert(String::from("description"), String::from("This is a description"));
/// metadata.insert(String::from("keywords"), String::from("rust,macros,metatags"));
///
/// // Call the macro with correct hashmap types
/// let html_meta_tags = macro_generate_tags_from_fields!(tag_names,metadata, "description" => description, "keywords" => keywords);
/// println!("{}", html_meta_tags);
/// ```
///
/// ## Arguments
///
/// * `$name:ident` - The name of the metadata HashMap.
/// * `$metadata:expr` - The metadata HashMap containing the field-value pairs.
/// * `$( $tag:literal => $field:ident ),*` - Pairs of literal tag names and metadata field names.
///
/// ## Returns
///
/// Returns a string containing the HTML meta tags generated from the metadata HashMap.
///
#[macro_export]
macro_rules! macro_generate_tags_from_fields {
    ($name:ident, $metadata:expr, $($tag:literal => $field:ident),*) => {
        {
            let tag_mapping: Vec<(String, Option<String>)> = vec![
                $(
                    ($tag.to_string(), $metadata.get(stringify!($field)).cloned()),
                )*
            ];
            generate_custom_meta_tags(&tag_mapping)
        }
    };
}

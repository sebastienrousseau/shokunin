// Copyright © 2023 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

/// # `macro_check_directory` Macro
///
/// Check if a directory exists and create it if necessary.
///
/// ## Usage
///
/// ```rust
/// use ssg::macro_check_directory;
/// use std::path::Path;
///
/// let path = Path::new("logs");
/// macro_check_directory!(path, "logs");
/// ```
///
/// ## Arguments
///
/// * `$dir` - The path to check, as a `std::path::Path`.
/// * `$name` - A string literal representing the directory name. This is used in error messages.
///
/// ## Behaviour
///
/// The `macro_check_directory` macro checks if the directory specified by `$dir` exists. If it exists and is not a directory, a panic with an error message is triggered. If the directory doesn't exist, the macro attempts to create it using `std::fs::create_dir_all($dir)`. If the creation is successful, no action is taken. If an error occurs during the directory creation, a panic is triggered with an error message indicating the failure.
///
/// Please note that the macro panics on failure. Consider using this macro in scenarios where panicking is an acceptable behavior, such as during application startup or setup.
///
#[macro_export]
macro_rules! macro_check_directory {
    ($dir:expr, $name:expr) => {{
        let directory: &std::path::Path = $dir;
        let name = $name;
        if directory.exists() {
            if !directory.is_dir() {
                panic!("❌ Error: '{}' is not a directory.", name);
            }
        } else {
            match std::fs::create_dir_all(directory) {
                Ok(_) => {}
                Err(e) => panic!(
                    "❌ Error: Cannot create '{}' directory: {}",
                    name, e
                ),
            }
        }
    }};
}

/// # `macro_cleanup_directories` Macro
///
/// Cleanup multiple directories by invoking the `cleanup_directory` function.
///
/// ## Usage
///
/// ```rust
/// use std::path::Path;
/// use ssg::macro_check_directory;
///
/// let path = Path::new("logs");
/// macro_check_directory!(path, "logs");
/// ```
///
/// ## Arguments
///
/// * `$( $dir:expr ),*` - A comma-separated list of directory paths to clean up.
///
/// ## Behaviour
///
/// The `macro_cleanup_directories` macro takes multiple directory paths as arguments and invokes the `cleanup_directory` function for each path. It is assumed that the `cleanup_directory` function is available in the crate's utilities module (`$crate::utilities::cleanup_directory`).
///
/// The macro creates an array `directories` containing the provided directory paths and passes it as an argument to `cleanup_directory`. The `cleanup_directory` function is responsible for performing the cleanup operations.
///
/// Please note that the macro uses the `?` operator for error propagation. It expects the `cleanup_directory` function to return a `Result` type. If an error occurs during the cleanup process, it will be propagated up the call stack, allowing the caller to handle it appropriately.
///
#[macro_export]
macro_rules! macro_cleanup_directories {
    ( $($dir:expr),* ) => {
        {
            use $crate::utilities::cleanup_directory;
            let directories: &[&Path] = &[ $($dir),* ];
            match cleanup_directory(directories) {
                Ok(()) => (),
                Err(err) => panic!("Cleanup failed: {:?}", err),
            }
        }
    };
}

/// # `macro_create_directories` Macro
///
/// Create multiple directories at once.
///
/// ## Usage
///
/// ```rust
/// use ssg::macro_create_directories;
/// use std::path::Path;
/// macro_create_directories!("logs", "cache", "data");
/// assert!(Path::new("logs").exists());
/// assert!(Path::new("cache").exists());
/// assert!(Path::new("data").exists());
/// std::fs::remove_dir("logs");
/// std::fs::remove_dir("cache");
/// std::fs::remove_dir("data");
/// ```
///
/// ## Arguments
///
/// * `...` - Variable number of directory paths, each specified as an expression (`expr`).
///
/// ## Behaviour
///
/// The `macro_create_directories` macro creates multiple directories at once. It takes a variable number of directory paths as arguments and uses the `create_directory` utility function from the `$crate` crate to create the directories.
///
/// The directories are specified as expressions and separated by commas. For example, `macro_create_directories!("logs", "cache", "data")` will attempt to create the `logs`, `cache`, and `data` directories.
///
/// The macro internally creates a slice of the directory paths and passes it to the `create_directory` function. If any error occurs during the directory creation, the macro returns an `Err` value, indicating the first encountered error. Otherwise, it returns `Ok(())`.
///
/// ## Example
///
/// ```rust
/// use ssg::macro_create_directories;
/// use std::path::Path;
///
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let test = Path::new("test");
///     let test2  = Path::new("test2");
///     macro_create_directories!(test, test2)?;
///     assert!(test.exists());
///     assert!(test2.exists());
///     std::fs::remove_dir(test)?;
///     std::fs::remove_dir(test2)?;
///     Ok(())
/// }
/// ```
///
#[macro_export]
macro_rules! macro_create_directories {
    ( $($dir:expr),* ) => {{
        use $crate::utilities::create_directory;
        use std::path::Path;
        let directories: Vec<&Path> = vec![ $(Path::new($dir)),* ];
        create_directory(&directories)
    }};
}

/// # `macro_generate_metatags` Macro
///
/// Generate a sequence of metatags using the provided keys and values.
///
/// ## Usage
///
/// ```rust
/// use ssg::macro_generate_metatags;
/// macro_generate_metatags!("description", "This is a description", "keywords", "rust,macros,metatags");
/// ```
///
/// ## Arguments
///
/// * `($key:literal, $value:expr)...` - Pairs of a literal key and an expression value, each specified as `literal, expr`. The pairs should be separated by commas.
///
/// ## Behaviour
///
/// The `macro_generate_metatags` macro generates metatags using the provided keys and values. It takes pairs of literal keys and expression values and uses the `generate_metatags` function to create the metatags.
///
/// The pairs of keys and values are specified as `literal, expr` and separated by commas. For example, `macro_generate_metatags!("description", "This is a description", "keywords", "rust,macros,metatags")` will generate metatags with the keys `description` and `keywords` and the corresponding values.
///
/// The macro internally creates a slice of tuples of the keys and values and passes it to the `generate_metatags` function. The function should return a string that represents the generated metatags.
///
/// ## Example
///
/// ```rust
/// use ssg::macro_generate_metatags;
/// let description = "This is a test description.";
/// let keywords = "test,rust,macro";
/// let metatags = macro_generate_metatags!("description", description, "keywords", keywords);
/// println!("{}", metatags);
/// ```
#[macro_export]
macro_rules! macro_generate_metatags {
    ($($key:literal, $value:expr),* $(,)?) => {
        $crate::modules::metatags::generate_metatags(&[ $(($key.to_owned(), $value.to_string())),* ])
    };
}

/// # `macro_get_args` Macro
///
/// Retrieve a named argument from a `clap::ArgMatches` object.
///
/// ## Usage
///
/// ```rust
/// use clap::{Arg, ArgMatches, Command, Error};
/// use ssg::macro_get_args;
///
/// fn test() -> Result<(), Box<dyn std::error::Error>> {
///     let matches = Command::new("test_app")
///         .arg(
///             Arg::new("content")
///                 .long("content")
///                 .short('c')
///                 .value_name("CONTENT"),
///         )
///         .get_matches_from(vec!["test_app", "--content", "test_content"]);
///
///     let content = macro_get_args!(matches, "content");
///     println!("Content: {}", content);
///     Ok(())
/// }
/// test();
/// ```
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
/// use ssg::macro_metadata_option;
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
/// ## Example
///
/// ```rust
/// use ssg::macro_metadata_option;
/// use std::collections::HashMap;
///
/// let mut metadata = HashMap::new();
/// metadata.insert("key", "value");
/// let value = macro_metadata_option!(metadata, "key");
/// println!("{}", value);
/// ```
///
#[macro_export]
macro_rules! macro_metadata_option {
    ($metadata:ident, $key:expr) => {
        $metadata.get($key).cloned().unwrap_or_default()
    };
}

/// # `macro_render_layout` Macro
///
/// Selects the appropriate template for rendering based on the specified layout,
/// and uses this template to render a context into a string.
///
#[macro_export]
macro_rules! macro_render_layout {
    ($layout:expr, $template_path:expr, $context:expr) => {{
        let layout_str: &str = &$layout;

        let template_file = match layout_str {
            "contact" => "contact.html",
            "index" => "index.html",
            "page" => "page.html",
            _ => "index.html",
        };

        let template_content = fs::read_to_string(
            Path::new($template_path).join(template_file),
        )
        .unwrap();
        render_template(&template_content, &$context)
    }};
}

/// # `macro_serve` Macro
///
/// Start a server at the specified address with the given document root.
///
/// ## Arguments
///
/// * `$server_address` - The address at which the server should listen, specified as an expression (`expr`).
/// * `$document_root` - The root directory of the documents that the server should serve, specified as an expression (`expr`).
///
/// ## Behaviour
///
/// The `macro_serve` macro starts a server at the specified address with the given document root. It internally calls the `start` function from an unspecified library, passing the server address and document root as arguments.
///
/// If the server starts successfully, the macro returns `Ok(())`. If an error occurs during server startup, the macro will panic with the error message provided by the `unwrap` method.
///
#[macro_export]
macro_rules! macro_serve {
    ($server_address:expr, $document_root:expr) => {
        start($server_address, $document_root).unwrap();
    };
}

#[macro_export]
/// # `write_element` macro
///
/// Writes an XML element to the specified writer.
///
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

#[macro_export]
/// # `macro_generate_tags_from_list` macro
///
/// Generates HTML meta tags based on a list of tag names and a metadata HashMap.
///
macro_rules! macro_generate_tags_from_list {
    ($tag_names:expr, $metadata:expr) => {
        load_metatags($tag_names, $metadata)
    };
}

#[macro_export]
/// # `macro_generate_tags_from_fields` macro
///
/// Generates HTML meta tags based on a list of tag names and a metadata HashMap.
///
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

#[macro_export]
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
/// Returns `Result<(), Box<dyn Error>>` indicating success or an error if XML writing fails.
///
macro_rules! macro_generate_rss {
    ($writer:expr, $options:expr) => {
        writer.write_event(Event::Decl(BytesDecl::new("1.0", Some("utf-8"), None)))?;

        let mut rss_start = BytesStart::new("rss");
        rss_start.push_attribute(("version", "2.0"));
        rss_start.push_attribute(("xmlns:atom", "http://www.w3.org/2005/Atom"));
        writer.write_event(Event::Start(rss_start))?;

        writer.write_event(Event::Start(BytesStart::new("channel")))?;

        macro_write_element!($writer, "title", &$options.title)?;
        macro_write_element!($writer, "link", &$options.link)?;
        macro_write_element!($writer, "description", &$options.description)?;
        macro_write_element!($writer, "language", &$options.language)?;
        macro_write_element!($writer, "pubDate", &$options.pub_date)?;
        macro_write_element!(
            $writer,
            "lastBuildDate",
            &$options.last_build_date
        )?;
        macro_write_element!($writer, "docs", &$options.docs)?;
        macro_write_element!($writer, "generator", &$options.generator)?;
        macro_write_element!(
            $writer,
            "managingEditor",
            &$options.managing_editor
        )?;
        macro_write_element!($writer, "webMaster", &$options.webmaster)?;
        macro_write_element!($writer, "category", &$options.category)?;
        macro_write_element!($writer, "ttl", &$options.ttl)?;

        // Write the `image` element.
        writer.write_event(Event::Start(BytesStart::new("image")))?;
        macro_write_element!($writer, "url", &$options.image)?;
        macro_write_element!($writer, "title", &$options.title)?;
        macro_write_element!($writer, "link", &$options.link)?;
        writer.write_event(Event::End(BytesEnd::new("image")))?;

        // Write the `atom:link` element.
        let mut atom_link_start =
            BytesStart::new(Cow::Borrowed("atom:link").into_owned());
        atom_link_start.push_attribute((
            "href",
            $options.atom_link.to_string().as_str(),
        ));
        atom_link_start.push_attribute(("rel", "self"));
        atom_link_start.push_attribute(("type", "application/rss+xml"));
        writer.write_event(Event::Empty(atom_link_start))?;

        // Write the `item` element.
        writer.write_event(Event::Start(BytesStart::new("item")))?;

        macro_write_element!($writer, "author", &$options.author)?;
        macro_write_element!(
            $writer,
            "description",
            &$options.item_description
        )?;
        macro_write_element!($writer, "guid", &$options.item_guid)?;
        macro_write_element!($writer, "link", &$options.item_link)?;
        macro_write_element!($writer, "pubDate", &$options.item_pub_date)?;
        macro_write_element!($writer, "title", &$options.item_title)?;

        // End the `item` element.
        writer.write_event(Event::End(BytesEnd::new("item")))?;

        // End the `channel` element.
        writer.write_event(Event::End(BytesEnd::new("channel")))?;

        // End the `rss` element.
        writer.write_event(Event::End(BytesEnd::new("rss")))?;

        Ok(())
    };
}
#[macro_export]
/// # `macro_set_rss_data_fields` macro
macro_rules! macro_set_rss_data_fields {
    ($rss_data:expr, $field:ident, $value:expr) => {
        $rss_data.set(stringify!($field), $value);
    };
}
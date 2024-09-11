// Copyright © 2024 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! This module provides several macros for common tasks such as retrieving arguments, extracting options from metadata,
//! rendering layouts, and starting a server.
//!
//! # `macro_get_args` Macro
//!
//! Retrieve a named argument from a `clap::ArgMatches` object.
//!
//! ## Usage
//!
//! ```rust
//! use clap::{Arg, ArgMatches, Command, Error};
//! use ssg_core::macro_get_args;
//!
//! fn test() -> Result<(), Box<dyn std::error::Error>> {
//!     let matches = Command::new("test_app")
//!         .arg(
//!             Arg::new("content")
//!                 .long("content")
//!                 .short('c')
//!                 .value_name("CONTENT"),
//!         )
//!         .get_matches_from(vec!["test_app", "--content", "test_content"]);
//!
//!     let content = macro_get_args!(matches, "content");
//!     println!("Content: {}", content);
//!     Ok(())
//! }
//! test();
//! ```
//!
//! ## Arguments
//!
//! * `$matches` - A `clap::ArgMatches` object representing the parsed command-line arguments.
//! * `$name` - A string literal specifying the name of the argument to retrieve.
//!
//! ## Behaviour
//!
//! The `macro_get_args` macro retrieves the value of the named argument `$name` from the `$matches` object. If the argument is found and its value can be converted to `String`, the macro returns the value as a `Result<String, String>`. If the argument is not found or its value cannot be converted to `String`, an `Err` variant is returned with an error message indicating the omission of the required parameter.
//!
//! The error message includes the name of the omitted parameter (`$name`) to assist with troubleshooting and providing meaningful feedback to users.
//!
//! # `macro_metadata_option` Macro
//!
//! Extracts an option value from metadata.
//!
//! ## Usage
//!
//! ```rust
//! use std::collections::HashMap;
//! use ssg_core::macro_metadata_option;
//!
//! let mut metadata = HashMap::new();
//! metadata.insert("key", "value");
//! let value = macro_metadata_option!(metadata, "key");
//! println!("{}", value);
//! ```
//!
//! ## Arguments
//!
//! * `$metadata` - A mutable variable that represents the metadata (of type `HashMap<String, String>` or any other type that supports the `get` and `cloned` methods).
//! * `$key` - A string literal that represents the key to search for in the metadata.
//!
//! ## Behaviour
//!
//! The `macro_metadata_option` macro is used to extract an option value from metadata. It takes a mutable variable representing the metadata and a string literal representing the key as arguments, and uses the `get` method of the metadata to find an option value with the specified key.
//!
//! If the key exists in the metadata, the macro clones the value and returns it. If the key does not exist, it returns the default value for the type of the metadata values.
//!
//! The macro is typically used in contexts where metadata is stored in a data structure that supports the `get` and `cloned` methods, such as a `HashMap<String, String>`.
//!
//! # `macro_render_layout` Macro
//!
//! This macro selects and renders a specified layout with a given context.
//!
//! * `$layout`: The desired layout name (e.g., "contact", "index", "page").
//!   If a template file corresponding to `$layout.html` exists in the template directory,
//!   it will be used. Otherwise, the macro falls back to predefined mappings.
//!
//! * `$template_path`: The path to the directory containing the template files.
//!
//! * `$context`: The context to be rendered into the template.
//!
//! ## Behaviour:
//!
//! 1. If a file named `$layout.html` exists in `$template_path`, it will be used as the template.
//! 2. If no such file exists, the macro checks for predefined layout names:
//!     * "contact" maps to "contact.html"
//!     * "index" maps to "index.html"
//!     * "page" maps to "page.html"
//! 3. If `$layout` is unrecognized and doesn't correspond to a file in the template directory,
//!    the macro defaults to using "index.html".
//!
//! ## Returns:
//!
//! The macro returns a rendered string using the selected template and provided context.
//!
//! # `macro_serve` Macro
//!
//! Start a server at the specified address with the given document root.
//!
//! ## Arguments
//!
//! * `$server_address` - The address at which the server should listen, specified as an expression (`expr`).
//! * `$document_root` - The root directory of the documents that the server should serve, specified as an expression (`expr`).
//!
//! ## Behaviour
//!
//! The `macro_serve` macro starts a server at the specified address with the given document root. It internally calls the `start` function from an unspecified library, passing the server address and document root as arguments.
//!
//! If the server starts successfully, the macro returns `Ok(())`. If an error occurs during server startup, the macro will panic with the error message provided by the `unwrap` method.
//!

/// # `macro_get_args` Macro
///
/// Retrieve a named argument from a `clap::ArgMatches` object.
///
/// ## Usage
///
/// ```rust
/// use clap::{Arg, ArgMatches, Command, Error};
/// use ssg_core::macro_get_args;
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
/// use ssg_core::macro_metadata_option;
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
/// use ssg_core::macro_metadata_option;
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
/// This macro selects and renders a specified layout with a given context.
///
/// * `$layout`: The desired layout name (e.g., "contact", "index", "page").
///   If a template file corresponding to `$layout.html` exists in the template directory,
///   it will be used. Otherwise, the macro falls back to predefined mappings.
///
/// * `$template_path`: The path to the directory containing the template files.
///
/// * `$context`: The context to be rendered into the template.
///
/// ## Behaviour:
///
/// 1. If a file named `$layout.html` exists in `$template_path`, it will be used as the template.
/// 2. If no such file exists, the macro checks for predefined layout names:
///     * "contact" maps to "contact.html"
///     * "index" maps to "index.html"
///     * "page" maps to "page.html"
/// 3. If `$layout` is unrecognized and doesn't correspond to a file in the template directory,
///    the macro defaults to using "index.html".
///
/// ## Returns:
///
/// The macro returns a rendered string using the selected template and provided context.
///
#[macro_export]
macro_rules! macro_render_layout {
    ($layout:expr, $template_path:expr, $context:expr) => {{
        let layout_str: &str = &$layout;

        // Check if a file with the name of the layout exists in the template directory
        let file_path = Path::new($template_path).join(format!("{}.html", layout_str));
        let template_file = if file_path.exists() {
            format!("{}.html", layout_str)
        } else {
            match layout_str {
                "contact" => "contact.html",
                "index" => "index.html",
                "page" => "page.html",
                "post" => "post.html",
                _ => "index.html",
            }.to_string()
        };

        let template_content = fs::read_to_string(
            Path::new($template_path).join(&template_file),
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

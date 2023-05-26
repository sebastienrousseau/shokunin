// Copyright © 2023 Shokunin (職人) Static Site Generator. All rights reserved.
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
/// fn main() {
///     let path = Path::new("logs");
///     macro_check_directory!(path, "logs");
/// }
/// ```
///
/// ## Arguments
///
/// * `$dir` - The path to check, as a `std::path::Path`.
/// * `$name` - A string literal representing the directory name. This is used in error messages.
///
/// ## Behavior
///
/// The `macro_check_directory` macro checks if the directory specified by `$dir` exists. If it exists and is not a directory, a panic with an error message is triggered. If the directory doesn't exist, the macro attempts to create it using `std::fs::create_dir_all($dir)`. If the creation is successful, no action is taken. If an error occurs during the directory creation, a panic is triggered with an error message indicating the failure.
///
/// Please note that the macro panics on failure. Consider using this macro in scenarios where panicking is an acceptable behavior, such as during application startup or setup.
///
#[macro_export]
macro_rules! macro_check_directory {
    ($dir:expr, $name:expr) => {
        if $dir.exists() {
            if !$dir.is_dir() {
                panic!("❌ Error: {} is not a directory.", $name);
            }
        } else {
            match std::fs::create_dir_all($dir) {
                Ok(_) => {}
                Err(e) => panic!("❌ Error: Cannot create {} directory: {}", $name, e),
            }
        }
    };
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
/// fn main() {
///     let path = Path::new("logs");
///     macro_check_directory!(path, "logs");
/// }
/// ```
///
/// ## Arguments
///
/// * `$( $dir:expr ),*` - A comma-separated list of directory paths to clean up.
///
/// ## Behavior
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
            let directories = &[ $($dir),* ];
            cleanup_directory(directories)?;
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
///
/// fn main() {
///     macro_create_directories!("logs", "cache", "data");
/// }
/// ```
///
/// ## Arguments
///
/// * `...` - Variable number of directory paths, each specified as an expression (`expr`).
///
/// ## Behavior
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


#[macro_export]
/// ## Macro: `macro_generate_metatags` - Generates HTML meta tags from a list of key-value pairs.
///
/// This macro takes a list of key-value pairs and generates code that creates HTML meta tags.
///
/// ### Arguments
///
/// * `$key`   - The key for the meta tag.
/// * `$value` - The value for the meta tag.
///
macro_rules! macro_generate_metatags {
    ($($key:literal, $value:expr),* $(,)?) => {
        generate_metatags(&[ $(($key.to_owned(), $value.to_string())),* ])
    };
}

#[macro_export]
/// ## Macro: `macro_get_args` - Retrieve a command-line argument or return an error message.
///
/// This macro takes a `clap::ArgMatches` object and a string literal and generates code that retrieves the argument or returns an error message if it does not exist.
///
/// ### Arguments
///
/// * `$matches` - A `clap::ArgMatches` object.
/// * `$name`    - A string literal for the error message.
///
macro_rules! macro_get_args {
    ($matches:ident, $name:expr) => {
        $matches.get_one::<String>($name).ok_or(format!(
            "❌ Error: A required parameter was omitted. Add the required parameter. \"{}\".",
            $name
        ))?
    };
}

#[macro_export]
/// ## Macro: `macro_metadata_option` - Retrieve a metadata option or return an empty string.
///
/// This macro takes a `HashMap` object and a string literal and generates code that retrieves the option or returns an empty string if it does not exist.
///
/// ### Arguments
///
/// * `$metadata` - A `HashMap` object.
/// * `$key`      - A string literal for the error message.
///
macro_rules! macro_metadata_option {
    ($metadata:ident, $key:expr) => {
        $metadata.get($key).cloned().unwrap_or_default()
    };
}

#[macro_export]
/// ## Macro: `macro_render_layout` - Render a layout template.
///
/// This macro takes a layout, a template path, and a context and generates code that renders the layout template.
///
/// ### Arguments
///
/// * `$layout`        - The layout to render.
/// * `$template_path` - The path to the template.
/// * `$context`       - The context to render.
///
macro_rules! macro_render_layout {
    ($layout:expr, $template_path:expr, $context:expr) => {{
        let layout_str: &str = &$layout;

        let template_file = match layout_str {
            "archive" => "archive.html",
            "category" => "category.html",
            "homepage" => "homepage.html",
            "index" => "index.html",
            "page" => "page.html",
            "post" => "post.html",
            "rss" => "rss.xml",
            "section" => "section.html",
            "sitemap" => "sitemap.xml",
            "tag" => "tag.html",
            _ => "template.html",
        };

        let template_content =
            fs::read_to_string(Path::new($template_path).join(template_file)).unwrap();
        render_template(&template_content, &$context)
    }};
}

#[macro_export]
/// ## Macro: `macro_serve` - Start a web server to serve the public directory.
///
/// This macro takes a server address and a document root and generates code that creates a TCP listener listening at the server address.
///
/// It then generates code that iterates over the incoming connections on the listener, and handles each connection by passing it to the `handle_connection` function.
///
/// ### Arguments
///
/// * `server_address` - A string literal for the server address.
/// * `document_root`  - A string literal for the document root.
///
macro_rules! macro_serve {
    ($server_address:expr, $document_root:expr) => {
        start($server_address, $document_root).unwrap();
    };
}

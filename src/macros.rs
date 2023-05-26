// Copyright © 2023 Shokunin (職人) Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

#[macro_export]
/// ## Macro: `macro_check_directory` - Check if a directory exists or return
/// an error message if it does not
macro_rules! macro_check_directory {
    ($path:expr, $arg:expr) => {
        if let Err(e) = directory($path, $arg) {
            return Err(format!("❌ Error: {}", e));
        }
    };
}

#[macro_export]
/// ## Macro: `macro_cleanup_directories` - Delete a list of directories or
/// return an error message if it does not succeed.
macro_rules! macro_cleanup_directories {
    ( $($dir:expr),* ) => {
        {
            use $crate::utilities::cleanup_directory;
            let directories = &[ $($dir),* ];
            cleanup_directory(directories)?;
        }
    };
}

#[macro_export]
/// ## Macro: `macro_create_directories` - Create a list of directories or
/// return an error message if it does not succeed.
macro_rules! macro_create_directories {
    ( $($dir:expr),* ) => {
        {
            use $crate::utilities::create_directory;
            let directories = &[ $($dir),* ];
            create_directory(directories)?;
        }
    };
}

#[macro_export]
/// ## Macro: `macro_generate_metatags` - Generates HTML meta tags from a list of
/// key-value pairs
macro_rules! macro_generate_metatags {
    ($($key:literal, $value:expr),* $(,)?) => {
        generate_metatags(&[ $(($key.to_owned(), $value.to_string())),* ])
    };
}

#[macro_export]
/// ## Macro: `macro_get_args` - Retrieve a command-line argument or return an
/// error message
macro_rules! macro_get_args {
    ($matches:ident, $name:expr) => {
        $matches.get_one::<String>($name).ok_or(format!(
            "❌ Error: A required parameter was omitted. Add the required parameter. \"{}\".",
            $name
        ))?
    };
}

#[macro_export]
/// ## Macro: `macro_metadata_option` - Retrieve a metadata option or return an
/// empty string
macro_rules! macro_metadata_option {
    ($metadata:ident, $key:expr) => {
        $metadata.get($key).cloned().unwrap_or_default()
    };
}

#[macro_export]
/// ## Macro: `macro_render_layout` - Render a layout template
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
/// This macro takes a server address and a document root and generates code
/// that creates a TCP listener listening at the server address.
///
/// It then generates code that iterates over the incoming connections on the
/// listener, and handles each connection by passing it to the
/// `handle_connection` function.
///
/// # Arguments
///
/// * `server_address` - A string literal for the server address.
/// * `document_root`  - A string literal for the document root.
///
macro_rules! macro_serve {
    ($server_address:expr, $document_root:expr) => {
        start($server_address, $document_root).unwrap();
    };
}

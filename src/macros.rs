// Copyright © 2023 Hash (HSH) library. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

#[macro_export]
/// ## Macro: `macro_metadata_option` - Retrieve a metadata option or return an
/// empty string
macro_rules! macro_metadata_option {
    ($metadata:ident, $key:expr) => {
        $metadata
            .get($key)
            .cloned()
            .unwrap_or_else(|| "".to_string())
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
/// ## Macro: `macro_check_directory` - Check if a directory exists or return
/// an error message if it does not
macro_rules! macro_check_directory {
    ($path:expr, $arg:expr) => {
        if let Err(e) = directory($path, $arg) {
            return Err(format!("❌ Error: {}", e));
        }
    };
}

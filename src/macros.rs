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

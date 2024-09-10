// Copyright © 2024 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Module for English translations.

/// Translates the given text into English.
///
/// This function looks up the translation for the given `text` in the `T` hash map.
/// If a translation is found, it returns the translated string. Otherwise, it returns
/// the original `text` as a fallback.
///
/// # Arguments
///
/// * `text` - The text to be translated.
///
/// # Returns
///
/// The translated string if a translation is found, or the original `text` if no
/// translation is available.
///
/// # Examples
///
/// ```
/// use ssg::languages::translate;
///
/// let text = "Hello";
/// let translated = translate("en", "Hello");
/// assert_eq!(translated, "Hello");
/// ```
pub(crate) fn translate(text: &str) -> String {
    T.get(text)
        .map(|s| s.to_string())
        .unwrap_or_else(|| text.to_string())
}

lazy_static::lazy_static! {
    /// Hash map containing English translations.
    ///
    /// This static hash map is lazily initialized using the `lazy_static` macro.
    /// It contains key-value pairs, where the key is the original text and the value
    /// is the corresponding English translation.
    ///
    /// The translations are defined as an array of tuples `(&str, &str)` and collected
    /// into the hash map using `iter().cloned().collect()`.
    ///
    pub static ref T: std::collections::HashMap<&'static str, &'static str> =
        [
            ("Hello", "Hello"),
            ("main_logger_msg", "\nPlease run `ssg --help` for more information.\n"),
            ("lib_banner_log_msg", "Banner printed successfully"),
            ("lib_args_log_msg", "Arguments processed successfully"),
            ("lib_server_log_msg", "Server started successfully"),
            // Add more translations here
        ].iter().cloned().collect();
}

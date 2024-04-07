// Copyright © 2024 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Module for German translations.

/// Translates the given text into German.
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
/// let translated = translate("de", "Hello");
/// assert_eq!(translated, "Hallo");
/// ```
pub(crate) fn translate(text: &str) -> String {
    T.get(text).map(|s| s.to_string()).unwrap_or_else(|| text.to_string())
}

lazy_static::lazy_static! {
    /// Hash map containing German translations.
    ///
    /// This static hash map is lazily initialized using the `lazy_static` macro.
    /// It contains key-value pairs, where the key is the original text and the value
    /// is the corresponding German translation.
    ///
    /// The translations are defined as an array of tuples `(&str, &str)` and collected
    /// into the hash map using `iter().cloned().collect()`.
    ///
    pub static ref T: std::collections::HashMap<&'static str, &'static str> =
        [
            ("Hello", "Hallo"),
            ("main_logger_msg", "\nFür weitere Informationen führen Sie bitte `ssg --help` aus.\n")
            // Add more translations here
        ].iter().cloned().collect();
}
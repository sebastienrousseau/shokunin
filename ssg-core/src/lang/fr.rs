// Copyright © 2024 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Module for French translations.

/// Translates the given text into French.
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
/// let translated = translate("fr", "Hello");
/// assert_eq!(translated, "Bonjour");
/// ```
pub(crate) fn translate(text: &str) -> String {
    T.get(text)
        .map(|s| s.to_string())
        .unwrap_or_else(|| text.to_string())
}

lazy_static::lazy_static! {
    /// Hash map containing French translations.
    ///
    /// This static hash map is lazily initialized using the `lazy_static` macro.
    /// It contains key-value pairs, where the key is the original text and the value
    /// is the corresponding French translation.
    ///
    /// The translations are defined as an array of tuples `(&str, &str)` and collected
    /// into the hash map using `iter().cloned().collect()`.
    ///
    pub static ref T: std::collections::HashMap<&'static str, &'static str> =
        [
            ("Hello", "Bonjour"),
            ("main_logger_msg", "\nVeuillez lancer `ssg --help` pour plus d'informations.\n"),
            ("lib_banner_log_msg", "Bannière imprimée avec succès"),
            ("lib_args_log_msg", "Arguments traités avec succès"),
            ("lib_server_log_msg", "Serveur démarré avec succès"),
            // Add more translations here
        ].iter().cloned().collect();
}

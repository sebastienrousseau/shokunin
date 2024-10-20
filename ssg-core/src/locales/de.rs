// Copyright © 2024 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Module for German translations.

use lazy_static::lazy_static;
use std::collections::HashMap;

use langweave::error::I18nError;

lazy_static! {
    static ref TRANSLATIONS: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
        m.insert("Hello", "Hallo");
        m.insert("Goodbye", "Auf Wiedersehen");
        m.insert("main_logger_msg", "\nFür weitere Informationen führen Sie bitte `ssg --help` aus.\n");
        m.insert("lib_banner_log_msg", "Banner erfolgreich gedruckt");
        m.insert("lib_args_log_msg", "Argumente erfolgreich verarbeitet");
        m.insert("lib_server_log_msg", "Server erfolgreich gestartet");
        // Add more translations here as needed
        m
    };
}

/// Translates the given text into German.
///
/// This function looks up the translation for the given `text` in the `TRANSLATIONS` hash map.
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
pub fn translate(key: &str) -> Result<String, I18nError> {
    if let Some(&translation) = TRANSLATIONS.get(key) {
        Ok(translation.to_string())
    } else {
        Err(I18nError::TranslationFailed(key.to_string()))
    }
}

// Copyright © 2024 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Module for French translations.

use lazy_static::lazy_static;
use std::collections::HashMap;

use crate::I18nError;

lazy_static! {
    static ref TRANSLATIONS: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
        m.insert("Hello", "Bonjour");
        m.insert("Goodbye", "Au revoir");
        m.insert("main_logger_msg", "\nVeuillez lancer `ssg --help` pour plus d'informations.\n");
        m.insert("lib_banner_log_msg", "Bannière imprimée avec succès");
        m.insert("lib_args_log_msg", "Arguments traités avec succès");
        m.insert("lib_server_log_msg", "Serveur démarré avec succès");
        // Add more translations here as needed
        m
    };
}

/// Translates the given text into French.
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
/// # Examples
///
/// ```
/// use ssg_i18n::languages::fr::translate;
///
/// assert_eq!(translate("Hello").unwrap(), "Bonjour");
/// assert_eq!(translate("Untranslated").unwrap_or("Untranslated".to_string()), "Untranslated");
/// ```
pub fn translate(key: &str) -> Result<String, I18nError> {
    if let Some(&translation) = TRANSLATIONS.get(key) {
        Ok(translation.to_string())
    } else {
        Err(I18nError::TranslationFailed(key.to_string()))
    }
}

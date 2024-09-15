// Copyright © 2024 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

use ssg_i18n::lang::de;
use ssg_i18n::lang::en;
use ssg_i18n::lang::fr;

/// A vector containing all supported languages and their names.
///
/// # Returns
///
/// A vector of tuples, where each tuple contains a language code and its corresponding name.
pub const LANGS: &[(&str, &str)] =
    &[("en", "English"), ("fr", "French"), ("de", "German")];

/// Translates the given text into the specified language.
///
/// # Arguments
///
/// * `lang` - The language code to translate to.
/// * `text` - The text to be translated.
///
/// # Returns
///
/// A string containing the translated text. If the specified language is not supported, the original text is returned.
///
pub fn translate(lang: &str, text: &str) -> String {
    match lang {
        "en" => en::translate(text),
        "fr" => fr::translate(text),
        "de" => de::translate(text),
        _ => text.to_string(),
    }
}

/// Returns a vector containing all supported languages and their names.
///
/// # Returns
///
/// A vector of tuples, where each tuple contains a language code and its corresponding name.
///
pub fn get_langs() -> Vec<(&'static str, &'static str)> {
    LANGS.to_vec()
}

/// Returns the name of the language corresponding to the given code.
///
/// # Arguments
///
/// * `lang` - The language code to search for.
///
/// # Returns
///
/// An `Option<String>` containing the name of the language if found, otherwise `None`.
///
pub fn get_lang(lang: &str) -> Option<String> {
    LANGS
        .iter()
        .find(|(code, _)| code == &lang)
        .map(|(_, name)| name.to_string())
}

/// Returns the language code corresponding to the given language name.
///
/// # Arguments
///
/// * `lang` - The language name to search for.
///
/// # Returns
///
/// An `Option<&str>` containing the language code if found, otherwise `None`.
///
pub fn get_lang_code(lang: &str) -> Option<&str> {
    LANGS
        .iter()
        .find(|(_, name)| name == &lang)
        .map(|(code, _)| code)
        .copied()
}

/// Get language from accept header
///
/// # Arguments
///
/// * `accept` - The accept header value
///
/// # Returns
///
/// * `Option<String>` - The language code if found, otherwise None
///
pub fn get_lang_from_accept(accept: &str) -> Option<String> {
    let mut langs: Vec<_> = accept
        .split(',')
        .filter_map(|s| {
            let mut parts = s.trim().split(';');
            let lang = parts.next()?.split('-').next()?;
            let quality = parts
                .next()?
                .trim_start_matches("q=")
                .parse::<f32>()
                .ok()?;
            Some((lang.to_string(), quality))
        })
        .collect();

    langs.sort_by(|(_, q1), (_, q2)| {
        q2.partial_cmp(q1).unwrap_or(std::cmp::Ordering::Equal)
    });

    let supported_langs = get_langs();

    for (lang_code, _) in langs {
        if let Some(lang_name) =
            supported_langs.iter().find_map(|(code, name)| {
                if code == &lang_code {
                    Some(name.to_string())
                } else {
                    None
                }
            })
        {
            return Some(lang_name);
        }
    }

    None
}

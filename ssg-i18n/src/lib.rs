//! # SSG I18n (Internationalization)
//!
//! `ssg-i18n` is a library for internationalization and localization support,
//! designed specifically for use with the Shokunin Static Site Generator (SSG).
//!
//! This library provides the following functionality:
//! - Translating text into different languages
//! - Detecting the language of text content
//! - Managing multiple language translations
//! - Integration with the static site generation process

/// The `error` module contains error types used by the library.
pub mod error;
/// The `language_detector` module contains a simple regex-based language detector.
pub mod language_detector;
/// The `languages` module contains translations for different languages.
pub mod languages;
/// The `translator` module contains a simple translation service using a predefined dictionary.
pub mod translator;

pub use error::I18nError;
pub use language_detector::LanguageDetector;
pub use translator::Translator;

/// A convenience function to translate a given text to a specified language.
///
/// # Arguments
///
/// * `lang` - A string slice that holds the language code (e.g., "en" for English, "fr" for French)
/// * `text` - A string slice that holds the text to be translated
///
/// # Returns
///
/// * `Result<String, I18nError>` - The translated string if successful, or an error if translation fails
///
/// # Examples
///
/// ```
/// use ssg_i18n::translate;
///
/// let result = translate("fr", "Hello");
/// assert_eq!(result.unwrap(), "Bonjour");
/// ```
pub fn translate(lang: &str, text: &str) -> Result<String, I18nError> {
    let translator = Translator::new(lang)?;
    translator.translate(text)
}

/// Detects the language of a given text using simple regex-based heuristics.
///
/// # Arguments
///
/// * `text` - A string slice that holds the text to analyze
///
/// # Returns
///
/// * `Result<String, I18nError>` - The detected language code if successful, or an error if detection fails
///
/// # Examples
///
/// ```
/// use ssg_i18n::detect_language;
///
/// let result = detect_language("The quick brown fox jumps over the lazy dog.");
/// assert_eq!(result.unwrap(), "en");
/// ```
pub fn detect_language(text: &str) -> Result<String, I18nError> {
    let detector = LanguageDetector::new();
    detector.detect(text)
}

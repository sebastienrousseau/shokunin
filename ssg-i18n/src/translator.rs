//! # Translator Module
//!
//! This module provides functionality to translate text into different languages.

use crate::error::I18nError;
use crate::languages::{de, en, fr};

/// A struct responsible for translating text into different languages.
pub struct Translator {
    lang: String,
    translate_fn: fn(&str) -> Result<String, I18nError>,
}

impl Translator {
    /// Creates a new `Translator` instance for a specific language.
    ///
    /// # Arguments
    ///
    /// * `lang` - A string slice that holds the language code (e.g., "en", "fr", "de")
    ///
    /// # Returns
    ///
    /// * `Result<Translator, I18nError>` - The translator instance or an error if the language is unsupported
    pub fn new(lang: &str) -> Result<Self, I18nError> {
        let translate_fn = match lang {
            "en" => en::translate,
            "fr" => fr::translate,
            "de" => de::translate,
            _ => {
                return Err(I18nError::UnsupportedLanguage(
                    lang.to_string(),
                ))
            }
        };

        Ok(Translator {
            lang: lang.to_string(),
            translate_fn,
        })
    }

    /// Translates the given text.
    ///
    /// # Arguments
    ///
    /// * `text` - A string slice that holds the text to be translated
    ///
    /// # Returns
    ///
    /// * `Result<String, I18nError>` - The translated string or an error if translation fails
    pub fn translate(&self, text: &str) -> Result<String, I18nError> {
        (self.translate_fn)(text)
    }

    /// Returns the language code of this translator.
    pub fn lang(&self) -> &str {
        &self.lang
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_translation_english() {
        let translator = Translator::new("en").unwrap();
        assert_eq!(translator.translate("Hello").unwrap(), "Hello");
    }

    #[test]
    fn test_translation_french() {
        let translator = Translator::new("fr").unwrap();
        assert_eq!(translator.translate("Hello").unwrap(), "Bonjour");
    }

    #[test]
    fn test_translation_german() {
        let translator = Translator::new("de").unwrap();
        assert_eq!(translator.translate("Hello").unwrap(), "Hallo");
    }

    #[test]
    fn test_unsupported_language() {
        let result = Translator::new("es");
        assert!(result.is_err());
    }
}

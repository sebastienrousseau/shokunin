//! # Language Detection Module
//!
//! This module provides a simple mechanism to detect the language of a given
//! text using regex-based heuristics. It supports basic detection for English,
//! French, and German.

use crate::error::I18nError;
use regex::Regex;

/// Struct for detecting the language of a given text.
pub struct LanguageDetector {
    patterns: Vec<(Regex, &'static str)>,
}

impl LanguageDetector {
    /// Creates a new instance of `LanguageDetector`.
    ///
    /// The instance includes predefined patterns to detect English, French, and German.
    ///
    /// # Returns
    ///
    /// * `LanguageDetector` - A new instance of the `LanguageDetector` struct
    pub fn new() -> Self {
        let patterns = vec![
            (Regex::new(r"(?i)\b(the|a|an)\b").unwrap(), "en"), // English pattern, case-insensitive
            (Regex::new(r"(?i)\b(le|la|les)\b").unwrap(), "fr"), // French pattern, case-insensitive
            (Regex::new(r"(?i)\b(der|die|das)\b").unwrap(), "de"), // German pattern, case-insensitive
        ];
        LanguageDetector { patterns }
    }

    /// Detects the language of the given text.
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
    /// use ssg_i18n::language_detector::LanguageDetector;
    ///
    /// let detector = LanguageDetector::new();
    /// let result = detector.detect("The quick brown fox");
    /// assert_eq!(result.unwrap(), "en");
    /// ```
    pub fn detect(&self, text: &str) -> Result<String, I18nError> {
        // Trim and normalize the input text
        let normalized_text = text.trim();

        for (pattern, lang) in &self.patterns {
            if pattern.is_match(normalized_text) {
                return Ok(lang.to_string());
            }
        }
        Err(I18nError::LanguageDetectionFailed)
    }
}

/// Implements the `Default` trait for `LanguageDetector`.
/// This allows the struct to be initialized with default settings.
impl Default for LanguageDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_detection() {
        let detector = LanguageDetector::new();
        assert_eq!(
            detector.detect("The quick brown fox").unwrap(),
            "en"
        );
        assert_eq!(detector.detect("Le chat noir").unwrap(), "fr");
        assert_eq!(
            detector.detect("Der schnelle Fuchs").unwrap(),
            "de"
        );
    }

    #[test]
    fn test_undetectable_language() {
        let detector = LanguageDetector::new();
        assert!(detector.detect("こんにちは").is_err());
    }
}

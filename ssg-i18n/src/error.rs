use thiserror::Error;

/// Enum representing various errors that can occur during internationalization and translation.
#[derive(Error, Debug)]
pub enum I18nError {
    /// Error for when language detection fails.
    #[error("Failed to detect language")]
    LanguageDetectionFailed,

    /// Error for when a translation fails due to unsupported text.
    #[error("Failed to translate text: {0}")]
    TranslationFailed(String),

    /// Error for unsupported languages.
    #[error("Unsupported language: {0}")]
    UnsupportedLanguage(String),
}

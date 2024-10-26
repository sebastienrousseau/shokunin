use langweave::error::I18nError;
use lazy_static::lazy_static;
use std::collections::HashMap;

lazy_static! {
    static ref TRANSLATIONS: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
        m.insert("Hello", "Hello");
        m.insert("Goodbye", "Goodbye");
        m.insert("main_logger_msg", "\nPlease run `ssg --help` for more information.\n");
        m.insert("lib_banner_log_msg", "Banner printed successfully");
        m.insert("lib_args_log_msg", "Arguments processed successfully");
        m.insert("lib_server_log_msg", "Server started successfully");
        // Add more translations here
        m
    };
}

/// Translates the given text into English.
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

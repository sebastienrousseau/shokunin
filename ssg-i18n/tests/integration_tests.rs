use ssg_i18n::{detect_language, translate, I18nError, Translator};

#[test]
fn test_translation() {
    assert_eq!(translate("fr", "Hello").unwrap(), "Bonjour");
    assert_eq!(translate("de", "Goodbye").unwrap(), "Auf Wiedersehen");
    assert!(translate("invalid", "Hello").is_err());
}

#[test]
fn test_language_detection() {
    assert_eq!(detect_language("The quick brown fox").unwrap(), "en");
    assert_eq!(detect_language("Le chat noir").unwrap(), "fr");
    assert_eq!(detect_language("Der schnelle Fuchs").unwrap(), "de");
}

#[test]
fn test_translator() {
    let translator = Translator::new("en").unwrap();
    assert_eq!(translator.translate("Hello").unwrap(), "Hello");

    let result = Translator::new("invalid");
    assert!(matches!(result, Err(I18nError::UnsupportedLanguage(_))));
}

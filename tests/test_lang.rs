// Copyright © 2024 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

#[cfg(test)]
mod tests {
    use staticrux::languages::{
        get_lang, get_lang_code, get_lang_from_accept, get_langs,
        translate,
    };

    #[test]
    fn test_translate() {
        assert_eq!(translate("en", "Hello"), "Hello");
        assert_eq!(translate("fr", "Hello"), "Bonjour");
    }

    #[test]
    fn test_get_langs() {
        let langs = get_langs();
        assert_eq!(langs.len(), 3); // Update this line to expect 3 languages
        assert_eq!(langs[0], ("en", "English"));
        assert_eq!(langs[1], ("fr", "French"));
        assert_eq!(langs[2], ("de", "German"));
    }

    #[test]
    fn test_get_lang() {
        assert_eq!(get_lang("en"), Some("English".to_string()));
        assert_eq!(get_lang("fr"), Some("French".to_string()));
    }

    #[test]
    fn test_get_lang_code() {
        assert_eq!(get_lang_code("English"), Some("en"));
        assert_eq!(get_lang_code("French"), Some("fr"));
    }

    #[test]
    fn test_get_lang_from_accept() {
        println!("Testing get_lang_from_accept function...");

        // Test case 1
        println!("Test case 1: en-GB,en;q=0.9");
        let result1 = get_lang_from_accept("en-GB,en;q=0.9");
        println!("Result 1: {:?}", result1);
        assert_eq!(result1, Some("English".to_string()));

        // Test case 2
        println!("Test case 2: fr-FR,fr;q=0.9,en;q=0.8");
        let result2 = get_lang_from_accept("fr-FR,fr;q=0.9,en;q=0.8");
        println!("Result 2: {:?}", result2);
        assert_eq!(result2, Some("French".to_string()));

        // Test case 3
        println!("Test case 3: de-DE,de;q=0.9,en;q=0.8");
        let result3 = get_lang_from_accept("de-DE,de;q=0.9,en;q=0.8");
        println!("Result 3: {:?}", result3);
        assert_eq!(result3, Some("German".to_string()));
    }
}

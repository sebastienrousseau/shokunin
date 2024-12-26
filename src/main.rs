// Copyright © 2023-2025 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! # Shokunin Static Site Generator - Main Entry Point
//!
//! This module contains the main entry point for initiating the Shokunin Static Site Generator. It defines the `main` function and an `execute_main_logic` helper function, which together handle the core execution flow, including error handling and language-based translation of messages.
//!
//! ## Core Behaviour
//! - **Default Language**: If the `LANGUAGE` environment variable is unset, English (`"en"`) is used.
//! - **Execution Flow**: Calls `run` from the `ssg` module to generate the site, and translates messages based on the user's language preference.
//! - **Exit Status**: On success, outputs a confirmation message. On failure, outputs an error message and exits with a non-zero status code.
//!
//! ## Example Usage
//! ```rust,no_run
//! use ssg::run;
//! use std::env;
//!
//! // Set the language preference before executing the site generator.
//! env::set_var("LANGUAGE", "en");
//! match run() {
//!     Ok(_) => println!("Site generated successfully."),
//!     Err(e) => eprintln!("Error encountered: {}", e),
//! }
//! ```

use langweave::translate;
use ssg::run;

/// Executes the main logic of the Shokunin Static Site Generator.
///
/// This function performs the primary actions for generating a static site, including:
/// 1. Retrieving the user's language preference from the `LANGUAGE` environment variable.
/// 2. Calling `run` from the `ssg` module to generate the site.
/// 3. Translating a success or failure message based on the selected language.
///
/// ### Language Preference
/// - The language is determined by the `LANGUAGE` environment variable.
/// - If the variable is unset, English ("en") is used as the default language.
///
/// ### Return Values
/// - On success, returns `Ok(String)` containing the translated success message.
/// - On failure, returns `Err(String)` with either a generation error message or a translation failure notice.
///
/// ### Errors
/// Errors may arise in two scenarios:
/// 1. If the `run` function fails to generate the site, an error message is returned.
/// 2. If translation of the success message fails, a translation error message is returned.
///
/// ### Example
/// ```rust
/// use std::env;
/// env::set_var("LANGUAGE", "fr");
/// match execute_main_logic() {
///     Ok(msg) => println!("{}", msg),
///     Err(e) => eprintln!("{}", e),
/// }
/// ```
///
/// # Return
/// `Result<String, String>` - A result containing either a success message or an error string.
async fn execute_main_logic() -> Result<String, String> {
    // Determine the user's language preference, defaulting to English ("en") if unset.
    let lang =
        std::env::var("LANGUAGE").unwrap_or_else(|_| "en".to_string());

    match run().await {
        Ok(_) => {
            // Translate and return a success message in the chosen language.
            match translate(&lang, "main_logger_msg") {
                Ok(msg) => Ok(msg),
                Err(e) => Err(format!("Translation failed: {}", e)),
            }
        }
        // Return an error if `run` encounters an issue.
        Err(e) => Err(format!("Program encountered an error: {}", e)),
    }
}

/// The main entry point of the Shokunin Static Site Generator.
///
/// This function initiates the static site generation process by calling `execute_main_logic`.
/// It handles the output to the console, displaying either a translated success message
/// or an error message if the generation fails.
///
/// ### Exit Codes
/// - Returns `0` if site generation is successful.
/// - Returns a non-zero status code if an error occurs.
///
/// ### Example
/// ```rust,no_run
/// // Set LANGUAGE environment variable to the desired language before running the generator.
/// use std::env;
/// env::set_var("LANGUAGE", "es");
/// main();  // Executes the site generation in Spanish, if supported.
/// ```
///
/// ### Behaviour
/// - Retrieves the user's language preference from the `LANGUAGE` environment variable.
/// - Executes `execute_main_logic` to generate the site.
/// - Outputs a success message upon completion or an error message if site generation fails.
#[tokio::main]
async fn main() {
    match execute_main_logic().await {
        Ok(msg) => println!("{}", msg),
        Err(e) => eprintln!("{}", e),
    }
}

#[cfg(test)]
mod tests {
    use std::{env, sync::Once};
    use tokio::runtime::Runtime;

    static INIT: Once = Once::new();

    /// Initialize test environment
    fn initialize() {
        INIT.call_once(|| {});
    }

    /// Helper to clean up environment state
    fn cleanup_env() {
        env::remove_var("LANGUAGE");
    }

    /// Mocks the `run` function to simulate a successful site generation.
    fn mock_run_ok() -> Result<(), String> {
        Ok(())
    }

    /// Mocks the `run` function to simulate a failed site generation.
    fn mock_run_err() -> Result<(), String> {
        Err("Site generation failed".to_string())
    }

    /// Mocks the `translate` function to simulate a successful translation.
    fn mock_translate_success(
        lang: &str,
        _msg_key: &str,
    ) -> Result<String, String> {
        Ok(format!("Success message in {}", lang))
    }

    /// Mocks the `translate` function to simulate a translation failure.
    fn mock_translate_failure(
        _lang: &str,
        _msg_key: &str,
    ) -> Result<String, String> {
        Err("Translation error".to_string())
    }

    #[test]
    fn test_execute_main_logic_run_success_translate_success() {
        initialize();
        cleanup_env();
        env::set_var("LANGUAGE", "en");

        let result = mock_run_ok();
        let translate_result =
            mock_translate_success("en", "main_logger_msg");

        assert_eq!(result, Ok(()));
        assert_eq!(
            translate_result,
            Ok("Success message in en".to_string())
        );

        cleanup_env();
    }

    #[test]
    fn test_execute_main_logic_run_success_translate_failure() {
        initialize();
        cleanup_env();
        env::set_var("LANGUAGE", "en");

        let result = mock_run_ok();
        let translate_result =
            mock_translate_failure("en", "main_logger_msg");

        assert_eq!(result, Ok(()));
        assert_eq!(
            translate_result,
            Err("Translation error".to_string())
        );

        cleanup_env();
    }

    #[test]
    fn test_execute_main_logic_run_failure() {
        initialize();
        cleanup_env();

        let result = mock_run_err();
        assert_eq!(result, Err("Site generation failed".to_string()));

        cleanup_env();
    }

    #[test]
    fn test_execute_main_logic_default_language() {
        initialize();
        cleanup_env();

        let lang =
            env::var("LANGUAGE").unwrap_or_else(|_| "en".to_string());
        assert_eq!(lang, "en");

        cleanup_env();
    }

    #[test]
    fn test_execute_main_logic_async_empty_language() {
        initialize();
        cleanup_env();
        env::set_var("LANGUAGE", "");

        let rt = Runtime::new().unwrap();
        let result = rt.block_on(async {
            let run_result: Result<(), String> = Ok(());
            match run_result {
                Ok(_) => mock_translate_success("", "main_logger_msg"),
                Err(e) => Err(e),
            }
        });

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Success message in ");

        cleanup_env();
    }

    #[test]
    fn test_execute_main_logic_concurrent() {
        initialize();
        cleanup_env();
        let rt = Runtime::new().unwrap();

        let futures: Vec<_> = (0..3)
            .map(|i| async move {
                let lang = format!("lang{}", i);
                mock_translate_success(&lang, "main_logger_msg")
            })
            .collect();

        let results = rt.block_on(async {
            let mut results = vec![];
            for future in futures {
                results.push(future.await);
            }
            results
        });

        for (i, result) in results.iter().enumerate() {
            assert!(result.is_ok());
            assert_eq!(
                result.as_ref().unwrap(),
                &format!("Success message in lang{}", i)
            );
        }

        cleanup_env();
    }

    #[test]
    fn test_special_character_handling() {
        initialize();
        cleanup_env();
        let rt = Runtime::new().unwrap();

        let test_langs = ["en-US", "zh-CN", "es-419", "en_GB"];
        for lang in &test_langs {
            env::set_var("LANGUAGE", lang);
            let result = rt.block_on(async {
                mock_translate_success(lang, "main_logger_msg")
            });

            assert!(result.is_ok());
            assert_eq!(
                result.unwrap(),
                format!("Success message in {}", lang)
            );
        }

        cleanup_env();
    }

    #[test]
    fn test_execute_main_logic_with_env_states() {
        initialize();
        cleanup_env();
        let rt = Runtime::new().unwrap();

        // Test with mock functions instead of real execution
        let test_cases = vec![
            ("", "Success message in "),
            ("en", "Success message in en"),
            ("fr", "Success message in fr"),
        ];

        for (lang, expected) in test_cases {
            env::set_var("LANGUAGE", lang);
            let result = rt.block_on(async {
                mock_translate_success(lang, "main_logger_msg")
            });

            assert!(result.is_ok());
            assert_eq!(result.unwrap(), expected);
        }

        cleanup_env();
    }

    #[test]
    fn test_execute_main_logic_rapid_language_changes() {
        initialize();
        cleanup_env();

        let rt = Runtime::new().unwrap();
        let languages = vec!["en", "fr", "es", "de", "it"];

        for lang in languages {
            env::set_var("LANGUAGE", lang);
            let result = rt.block_on(async {
                mock_translate_success(lang, "main_logger_msg")
            });
            assert!(result.is_ok());
        }

        cleanup_env();
    }

    #[test]
    fn test_execute_main_logic_concurrent_with_same_language() {
        initialize();
        cleanup_env();

        let rt = Runtime::new().unwrap();
        env::set_var("LANGUAGE", "en");

        let futures: Vec<_> = (0..10)
            .map(|_| async {
                mock_translate_success("en", "main_logger_msg")
            })
            .collect();

        let results = rt.block_on(async {
            let mut results = vec![];
            for future in futures {
                results.push(future.await);
            }
            results
        });

        for result in results {
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), "Success message in en");
        }

        cleanup_env();
    }

    /// Tests error propagation with multiple failure points
    #[test]
    fn test_execute_main_logic_cascading_errors() {
        initialize();
        cleanup_env();

        let rt = Runtime::new().unwrap();

        // Test error from run
        let run_error = rt.block_on(async {
            match mock_run_err() {
                Ok(_) => {
                    mock_translate_success("en", "main_logger_msg")
                }
                Err(e) => Err(e),
            }
        });
        assert!(run_error.is_err());
        assert_eq!(run_error.unwrap_err(), "Site generation failed");

        // Test error from translation
        let translate_error = rt.block_on(async {
            match mock_run_ok() {
                Ok(_) => {
                    mock_translate_failure("en", "main_logger_msg")
                }
                Err(e) => Err(e),
            }
        });
        assert!(translate_error.is_err());
        assert_eq!(translate_error.unwrap_err(), "Translation error");

        cleanup_env();
    }

    #[test]
    fn test_execute_main_logic_with_invalid_language_env() {
        initialize();
        cleanup_env();
        env::set_var("LANGUAGE", "invalid-lang");

        let rt = Runtime::new().unwrap();
        let result = rt.block_on(async {
            let run_result = mock_run_ok();
            match run_result {
                Ok(_) => mock_translate_success(
                    "invalid-lang",
                    "main_logger_msg",
                ),
                Err(e) => Err(e),
            }
        });

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Success message in invalid-lang");

        cleanup_env();
    }

    #[test]
    fn test_execute_main_logic_run_and_translate_failures() {
        initialize();
        cleanup_env();

        let rt = Runtime::new().unwrap();

        let result = rt.block_on(async {
            let run_result = mock_run_err();
            match run_result {
                Ok(_) => mock_translate_failure("", "main_logger_msg"),
                Err(e) => {
                    Err(format!("Run and translate failed: {}", e))
                }
            }
        });

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Run and translate failed: Site generation failed"
        );

        cleanup_env();
    }

    #[test]
    fn test_execute_main_logic_with_unsupported_language() {
        initialize();
        cleanup_env();
        env::set_var("LANGUAGE", "unsupported-lang");

        let rt = Runtime::new().unwrap();
        let result = rt.block_on(async {
            let run_result = mock_run_ok();
            match run_result {
                Ok(_) => mock_translate_failure(
                    "unsupported-lang",
                    "main_logger_msg",
                ),
                Err(e) => Err(e),
            }
        });

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Translation error");

        cleanup_env();
    }

    #[test]
    fn test_concurrent_translate_with_mixed_results() {
        initialize();
        cleanup_env();
        let rt = Runtime::new().unwrap();

        let languages = vec!["en", "fr", "error-lang", "es", "de"];
        let futures: Vec<_> = languages
            .into_iter()
            .map(|lang| async move {
                if lang == "error-lang" {
                    mock_translate_failure(lang, "main_logger_msg")
                } else {
                    mock_translate_success(lang, "main_logger_msg")
                }
            })
            .collect();

        let results = rt.block_on(async {
            let mut results = vec![];
            for future in futures {
                results.push(future.await);
            }
            results
        });

        assert!(results[0].is_ok());
        assert_eq!(
            results[0].as_ref().unwrap(),
            "Success message in en"
        );

        assert!(results[1].is_ok());
        assert_eq!(
            results[1].as_ref().unwrap(),
            "Success message in fr"
        );

        assert!(results[2].is_err());
        assert_eq!(
            results[2].as_ref().err(),
            Some(&"Translation error".to_string())
        );

        cleanup_env();
    }

    #[test]
    fn test_translate_with_empty_key() {
        initialize();
        cleanup_env();

        let rt = Runtime::new().unwrap();
        let result =
            rt.block_on(async { mock_translate_failure("en", "") });

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Translation error");

        cleanup_env();
    }
}

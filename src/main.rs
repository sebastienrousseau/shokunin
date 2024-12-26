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
    use std::env;

    /// Mocks the `run` function to simulate a successful site generation.
    ///
    /// ### Return
    /// Returns `Ok(())` to indicate that the site generation was successful.
    fn mock_run_ok() -> Result<(), String> {
        Ok(())
    }

    /// Mocks the `run` function to simulate a failed site generation.
    ///
    /// ### Return
    /// Returns `Err(String)` to simulate a failure with an error message.
    fn mock_run_err() -> Result<(), String> {
        Err("Site generation failed".to_string())
    }

    /// Mocks the `translate` function to simulate a successful translation.
    ///
    /// ### Parameters
    /// - `lang`: Language code (e.g., "en", "fr").
    /// - `_msg_key`: The message key for the translation.
    ///
    /// ### Return
    /// Returns `Ok(String)` containing a success message in the specified language.
    fn mock_translate_success(
        lang: &str,
        _msg_key: &str,
    ) -> Result<String, String> {
        Ok(format!("Success message in {}", lang))
    }

    /// Mocks the `translate` function to simulate a translation failure.
    ///
    /// ### Parameters
    /// - `_lang`: Language code, though it is unused as this mock always fails.
    /// - `_msg_key`: The message key for the translation.
    ///
    /// ### Return
    /// Returns `Err(String)` to indicate a translation error.
    fn mock_translate_failure(
        _lang: &str,
        _msg_key: &str,
    ) -> Result<String, String> {
        Err("Translation error".to_string())
    }

    /// Tests successful site generation and message translation.
    ///
    /// ### Behaviour
    /// Simulates a scenario where `run` succeeds, and `translate` also succeeds, producing
    /// a successful message output.
    #[test]
    fn test_execute_main_logic_run_success_translate_success() {
        env::set_var("LANGUAGE", "en");

        let result = mock_run_ok();
        let translate_result =
            mock_translate_success("en", "main_logger_msg");

        assert_eq!(result, Ok(()));
        assert_eq!(
            translate_result,
            Ok("Success message in en".to_string())
        );
    }

    /// Tests successful site generation with a translation failure.
    ///
    /// ### Behaviour
    /// Simulates a scenario where `run` succeeds, but `translate` fails, resulting
    /// in a translation error message.
    #[test]
    fn test_execute_main_logic_run_success_translate_failure() {
        env::set_var("LANGUAGE", "en");

        let result = mock_run_ok();
        let translate_result =
            mock_translate_failure("en", "main_logger_msg");

        assert_eq!(result, Ok(()));
        assert_eq!(
            translate_result,
            Err("Translation error".to_string())
        );
    }

    /// Tests a failed site generation process.
    ///
    /// ### Behaviour
    /// Simulates a scenario where `run` fails, leading to a site generation error message.
    #[test]
    fn test_execute_main_logic_run_failure() {
        let result = mock_run_err();
        assert_eq!(result, Err("Site generation failed".to_string()));
    }

    /// Tests the default language setting when `LANGUAGE` is not specified.
    ///
    /// ### Behaviour
    /// Ensures that "en" is used as the default language when the `LANGUAGE` environment
    /// variable is unset.
    #[test]
    fn test_execute_main_logic_default_language() {
        env::remove_var("LANGUAGE");
        let lang =
            env::var("LANGUAGE").unwrap_or_else(|_| "en".to_string());
        assert_eq!(lang, "en");
    }
}

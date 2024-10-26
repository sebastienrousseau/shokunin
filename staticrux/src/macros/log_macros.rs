// Copyright © 2024 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! This module contains macros related to logging messages at various log levels and formats.
//!
//! It includes a custom logging macro, `macro_log_info`, which allows logging messages with
//! specified log levels, components, descriptions, and formats.

/// Custom logging macro for various log levels and formats.
///
/// # Parameters
///
/// * `$level`: The log level of the message.
/// * `$component`: The component where the log is coming from.
/// * `$description`: A description of the log message.
/// * `$format`: The format of the log message.
///
/// # Example
///
/// ```
/// use staticrux::macro_log_info;
/// use rlg::log_level::LogLevel;
/// use rlg::log_format::LogFormat;
///
/// let level = LogLevel::INFO;
/// let component = "TestComponent";
/// let description = "Test description";
/// let format = LogFormat::CLF;
///
/// let log = macro_log_info!(&level, component, description, &format);
/// ```
#[macro_export]
macro_rules! macro_log_info {
    ($level:expr, $component:expr, $description:expr, $format:expr) => {{
        use dtt::datetime::DateTime;
        use rlg::{log::Log, log_format::LogFormat};
        use vrd::random::Random;
        let date = DateTime::new();
        let mut rng = Random::default();
        let session_id = rng.rand().to_string();
        Log::new(
            &session_id,
            &date.to_string(),
            $level,
            $component,
            $description,
            $format,
        )
    }};
}

#[cfg(test)]
mod tests {
    use rlg::log_level::LogLevel;

    #[test]
    fn test_macro_log_info() {
        let log = macro_log_info!(
            &LogLevel::INFO,
            "TestComponent",
            "Test message",
            &LogFormat::CLF
        );
        assert_eq!(log.component, "TestComponent");
        assert_eq!(log.description, "Test message");
    }
}

// Copyright © 2024 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Contains macros related to directory operations.

//! This module contains macros related to logging messages at various log levels and formats.
//!
//! It includes a custom logging macro, `macro_log_info`, which allows logging messages with
//! specified log levels, components, descriptions, and formats.
//!
//! # Custom Logging Macro
//!
//! The `macro_log_info` macro is designed for logging messages with customizable log levels,
//! components, descriptions, and formats. It provides flexibility in defining log messages
//! according to specific requirements.
//!
//! # Parameters
//!
//! - `$level`: The log level of the message.
//! - `$component`: The component where the log is coming from.
//! - `$description`: A description of the log message.
//! - `$format`: The format of the log message.
//!
//! # Example
//!
//! ```
//! use ssg_core::macro_log_info;
//! use rlg::log_level::LogLevel;
//! use rlg::log_format::LogFormat;
//!
//! let level = LogLevel::INFO;
//! let component = "TestComponent";
//! let description = "Test description";
//! let format = LogFormat::CLF;
//!
//! // Log an informational message
//! let log =
//!     macro_log_info!(&level, component, description, &format);
//!
//! // Further processing of log message...
//! ```
//!
//! This example demonstrates how to use the `macro_log_info` macro to log an informational message
//! with specified log level, component, description, and format. Users can customize log messages
//! according to their logging requirements by modifying the parameters passed to the macro.
//!
//! # Note
//!
//! This module assumes familiarity with Rust macros and their usage. Users are encouraged
//! to review Rust macro documentation for a better understanding of how to work with macros effectively.

/// Custom logging macro for various log levels and formats.
///
/// # Parameters
///
/// * `$level`: The log level of the message.
/// * `$component`: The component where the log is coming from.
/// * `$description`: A description of the log message.
/// * `$format`: The format of the log message.
///
#[macro_export]
macro_rules! macro_log_info {
    ($level:expr, $component:expr, $description:expr, $format:expr) => {{
        // Import necessary modules
        use dtt::datetime::DateTime; // Date and time module
        use rlg::{
            log::Log,
            log_format::LogFormat,
        }; // Logging module and log format module
        use vrd::random::Random; // Random number generator module

        // Get the current date and time in ISO 8601 format.
        let date = DateTime::new(); // Create a new DateTime instance

        // Create a new random number generator
        let mut rng = Random::default(); // Default random number generator
        let session_id = rng.rand().to_string(); // Generate session ID

        // Create a new log instance
        let log = Log::new(
            &session_id, // Session ID
            &date.to_string(), // Date and time
            $level, // Log level
            $component, // Component name
            $description, // Log description
            $format, // Log format
        );
        log // Return the Log instance
    }};
}

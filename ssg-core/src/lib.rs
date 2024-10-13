//! # ssg-core
//!
//! `ssg-core` is the core library for the Shokunin Static Site Generator.
//! It provides essential functionality for compiling, processing, and
//! serving static websites.
//!
//! This crate includes modules for:
//! - Compiling static site content
//! - Language processing and translation
//! - Logging and diagnostics
//! - Macro definitions for common operations
//! - Data models and structures
//! - Various utility modules for file handling, templating, etc.
//! - A built-in server for local testing
//!
//! ## Main Components
//!
//! - `compile`: Main function for compiling static site content
//! - `translate`: Function for translating text between supported languages
//! - `init_logger`: Initialize the logging system
//! - `start`: Start the built-in server for local testing
//! - `generate_unique_string`: Generate a unique identifier string

/// Compiler module for processing and generating static site content
pub mod compiler;

/// Logging and diagnostic utilities
pub mod loggers;

/// Macro definitions for common operations
pub mod macros;

/// Data models and structures used throughout the crate
pub mod models;

/// Various modules for specific functionalities (e.g., HTML generation, RSS feeds)
pub mod modules;

/// Utility functions and helpers
pub mod utilities;

// Re-export commonly used items for easier access
pub use compiler::service::compile;
pub use http_handle::Server;
pub use loggers::init_logger;
pub use ssg_template;
pub use utilities::uuid::generate_unique_string;

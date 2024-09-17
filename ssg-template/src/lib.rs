//! # ssg-template
//!
//! `ssg-template` is a flexible templating engine designed for use in static site generators,
//! particularly tailored for the Shokunin Static Site Generator. It provides a simple yet
//! powerful way to render templates with dynamic content.
//!
//! ## Features
//!
//! - Variable interpolation
//! - Template file management
//! - Error handling
//! - Extensible context system
//!

/// The `context` module contains the `Context` struct, which is used to store and manage template variables.
pub mod context;

/// The `engine` module contains the `Engine` struct, which is used to render templates.
pub mod engine;

/// The `error` module contains the `TemplateError` enum, which represents errors that can occur during template processing.
pub mod error;

pub use context::Context;
pub use engine::{Engine, PageOptions};
pub use error::TemplateError;

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::{Context, Engine, TemplateError};
}

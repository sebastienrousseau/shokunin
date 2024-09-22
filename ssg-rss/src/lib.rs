// Copyright © 2024 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! # ssg-rss
//!
//! `ssg-rss` is a Rust library for generating RSS feeds for the Shokunin Static Site Generator.
//! It provides functionality to create RSS 2.0 feeds with support for various RSS elements and attributes.
//!
//! ## Features
//!
//! - Generate RSS 2.0 feeds
//! - Support for standard RSS elements (title, link, description, etc.)
//! - Support for optional elements (language, pubDate, category, etc.)
//! - Atom link support
//! - Image element support
//!
//! ## Usage
//!
//! ```rust
//! use ssg_rss::{RssData, generate_rss};
//!
//! let rss_data = RssData::new()
//!     .title("My Blog")
//!     .link("https://myblog.com")
//!     .description("A blog about Rust programming");
//!
//! match generate_rss(&rss_data) {
//!     Ok(rss_feed) => println!("{}", rss_feed),
//!     Err(e) => eprintln!("Error generating RSS: {}", e),
//! }
//! ```

/// The `data` module contains the main types and data structures used to generate RSS feeds.
pub mod data;
/// The `error` module contains error types used by the library.
pub mod error;
/// The `generator` module contains the main logic for generating RSS feeds.
pub mod generator;
/// The `macros` module contains procedural macros used by the library.
pub mod macros;

pub use data::RssData;
pub use error::RssError;
pub use generator::generate_rss;

// Re-export main types for ease of use
pub use error::Result;

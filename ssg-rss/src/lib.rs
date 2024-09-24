// Copyright © 2024 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! # ssg-rss
//!
//! `ssg-rss` is a Rust library for generating, serializing, and deserializing RSS feeds.
//! It supports multiple RSS versions and provides functionality to create and parse
//! RSS feeds with various elements and attributes.
//!
//! ## Features
//!
//! - Generate RSS feeds for versions 0.90, 0.91, 0.92, 1.0, and 2.0
//! - Serialize RSS data to XML format
//! - Deserialize XML content into RSS data structures
//! - Support for standard RSS elements (title, link, description, etc.)
//! - Support for optional elements (language, pubDate, category, etc.)
//! - Atom link support
//! - Image element support
//!
//! ## Usage
//!
//! ```rust
//! use ssg_rss::{RssData, RssVersion, generate_rss, parse_rss};
//!
//! let rss_data = RssData::new()
//!     .title("My Blog")
//!     .link("https://myblog.com")
//!     .description("A blog about Rust programming");
//!
//! match generate_rss(&rss_data) {
//!     Ok(rss_feed) => {
//!         println!("Generated RSS feed: {}", rss_feed);
//!
//!         // Parse the generated RSS feed
//!         match parse_rss(&rss_feed) {
//!             Ok(parsed_data) => println!("Parsed RSS data: {:?}", parsed_data),
//!             Err(e) => eprintln!("Error parsing RSS: {}", e),
//!         }
//!     },
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
/// The `parser` module contains the logic for parsing RSS feeds.
pub mod parser;
/// The `version` module contains definitions for different RSS versions.
pub mod version;

pub use data::RssData;
pub use error::RssError;
pub use generator::generate_rss;
pub use parser::parse_rss;
pub use version::RssVersion;

// Re-export main types for ease of use
pub use error::Result;

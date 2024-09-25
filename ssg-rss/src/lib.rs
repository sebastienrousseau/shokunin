// Copyright © 2024 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

// Copyright © 2024 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! # ssg-rss
//!
//! `ssg-rss` is a comprehensive Rust library for generating, parsing, serializing, and deserializing RSS feeds.
//! It supports multiple versions of RSS and provides flexibility in creating and handling feeds across different formats.
//!
//! ## Key Features
//!
//! - **Support for Multiple Versions**: Generate RSS feeds in versions 0.90, 0.91, 0.92, 1.0 (RDF-based), and 2.0.
//! - **Feed Generation**: Easily create RSS feeds from structured data using `RssData`.
//! - **Feed Parsing**: Parse existing RSS feeds back into structured `RssData` format.
//! - **Serialization & Deserialization**: Convert RSS data into XML format and deserialize XML content into RSS data structures.
//! - **Extensible Elements**: Handles standard RSS elements (title, link, description, etc.) and optional elements (pubDate, language, category, etc.).
//! - **Atom Support**: Includes optional Atom link elements for better compatibility with modern standards.
//! - **Image Support**: RSS 2.0 feeds can include images directly within the feed.
//!
//! ## Getting Started
//!
//! Example usage for generating and parsing an RSS feed:
//!
//! ```rust
//! use ssg_rss::{RssData, RssVersion, generate_rss, parse_rss};
//!
//! let rss_data = RssData::new(Some(RssVersion::RSS2_0))
//!     .title("My Rust Blog")
//!     .link("https://myrustblog.com")
//!     .description("A blog about Rust programming and tutorials.");
//!
//! match generate_rss(&rss_data) {
//!     Ok(rss_feed) => {
//!         println!("Generated RSS feed: {}", rss_feed);
//!
//!         // Parsing the generated RSS feed
//!         match parse_rss(&rss_feed) {
//!             Ok(parsed_data) => println!("Parsed RSS data: {:?}", parsed_data),
//!             Err(e) => eprintln!("Error parsing RSS feed: {}", e),
//!         }
//!     },
//!     Err(e) => eprintln!("Error generating RSS feed: {}", e),
//! }
//! ```
//!
//! ## Modules
//!
//! - **data**: Defines the main data structures (`RssData`, `RssItem`) used for representing RSS feeds.
//! - **error**: Defines custom error types and handling for RSS generation and parsing errors.
//! - **generator**: Contains the core functionality for generating RSS feeds in various versions.
//! - **macros**: Provides procedural macros to extend functionality and ease of use.
//! - **parser**: Handles the parsing of RSS feeds back into data structures.
//! - **version**: Contains enumerations and definitions for supported RSS versions.
//!
//! ## License
//! This library is licensed under either of Apache License, Version 2.0 or MIT License.

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

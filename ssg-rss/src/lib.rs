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

///  The `error` module provides error types for the RSS library.
pub mod error;

/// The `generator` module provides functions for generating RSS feeds.
pub mod generator;

/// The `macros` module provides macros for generating RSS elements.
pub mod macros;

/// The `models` module contains data structures for RSS elements and attributes.
pub mod models;

/// The `RssError` type represents errors that can occur when generating an RSS feed.
pub use error::RssError;

/// The `RssData` struct represents the metadata for generating an RSS feed.
pub use models::data::RssData;

/// The `generate_rss` function generates an RSS feed from the given `RssData` struct.
pub use generator::generate_rss;

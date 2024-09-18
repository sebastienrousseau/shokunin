//! # SSG Sitemap
//!
//! `ssg-sitemap` is a Rust library for generating and managing XML sitemaps.
//! It provides functionality to create, parse, and manipulate sitemaps for
//! static site generators and other web applications.

/// Contains error types specific to sitemap operations.
pub mod error;

/// Provides the core functionality for creating and managing sitemaps.
pub mod sitemap;

pub use error::SitemapError;
pub use sitemap::{
    create_site_map_data, ChangeFreq, SiteMapData, Sitemap,
};

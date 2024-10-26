// Copyright © 2024 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Core data models for the Shokunin Static Site Generator
//!
//! This module defines the fundamental data structures and validation logic used
//! throughout the static site generator. It provides type-safe representations
//! of various content types along with comprehensive validation and security checks.
//!
//! # Key Features
//!
//! - Comprehensive input validation
//! - Strong security guarantees
//! - Extensible data models
//! - Rich error handling
//! - Type-safe field constraints
//!
//! # Security Features
//!
//! - URL/URI sanitization and validation
//! - Path traversal prevention
//! - HTML/script injection protection
//! - Character set validation
//! - Input length limits
//! - Format verification for dates, colors, etc.
//!
//! # Examples
//!
//! ```rust
//! use staticrux::models::data::{FileData, PageData, SecurityData};
//!
//! // Create a new page
//! let page = PageData::new(
//!     "Welcome".to_string(),
//!     "Welcome to my site".to_string(),
//!     "2024-02-20T12:00:00Z".to_string(),
//!     "/welcome".to_string(),
//! );
//!
//! // Validate the page data
//! if let Err(e) = page.validate() {
//!     eprintln!("Invalid page data: {}", e);
//! }
//! ```

use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;
use time::{format_description::well_known::Rfc3339, OffsetDateTime};
use url::Url;

/// Maximum length for text fields to prevent DoS
const MAX_TEXT_LENGTH: usize = 5000;
/// Maximum length for short text fields
const MAX_SHORT_TEXT_LENGTH: usize = 200;
/// Maximum length for metadata fields
const MAX_METADATA_LENGTH: usize = 1000;

/// Maximum length for manifest name
const MAX_MANIFEST_NAME_LENGTH: usize = 45;

/// Maximum length for manifest short name
const MAX_MANIFEST_SHORT_NAME_LENGTH: usize = 12;

/// Errors that can occur when working with data models
#[derive(Error, Debug)]
pub enum DataError {
    /// Invalid domain name
    #[error("Invalid domain name: {0}")]
    InvalidDomain(String),

    /// Missing required field
    #[error("Missing required field: {0}")]
    MissingField(String),

    /// Invalid date format
    #[error("Invalid date format: {0}")]
    InvalidDate(String),

    /// Invalid URL format
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    /// Invalid language code
    #[error("Invalid language code: {0}")]
    InvalidLanguage(String),

    /// Invalid color code
    #[error("Invalid color code: {0}")]
    InvalidColor(String),

    /// Invalid email format
    #[error("Invalid email format: {0}")]
    InvalidEmail(String),

    /// Invalid file name
    #[error("Invalid file name: {0}")]
    InvalidFileName(String),

    /// Invalid content
    #[error("Invalid content: {0}")]
    InvalidContent(String),

    /// Invalid metadata
    #[error("Invalid metadata: {0}")]
    InvalidMetadata(String),

    /// Invalid size format
    #[error("Invalid size format: {0}")]
    InvalidSize(String),

    /// Invalid Twitter handle
    #[error("Invalid Twitter handle: {0}")]
    InvalidTwitterHandle(String),

    /// Content exceeds maximum length
    #[error("Content exceeds maximum length of {0} characters")]
    ContentTooLong(usize),

    /// Security-related validation error
    #[error("Security validation failed: {0}")]
    SecurityValidation(String),
}

/// Common validation functions for data models
pub mod validation {
    use std::{path::PathBuf, str::FromStr};

    use super::*;

    /// Validates a URL string
    pub fn validate_url(url: &str) -> Result<(), DataError> {
        if url.is_empty() {
            return Ok(());
        }

        Url::parse(url).map_err(|e| {
            DataError::InvalidUrl(format!("Invalid URL: {}", e))
        })?;

        // Check for unsafe characters
        if url.contains('<')
            || url.contains('>')
            || url.contains('"')
            || url.contains('\'')
        {
            return Err(DataError::InvalidUrl(
                "URL contains unsafe characters".to_string(),
            ));
        }

        Ok(())
    }

    /// Validates a date string in RFC3339 format
    pub fn validate_date(date: &str) -> Result<(), DataError> {
        if date.is_empty() {
            return Ok(());
        }

        OffsetDateTime::parse(date, &Rfc3339).map_err(|e| {
            DataError::InvalidDate(format!(
                "Invalid date format: {}",
                e
            ))
        })?;

        Ok(())
    }

    /// Validates language code against ISO 639-1
    pub fn validate_language_code(code: &str) -> Result<(), DataError> {
        if code.is_empty() {
            return Ok(());
        }

        if code.len() != 2
            || !code.chars().all(|c| c.is_ascii_lowercase())
        {
            return Err(DataError::InvalidLanguage(
                "Language code must be a 2-letter ISO 639-1 code"
                    .to_string(),
            ));
        }

        Ok(())
    }

    /// Validates color format (hex or RGB)
    pub fn validate_color(color: &str) -> Result<(), DataError> {
        if color.is_empty() {
            return Ok(());
        }

        let color = color.trim();

        let hex_valid = color.starts_with('#')
            && (color.len() == 4 || color.len() == 7)
            && color[1..].chars().all(|c| c.is_ascii_hexdigit());

        let rgb_valid = color.starts_with("rgb(")
            && color.ends_with(')')
            && color[4..color.len() - 1]
                .split(',')
                .filter_map(|s| s.trim().parse::<u8>().ok())
                .count()
                == 3;

        if !hex_valid && !rgb_valid {
            return Err(DataError::InvalidColor(
                "Color must be in #RGB, #RRGGBB, or rgb(r,g,b) format"
                    .to_string(),
            ));
        }

        Ok(())
    }

    /// Validates Twitter handle format
    pub fn validate_twitter_handle(
        handle: &str,
    ) -> Result<(), DataError> {
        if handle.is_empty() {
            return Ok(());
        }

        if !handle.starts_with('@') {
            return Err(DataError::InvalidTwitterHandle(
                "Twitter handle must start with @".to_string(),
            ));
        }

        let username = &handle[1..];
        if username.len() > 15
            || !username
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_')
        {
            return Err(DataError::InvalidTwitterHandle(
                "Invalid Twitter handle format".to_string(),
            ));
        }

        Ok(())
    }

    /// Validates image size format (e.g., "192x192")
    pub fn validate_image_size(size: &str) -> Result<(), DataError> {
        if size.is_empty() {
            return Ok(());
        }

        let parts: Vec<&str> = size.split('x').collect();
        if parts.len() != 2
            || parts[0].parse::<u32>().is_err()
            || parts[1].parse::<u32>().is_err()
        {
            return Err(DataError::InvalidSize(
                "Size must be in format WxH (e.g., 192x192)"
                    .to_string(),
            ));
        }

        Ok(())
    }

    /// Validates text length against a maximum limit
    ///
    /// # Arguments
    ///
    /// * `text` - The text to validate
    /// * `max_length` - Maximum allowed length
    /// * `field_name` - Name of the field being validated (for error messages)
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the text length is within limits, or an error otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use staticrux::models::data::validation::validate_text_length;
    ///
    /// let result = validate_text_length("test", 10, "example field");
    /// assert!(result.is_ok());
    ///
    /// let result = validate_text_length("too long text", 5, "example field");
    /// assert!(result.is_err());
    /// ```
    pub fn validate_text_length(
        text: &str,
        max_length: usize,
        _field_name: &str,
    ) -> Result<(), DataError> {
        let char_count = text.chars().count(); // Count Unicode characters instead of bytes
        if char_count > max_length {
            return Err(DataError::ContentTooLong(max_length));
        }
        Ok(())
    }

    /// Sanitizes a file path
    pub fn sanitize_path(path: &str) -> Result<PathBuf, DataError> {
        let path = PathBuf::from_str(path).map_err(|e| {
            DataError::InvalidFileName(format!(
                "Invalid path format: {}",
                e
            ))
        })?;

        // Prevent directory traversal
        if path.components().any(|c| c.as_os_str() == "..") {
            return Err(DataError::SecurityValidation(
                "Path contains directory traversal attempts"
                    .to_string(),
            ));
        }

        Ok(path)
    }
}

/// Represents the CNAME data for a website
#[derive(
    Debug, Default, PartialEq, Eq, Hash, Clone, Serialize, Deserialize,
)]
pub struct CnameData {
    /// The domain name for the website
    pub cname: String,
}

impl CnameData {
    /// Creates a new `CnameData` instance
    ///
    /// # Arguments
    ///
    /// * `cname` - The domain name for the website
    ///
    /// # Examples
    ///
    /// ```
    /// use staticrux::models::data::CnameData;
    ///
    /// let cname = CnameData::new("example.com".to_string());
    /// assert!(cname.validate().is_ok());
    /// ```
    pub fn new(cname: String) -> Self {
        CnameData { cname }
    }

    /// Validates the CNAME data
    ///
    /// Ensures the domain name:
    /// - Is not empty
    /// - Contains valid characters
    /// - Has proper format (name.tld)
    /// - Follows DNS naming rules
    pub fn validate(&self) -> Result<(), DataError> {
        if self.cname.is_empty() {
            return Err(DataError::InvalidDomain(
                "Domain name cannot be empty".to_string(),
            ));
        }

        let parts: Vec<&str> = self.cname.split('.').collect();
        if parts.len() < 2 {
            return Err(DataError::InvalidDomain(
                "Domain must have at least two parts".to_string(),
            ));
        }

        for part in parts {
            if part.is_empty() || part.len() > 63 {
                return Err(DataError::InvalidDomain(
                    "Domain parts must be 1-63 characters".to_string(),
                ));
            }

            if !part
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-')
            {
                return Err(DataError::InvalidDomain(
                    "Invalid characters in domain".to_string(),
                ));
            }

            if part.starts_with('-') || part.ends_with('-') {
                return Err(DataError::InvalidDomain(
                    "Domain parts cannot start or end with hyphens"
                        .to_string(),
                ));
            }
        }

        Ok(())
    }
}

/// Represents metadata for a single page
#[derive(
    Debug, Default, PartialEq, Eq, Hash, Clone, Serialize, Deserialize,
)]
pub struct PageData {
    /// The title of the page
    pub title: String,
    /// A brief description of the page content
    pub description: String,
    /// The publication date of the page
    pub date: String,
    /// The permanent link to the page
    pub permalink: String,
}

impl PageData {
    /// Creates a new PageData instance
    ///
    /// # Arguments
    ///
    /// * `title` - The page title
    /// * `description` - A brief description of the page
    /// * `date` - Publication date in RFC3339 format
    /// * `permalink` - Permanent link to the page
    ///
    /// # Examples
    ///
    /// ```
    /// use staticrux::models::data::PageData;
    ///
    /// let page = PageData::new(
    ///     "Welcome".to_string(),
    ///     "Welcome to my site".to_string(),
    ///     "2024-02-20T12:00:00Z".to_string(),
    ///     "/welcome".to_string()
    /// );
    /// assert!(page.validate().is_ok());
    /// ```
    pub fn new(
        title: String,
        description: String,
        date: String,
        permalink: String,
    ) -> Self {
        Self {
            title,
            description,
            date,
            permalink,
        }
    }

    /// Validates the page data
    ///
    /// Checks:
    /// - Title presence and length
    /// - Description presence
    /// - Date format (if present)
    /// - Permalink format
    pub fn validate(&self) -> Result<(), DataError> {
        // Title validation
        if self.title.is_empty() {
            return Err(DataError::MissingField("title".to_string()));
        }
        if self.title.len() > 200 {
            return Err(DataError::InvalidMetadata(
                "Title exceeds maximum length".to_string(),
            ));
        }

        // Description validation
        if self.description.is_empty() {
            return Err(DataError::MissingField(
                "description".to_string(),
            ));
        }

        // Date validation
        if !self.date.is_empty() {
            validation::validate_date(&self.date)?;
        }

        // Permalink validation
        if self.permalink.is_empty() {
            return Err(DataError::MissingField(
                "permalink".to_string(),
            ));
        }
        if !self.permalink.starts_with('/') {
            return Err(DataError::InvalidUrl(
                "Permalink must start with '/'".to_string(),
            ));
        }

        Ok(())
    }

    /// Returns a sanitized version of the title
    pub fn sanitized_title(&self) -> String {
        self.title
            .chars()
            .filter(|c| {
                c.is_ascii_alphanumeric() || c.is_ascii_whitespace()
            })
            .collect()
    }
}

impl fmt::Display for PageData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Title: {}\nDescription: {}\nDate: {}\nPermalink: {}",
            self.title, self.description, self.date, self.permalink
        )
    }
}

/// Represents the content and metadata of a file
#[derive(
    Debug, Default, PartialEq, Eq, Hash, Clone, Serialize, Deserialize,
)]
pub struct FileData {
    /// The name of the file
    pub name: String,
    /// The main content of the file
    pub content: String,
    /// The CNAME content, if applicable
    pub cname: String,
    /// The JSON representation of the file content
    pub json: String,
    /// The human-readable metadata
    pub human: String,
    /// Keywords associated with the file content
    pub keyword: String,
    /// The RSS feed content
    pub rss: String,
    /// The security.txt content
    pub security: String,
    /// The sitemap content
    pub sitemap: String,
    /// The news sitemap content
    pub sitemap_news: String,
    /// The robots.txt content
    pub txt: String,
}

impl FileData {
    /// Creates a new `FileData` instance
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the file
    /// * `content` - The main content of the file
    ///
    /// # Examples
    ///
    /// ```
    /// use staticrux::models::data::FileData;
    ///
    /// let file = FileData::new(
    ///     "index.md".to_string(),
    ///     "# Welcome\n\nWelcome to my site.".to_string()
    /// );
    /// assert!(file.validate().is_ok());
    /// ```
    pub fn new(name: String, content: String) -> Self {
        FileData {
            name,
            content,
            cname: String::new(),
            json: String::new(),
            human: String::new(),
            keyword: String::new(),
            rss: String::new(),
            security: String::new(),
            sitemap: String::new(),
            sitemap_news: String::new(),
            txt: String::new(),
        }
    }

    /// Validates the file data
    ///
    /// Checks:
    /// - File name format and extension
    /// - Content presence
    /// - JSON validity if present
    /// - URL formats in various fields
    pub fn validate(&self) -> Result<(), DataError> {
        // Validate file name
        if self.name.is_empty() {
            return Err(DataError::MissingField("name".to_string()));
        }

        // Sanitize and validate the file path
        validation::sanitize_path(&self.name)?;

        validation::validate_text_length(
            &self.name,
            MAX_SHORT_TEXT_LENGTH,
            "file name",
        )?;

        // Check file extension
        let valid_extensions = ["md", "html", "txt", "json", "xml"];
        let has_valid_ext = valid_extensions.iter().any(|&ext| {
            self.name.to_lowercase().ends_with(&format!(".{}", ext))
        });

        if !has_valid_ext {
            return Err(DataError::InvalidFileName(
                "Invalid file extension".to_string(),
            ));
        }

        // Validate content length
        validation::validate_text_length(
            &self.content,
            MAX_TEXT_LENGTH,
            "content",
        )?;

        // Validate content
        if self.content.is_empty() {
            return Err(DataError::MissingField("content".to_string()));
        }

        // Validate JSON if present
        if !self.json.is_empty() {
            serde_json::from_str::<serde_json::Value>(&self.json)
                .map_err(|e| {
                    DataError::InvalidContent(format!(
                        "Invalid JSON: {}",
                        e
                    ))
                })?;
        }

        // Validate URLs in various fields
        if !self.cname.is_empty() {
            CnameData::new(self.cname.clone()).validate()?;
        }

        // Validate other fields
        self.validate_auxiliary_content()?;

        Ok(())
    }

    /// Validates auxiliary content fields
    fn validate_auxiliary_content(&self) -> Result<(), DataError> {
        // Validate RSS content length
        validation::validate_text_length(
            &self.rss,
            MAX_TEXT_LENGTH,
            "RSS content",
        )?;

        // Validate sitemap content length
        validation::validate_text_length(
            &self.sitemap,
            MAX_TEXT_LENGTH,
            "sitemap content",
        )?;

        // Validate security.txt content length
        validation::validate_text_length(
            &self.security,
            MAX_TEXT_LENGTH,
            "security.txt content",
        )?;

        // Validate robots.txt content length
        validation::validate_text_length(
            &self.txt,
            MAX_SHORT_TEXT_LENGTH,
            "robots.txt content",
        )?;

        Ok(())
    }

    /// Returns the file extension
    pub fn extension(&self) -> Option<&str> {
        self.name.rsplit_once('.').map(|(_, ext)| ext)
    }

    /// Returns true if the file is a markdown file
    pub fn is_markdown(&self) -> bool {
        self.extension()
            .map(|ext| ext.eq_ignore_ascii_case("md"))
            .unwrap_or(false)
    }
}

/// Represents tag metadata for pages
#[derive(
    Debug, Default, PartialEq, Eq, Hash, Clone, Serialize, Deserialize,
)]
pub struct TagsData {
    /// Publication dates for tagged pages
    pub dates: String,
    /// Titles of tagged pages
    pub titles: String,
    /// Descriptions of tagged pages
    pub descriptions: String,
    /// Permalinks to tagged pages
    pub permalinks: String,
    /// Keywords associated with tagged pages
    pub keywords: String,
}

impl TagsData {
    /// Creates a new `TagsData` instance
    ///
    /// # Arguments
    ///
    /// * `dates` - Publication dates
    /// * `titles` - Page titles
    /// * `descriptions` - Page descriptions
    /// * `permalinks` - Page permalinks
    /// * `keywords` - Associated keywords
    pub fn new(
        dates: String,
        titles: String,
        descriptions: String,
        permalinks: String,
        keywords: String,
    ) -> Self {
        Self {
            dates,
            titles,
            descriptions,
            permalinks,
            keywords,
        }
    }

    /// Validates the tags data
    pub fn validate(&self) -> Result<(), DataError> {
        // Validate dates if present
        if !self.dates.is_empty() {
            // Split multiple dates and validate each
            for date in self.dates.split(',') {
                validation::validate_date(date.trim())?;
            }
        }

        // Validate permalinks if present
        if !self.permalinks.is_empty() {
            for permalink in self.permalinks.split(',') {
                let permalink = permalink.trim();
                if !permalink.starts_with('/') {
                    return Err(DataError::InvalidUrl(format!(
                        "Invalid permalink format: {}",
                        permalink
                    )));
                }
            }
        }

        // Validate field lengths
        validation::validate_text_length(
            &self.titles,
            MAX_TEXT_LENGTH,
            "titles",
        )?;
        validation::validate_text_length(
            &self.descriptions,
            MAX_TEXT_LENGTH,
            "descriptions",
        )?;
        validation::validate_text_length(
            &self.keywords,
            MAX_METADATA_LENGTH,
            "keywords",
        )?;

        Ok(())
    }

    /// Returns keywords as a vector
    pub fn keywords_list(&self) -> Vec<String> {
        self.keywords
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }
}

/// Represents data for the service worker file
#[derive(
    Debug, Default, PartialEq, Eq, Hash, Clone, Serialize, Deserialize,
)]
pub struct SwFileData {
    /// URL of the offline page
    pub offline_page_url: String,
}

impl SwFileData {
    /// Creates a new `SwFileData` instance
    ///
    /// # Arguments
    ///
    /// * `offline_page_url` - The URL of the offline page
    pub fn new(offline_page_url: String) -> Self {
        SwFileData { offline_page_url }
    }

    /// Validates the service worker data
    pub fn validate(&self) -> Result<(), DataError> {
        if self.offline_page_url.is_empty() {
            return Err(DataError::MissingField(
                "offline_page_url".to_string(),
            ));
        }

        if !self.offline_page_url.starts_with('/') {
            return Err(DataError::InvalidUrl(
                "Offline page URL must start with '/'".to_string(),
            ));
        }

        Ok(())
    }
}

/// Represents data for an icon
#[derive(
    Debug, Default, PartialEq, Eq, Hash, Clone, Serialize, Deserialize,
)]
pub struct IconData {
    /// The purpose of the icon (e.g., "maskable", "any")
    pub purpose: Option<String>,
    /// The sizes of the icon (e.g., "192x192")
    pub sizes: String,
    /// The source URL of the icon
    pub src: String,
    /// The MIME type of the icon
    pub icon_type: Option<String>,
}

impl IconData {
    /// Creates a new `IconData` instance
    ///
    /// # Arguments
    ///
    /// * `src` - The source URL of the icon
    /// * `sizes` - The sizes of the icon
    pub fn new(src: String, sizes: String) -> Self {
        IconData {
            purpose: None,
            sizes,
            src,
            icon_type: None,
        }
    }

    /// Validates the icon data
    pub fn validate(&self) -> Result<(), DataError> {
        // Validate source URL
        if self.src.is_empty() {
            return Err(DataError::MissingField("src".to_string()));
        }
        validation::validate_url(&self.src)?;

        // Validate sizes
        if self.sizes.is_empty() {
            return Err(DataError::MissingField("sizes".to_string()));
        }
        validation::validate_image_size(&self.sizes)?;

        // Validate purpose if present
        if let Some(purpose) = &self.purpose {
            let valid_purposes = ["maskable", "any"];
            if !valid_purposes.contains(&purpose.as_str()) {
                return Err(DataError::InvalidMetadata(
                    "Invalid icon purpose".to_string(),
                ));
            }
        }

        // Validate MIME type if present
        if let Some(mime_type) = &self.icon_type {
            let valid_types = [
                "image/png",
                "image/jpeg",
                "image/webp",
                "image/svg+xml",
            ];
            if !valid_types.contains(&mime_type.as_str()) {
                return Err(DataError::InvalidMetadata(
                    "Invalid icon MIME type".to_string(),
                ));
            }
        }

        Ok(())
    }
}

/// Represents data for the web app manifest
#[derive(
    Debug, Default, PartialEq, Eq, Hash, Clone, Serialize, Deserialize,
)]
pub struct ManifestData {
    /// The background color of the web app
    pub background_color: String,
    /// A description of the web app
    pub description: String,
    /// The display mode of the web app
    pub display: String,
    /// Icons associated with the web app
    pub icons: Vec<IconData>,
    /// The name of the web app
    pub name: String,
    /// The orientation of the web app
    pub orientation: String,
    /// The scope of the web app
    pub scope: String,
    /// The short name of the web app
    pub short_name: String,
    /// The start URL of the web app
    pub start_url: String,
    /// The theme color of the web app
    pub theme_color: String,
}

impl ManifestData {
    /// Creates a new `ManifestData` instance with default values
    pub fn new() -> Self {
        ManifestData::default()
    }

    /// Validates the manifest data
    pub fn validate(&self) -> Result<(), DataError> {
        // Name length validation
        if self.name.len() > MAX_MANIFEST_NAME_LENGTH {
            return Err(DataError::InvalidMetadata(format!(
                "Name exceeds maximum length of {} characters",
                MAX_MANIFEST_NAME_LENGTH
            )));
        }

        // Short name length validation
        if self.short_name.len() > MAX_MANIFEST_SHORT_NAME_LENGTH {
            return Err(DataError::InvalidMetadata(format!(
                "Short name exceeds maximum length of {} characters",
                MAX_MANIFEST_SHORT_NAME_LENGTH
            )));
        }

        // Description length validation
        validation::validate_text_length(
            &self.description,
            MAX_METADATA_LENGTH,
            "manifest description",
        )?;

        // Then do the other validations
        // Validate colors
        if !self.background_color.is_empty() {
            validation::validate_color(&self.background_color)?;
        }
        if !self.theme_color.is_empty() {
            validation::validate_color(&self.theme_color)?;
        }

        // Validate display mode
        if !self.display.is_empty() {
            let valid_displays =
                ["fullscreen", "standalone", "minimal-ui", "browser"];
            if !valid_displays.contains(&self.display.as_str()) {
                return Err(DataError::InvalidMetadata(
                    "Invalid display mode".to_string(),
                ));
            }
        }

        // Validate orientation
        if !self.orientation.is_empty() {
            let valid_orientations = [
                "any",
                "natural",
                "landscape",
                "portrait",
                "portrait-primary",
                "portrait-secondary",
                "landscape-primary",
                "landscape-secondary",
            ];
            if !valid_orientations.contains(&self.orientation.as_str())
            {
                return Err(DataError::InvalidMetadata(
                    "Invalid orientation".to_string(),
                ));
            }
        }

        // Validate scope and start_url
        if !self.scope.is_empty() && !self.scope.starts_with('/') {
            return Err(DataError::InvalidUrl(
                "Scope must start with '/'".to_string(),
            ));
        }
        if !self.start_url.is_empty()
            && !self.start_url.starts_with('/')
        {
            return Err(DataError::InvalidUrl(
                "Start URL must start with '/'".to_string(),
            ));
        }

        // Validate icons
        for icon in &self.icons {
            icon.validate()?;
        }

        // Validate name lengths
        if self.name.len() > 45 {
            return Err(DataError::InvalidMetadata(
                "Name exceeds maximum length of 45 characters"
                    .to_string(),
            ));
        }
        if self.short_name.len() > 12 {
            return Err(DataError::InvalidMetadata(
                "Short name exceeds maximum length of 12 characters"
                    .to_string(),
            ));
        }

        Ok(())
    }
}

/// Represents data for the news sitemap
#[derive(
    Debug, Default, PartialEq, Eq, Hash, Clone, Serialize, Deserialize,
)]
pub struct NewsData {
    /// The genres of the news content
    pub news_genres: String,
    /// Keywords associated with the news content
    pub news_keywords: String,
    /// The language of the news content
    pub news_language: String,
    /// The URL of the news image
    pub news_image_loc: String,
    /// The URL of the news content
    pub news_loc: String,
    /// The publication date of the news content
    pub news_publication_date: String,
    /// The name of the news publication
    pub news_publication_name: String,
    /// The title of the news content
    pub news_title: String,
}

impl NewsData {
    /// Creates a new `NewsData` instance
    pub fn new(data: NewsData) -> Self {
        data
    }

    /// Creates a new `NewsData` instance with default values
    pub fn create_default() -> Self {
        Default::default()
    }

    /// Validates the news data
    pub fn validate(&self) -> Result<(), DataError> {
        // Validate URLs
        if !self.news_image_loc.is_empty() {
            validation::validate_url(&self.news_image_loc)?;
        }
        if !self.news_loc.is_empty() {
            validation::validate_url(&self.news_loc)?;
        }

        // Validate language
        if !self.news_language.is_empty() {
            validation::validate_language_code(&self.news_language)?;
        }

        // Validate publication date
        if !self.news_publication_date.is_empty() {
            validation::validate_date(&self.news_publication_date)?;
        }

        // Validate genres
        if !self.news_genres.is_empty() {
            let valid_genres = [
                "PressRelease",
                "Satire",
                "Blog",
                "OpEd",
                "Opinion",
                "UserGenerated",
            ];
            for genre in self.news_genres.split(',') {
                let genre = genre.trim();
                if !valid_genres.contains(&genre) {
                    return Err(DataError::InvalidMetadata(format!(
                        "Invalid news genre: {}",
                        genre
                    )));
                }
            }
        }

        // Validate title length
        if !self.news_title.is_empty() && self.news_title.len() > 200 {
            return Err(DataError::InvalidMetadata(
                "News title exceeds maximum length".to_string(),
            ));
        }

        Ok(())
    }

    /// Returns the genres as a vector
    pub fn genres_list(&self) -> Vec<String> {
        self.news_genres
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }
}

/// Represents options for the news sitemap visit function
#[derive(
    Debug, Default, PartialEq, Eq, Hash, Clone, Serialize, Deserialize,
)]
pub struct NewsVisitOptions<'a> {
    /// The base URL of the news website
    pub base_url: &'a str,
    /// The genres of the news content
    pub news_genres: &'a str,
    /// Keywords associated with the news content
    pub news_keywords: &'a str,
    /// The language of the news content
    pub news_language: &'a str,
    /// The publication date of the news content
    pub news_publication_date: &'a str,
    /// The name of the news publication
    pub news_publication_name: &'a str,
    /// The title of the news content
    pub news_title: &'a str,
}

impl<'a> NewsVisitOptions<'a> {
    /// Creates a new `NewsVisitOptions` instance
    pub fn new(
        base_url: &'a str,
        news_genres: &'a str,
        news_keywords: &'a str,
        news_language: &'a str,
        news_publication_date: &'a str,
        news_publication_name: &'a str,
        news_title: &'a str,
    ) -> Self {
        Self {
            base_url,
            news_genres,
            news_keywords,
            news_language,
            news_publication_date,
            news_publication_name,
            news_title,
        }
    }

    /// Validates the news visit options
    pub fn validate(&self) -> Result<(), DataError> {
        // Validate base URL
        validation::validate_url(self.base_url)?;

        // Validate language code
        validation::validate_language_code(self.news_language)?;

        // Validate publication date
        validation::validate_date(self.news_publication_date)?;

        // Validate genres
        if !self.news_genres.is_empty() {
            let valid_genres = [
                "PressRelease",
                "Satire",
                "Blog",
                "OpEd",
                "Opinion",
                "UserGenerated",
            ];
            for genre in self.news_genres.split(',') {
                let genre = genre.trim();
                if !valid_genres.contains(&genre) {
                    return Err(DataError::InvalidMetadata(format!(
                        "Invalid news genre: {}",
                        genre
                    )));
                }
            }
        }

        // Validate required fields presence
        if self.news_publication_name.is_empty() {
            return Err(DataError::MissingField(
                "news_publication_name".to_string(),
            ));
        }
        if self.news_title.is_empty() {
            return Err(DataError::MissingField(
                "news_title".to_string(),
            ));
        }

        Ok(())
    }
}

/// Represents data for the humans.txt file
#[derive(
    Debug, Default, PartialEq, Eq, Hash, Clone, Serialize, Deserialize,
)]
pub struct HumansData {
    /// The name of the author
    pub author: String,
    /// The website of the author
    pub author_website: String,
    /// The Twitter handle of the author
    pub author_twitter: String,
    /// The location of the author
    pub author_location: String,
    /// Acknowledgements or thanks
    pub thanks: String,
    /// The date when the site was last updated
    pub site_last_updated: String,
    /// The standards followed by the site
    pub site_standards: String,
    /// The components used in the site
    pub site_components: String,
    /// The software used to build the site
    pub site_software: String,
}

impl HumansData {
    /// Creates a new `HumansData` instance
    ///
    /// # Arguments
    ///
    /// * `author` - The name of the author
    /// * `thanks` - Acknowledgements or thanks
    pub fn new(author: String, thanks: String) -> Self {
        HumansData {
            author,
            author_website: String::new(),
            author_twitter: String::new(),
            author_location: String::new(),
            thanks,
            site_last_updated: String::new(),
            site_standards: String::new(),
            site_components: String::new(),
            site_software: String::new(),
        }
    }

    /// Validates the humans.txt data
    pub fn validate(&self) -> Result<(), DataError> {
        // Validate required fields
        if self.author.is_empty() {
            return Err(DataError::MissingField("author".to_string()));
        }

        // Validate text lengths
        validation::validate_text_length(
            &self.author,
            MAX_SHORT_TEXT_LENGTH,
            "author name",
        )?;

        validation::validate_text_length(
            &self.author_location,
            MAX_SHORT_TEXT_LENGTH,
            "author location",
        )?;
        validation::validate_text_length(
            &self.thanks,
            MAX_METADATA_LENGTH,
            "thanks",
        )?;
        validation::validate_text_length(
            &self.site_standards,
            MAX_METADATA_LENGTH,
            "site standards",
        )?;
        validation::validate_text_length(
            &self.site_components,
            MAX_METADATA_LENGTH,
            "site components",
        )?;
        validation::validate_text_length(
            &self.site_software,
            MAX_METADATA_LENGTH,
            "site software",
        )?;

        // Validate website URL if present
        if !self.author_website.is_empty() {
            validation::validate_url(&self.author_website)?;
        }

        // Validate Twitter handle if present
        if !self.author_twitter.is_empty() {
            validation::validate_twitter_handle(&self.author_twitter)?;
        }

        // Validate last updated date if present
        if !self.site_last_updated.is_empty() {
            validation::validate_date(&self.site_last_updated)?;
        }

        Ok(())
    }

    /// Returns true if all optional fields are empty
    pub fn is_minimal(&self) -> bool {
        self.author_website.is_empty()
            && self.author_twitter.is_empty()
            && self.author_location.is_empty()
            && self.site_standards.is_empty()
            && self.site_components.is_empty()
            && self.site_software.is_empty()
    }
}

/// Represents groups of meta tags for different platforms and categories
#[derive(
    Debug, Default, PartialEq, Eq, Hash, Clone, Serialize, Deserialize,
)]
pub struct MetaTagGroups {
    /// Meta tags specific to Apple devices
    pub apple: String,
    /// Primary meta tags, such as author, description, etc.
    pub primary: String,
    /// Open Graph meta tags, mainly used for social media
    pub og: String,
    /// Microsoft-specific meta tags
    pub ms: String,
    /// Twitter-specific meta tags
    pub twitter: String,
}

impl MetaTagGroups {
    /// Creates a new `MetaTagGroups` instance with default values
    pub fn new() -> Self {
        MetaTagGroups::default()
    }

    /// Returns the value for the given key, if it exists
    pub fn get(&self, key: &str) -> Option<&String> {
        match key {
            "apple" => Some(&self.apple),
            "primary" => Some(&self.primary),
            "og" => Some(&self.og),
            "ms" => Some(&self.ms),
            "twitter" => Some(&self.twitter),
            _ => None,
        }
    }

    /// Returns true if all fields are empty
    pub fn is_empty(&self) -> bool {
        self.apple.is_empty()
            && self.primary.is_empty()
            && self.og.is_empty()
            && self.ms.is_empty()
            && self.twitter.is_empty()
    }

    /// Validates meta tag content
    pub fn validate(&self) -> Result<(), DataError> {
        // Validate Open Graph tags
        if !self.og.is_empty() {
            // Check for required OG properties
            let required_og = ["title", "type", "url", "description"];
            for prop in required_og {
                if !self.og.contains(&format!("og:{}", prop)) {
                    return Err(DataError::InvalidMetadata(format!(
                        "Missing required OG property: {}",
                        prop
                    )));
                }
            }
        }

        // Validate Twitter card tags
        if !self.twitter.is_empty()
            && !self.twitter.contains("twitter:card")
        {
            return Err(DataError::InvalidMetadata(
                "Missing required Twitter card type".to_string(),
            ));
        }

        Ok(())
    }
}

/// Represents data for the robots.txt file
#[derive(
    Debug, Default, PartialEq, Eq, Hash, Clone, Serialize, Deserialize,
)]
pub struct TxtData {
    /// The permalink of the website
    pub permalink: String,
}

impl TxtData {
    /// Creates a new `TxtData` instance
    ///
    /// # Arguments
    ///
    /// * `permalink` - The permalink of the website
    pub fn new(permalink: String) -> Self {
        TxtData { permalink }
    }

    /// Validates the robots.txt data
    pub fn validate(&self) -> Result<(), DataError> {
        if self.permalink.is_empty() {
            return Err(DataError::MissingField(
                "permalink".to_string(),
            ));
        }

        validation::validate_url(&self.permalink)?;

        Ok(())
    }

    /// Generates the robots.txt content
    pub fn generate_content(&self) -> String {
        format!(
            "User-agent: *\nAllow: /\nSitemap: {}/sitemap.xml",
            self.permalink.trim_end_matches('/')
        )
    }
}

/// Represents data for the RSS feed
#[derive(
    Debug, Default, PartialEq, Eq, Hash, Clone, Serialize, Deserialize,
)]
pub struct RssData {
    /// The Atom link of the RSS feed
    pub atom_link: String,
    /// The author of the RSS feed
    pub author: String,
    /// The category of the RSS feed
    pub category: String,
    /// The copyright notice for the content of the feed
    pub copyright: String,
    /// The description of the RSS feed
    pub description: String,
    /// The documentation URL for the RSS feed format
    pub docs: String,
    /// The generator of the RSS feed
    pub generator: String,
    /// The image URL for the RSS feed
    pub image: String,
    /// The unique identifier (GUID) of an RSS item
    pub item_guid: String,
    /// The description of an RSS item
    pub item_description: String,
    /// The link to an RSS item
    pub item_link: String,
    /// The publication date of an RSS item
    pub item_pub_date: String,
    /// The title of an RSS item
    pub item_title: String,
    /// The language of the RSS feed
    pub language: String,
    /// The last build date of the RSS feed
    pub last_build_date: String,
    /// The link to the website associated with the RSS feed
    pub link: String,
    /// The managing editor of the RSS feed
    pub managing_editor: String,
    /// The publication date of the RSS feed
    pub pub_date: String,
    /// The title of the RSS feed
    pub title: String,
    /// Time To Live: the number of minutes the feed should be cached
    pub ttl: String,
    /// The webmaster of the RSS feed
    pub webmaster: String,
}

impl RssData {
    /// Creates a new `RssData` instance with default values
    pub fn new() -> Self {
        RssData::default()
    }

    /// Sets the value of a field
    ///
    /// # Arguments
    ///
    /// * `field` - The name of the field to set
    /// * `value` - The value to set for the field
    pub fn set(&mut self, field: &str, value: String) {
        match field {
            "atom_link" => self.atom_link = value,
            "author" => self.author = value,
            "category" => self.category = value,
            "copyright" => self.copyright = value,
            "description" => self.description = value,
            "docs" => self.docs = value,
            "generator" => self.generator = value,
            "image" => self.image = value,
            "item_guid" => self.item_guid = value,
            "item_description" => self.item_description = value,
            "item_link" => self.item_link = value,
            "item_pub_date" => self.item_pub_date = value,
            "item_title" => self.item_title = value,
            "language" => self.language = value,
            "last_build_date" => self.last_build_date = value,
            "link" => self.link = value,
            "managing_editor" => self.managing_editor = value,
            "pub_date" => self.pub_date = value,
            "title" => self.title = value,
            "ttl" => self.ttl = value,
            "webmaster" => self.webmaster = value,
            _ => (),
        }
    }

    /// Validates the RSS data
    pub fn validate(&self) -> Result<(), DataError> {
        // Validate required fields
        let required_fields = [
            ("title", &self.title),
            ("link", &self.link),
            ("description", &self.description),
        ];

        for (field, value) in required_fields {
            if value.is_empty() {
                return Err(DataError::MissingField(field.to_string()));
            }
        }

        // Validate URLs
        for url in
            [&self.link, &self.atom_link, &self.docs, &self.image]
                .iter()
                .filter(|u| !u.is_empty())
        {
            validation::validate_url(url)?;
        }

        // Validate dates
        for date in
            [&self.pub_date, &self.last_build_date, &self.item_pub_date]
                .iter()
                .filter(|d| !d.is_empty())
        {
            validation::validate_date(date)?;
        }

        // Validate TTL if present
        if !self.ttl.is_empty() {
            self.ttl.parse::<u32>().map_err(|_| {
                DataError::InvalidMetadata(
                    "Invalid TTL value".to_string(),
                )
            })?;
        }

        // Validate language code if present
        if !self.language.is_empty() {
            validation::validate_language_code(&self.language)?;
        }

        Ok(())
    }
}

/// Represents a single meta tag
#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
pub struct MetaTag {
    /// The name of the meta tag
    pub name: String,
    /// The content of the meta tag
    pub value: String,
}

impl MetaTag {
    /// Creates a new `MetaTag` instance
    pub fn new(name: String, value: String) -> Self {
        MetaTag { name, value }
    }

    /// Validates the meta tag
    pub fn validate(&self) -> Result<(), DataError> {
        if self.name.is_empty() {
            return Err(DataError::MissingField(
                "meta tag name".to_string(),
            ));
        }

        // Sanitize name and value
        if self.name.contains('"')
            || self.name.contains('>')
            || self.name.contains('<')
        {
            return Err(DataError::InvalidMetadata(
                "Meta tag name contains invalid characters".to_string(),
            ));
        }

        if self.value.contains('"')
            || self.value.contains('>')
            || self.value.contains('<')
        {
            return Err(DataError::InvalidMetadata(
                "Meta tag value contains invalid characters"
                    .to_string(),
            ));
        }

        Ok(())
    }

    /// Generates the HTML representation of the meta tag
    pub fn generate(&self) -> String {
        format!(
            r#"<meta name="{}" content="{}">"#,
            self.name, self.value
        )
    }

    /// Generates a complete list of metatags in HTML format
    pub fn generate_metatags(metatags: &[MetaTag]) -> String {
        metatags.iter().map(MetaTag::generate).collect()
    }
}

/// Represents data for the security.txt file according to RFC 9116
#[derive(
    Debug, Default, PartialEq, Eq, Hash, Clone, Serialize, Deserialize,
)]
pub struct SecurityData {
    /// Required: One or more URIs or email addresses for reporting vulnerabilities
    pub contact: Vec<String>,
    /// Required: Expiration date for the security.txt data (in ISO 8601 format)
    pub expires: String,
    /// Optional: Link to a page where security researchers are recognized
    pub acknowledgments: String,
    /// Optional: Preferred languages for security reports (comma-separated language tags)
    pub preferred_languages: String,
    /// Optional: Canonical URI where this security.txt file is located
    pub canonical: String,
    /// Optional: Link to the security policy
    pub policy: String,
    /// Optional: Link to security-related job positions
    pub hiring: String,
    /// Optional: Link to an encryption key
    pub encryption: String,
}

impl SecurityData {
    /// Creates a new SecurityData instance with required fields
    ///
    /// # Arguments
    ///
    /// * contact - Vector of contact URIs or email addresses  
    /// * expires - Expiration date in ISO 8601 format
    ///
    /// # Examples
    ///
    /// ```
    /// use staticrux::models::data::SecurityData;
    ///
    /// let security_data = SecurityData::new(
    ///     vec!["https://example.com/security".to_string()],
    ///     "2024-12-31T23:59:59Z".to_string()
    /// );
    /// ```
    pub fn new(contact: Vec<String>, expires: String) -> Self {
        SecurityData {
            contact,
            expires,
            acknowledgments: String::new(),
            preferred_languages: String::new(),
            canonical: String::new(),
            policy: String::new(),
            hiring: String::new(),
            encryption: String::new(),
        }
    }

    /// Creates a new SecurityData instance with all fields empty
    pub fn create_default() -> Self {
        Default::default()
    }

    /// Validates if the required fields are properly set
    ///
    /// # Returns
    ///
    /// true if both contact and expires fields are non-empty
    pub fn is_valid(&self) -> bool {
        !self.contact.is_empty() && !self.expires.is_empty()
    }

    /// Returns a list of all non-empty fields
    ///
    /// # Returns
    ///
    /// A vector of field names that have content
    pub fn get_populated_fields(&self) -> Vec<String> {
        let mut fields = Vec::new();

        if !self.contact.is_empty() {
            fields.push("contact".to_string());
        }
        if !self.expires.is_empty() {
            fields.push("expires".to_string());
        }
        if !self.acknowledgments.is_empty() {
            fields.push("acknowledgments".to_string());
        }
        if !self.preferred_languages.is_empty() {
            fields.push("preferred-languages".to_string());
        }
        if !self.canonical.is_empty() {
            fields.push("canonical".to_string());
        }
        if !self.policy.is_empty() {
            fields.push("policy".to_string());
        }
        if !self.hiring.is_empty() {
            fields.push("hiring".to_string());
        }
        if !self.encryption.is_empty() {
            fields.push("encryption".to_string());
        }

        fields
    }

    /// Validates the security.txt data according to RFC 9116
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if validation passes, or appropriate error if validation fails.
    pub fn validate(&self) -> Result<(), DataError> {
        // Validate required fields
        if self.contact.is_empty() {
            return Err(DataError::MissingField("contact".to_string()));
        }
        if self.expires.is_empty() {
            return Err(DataError::MissingField("expires".to_string()));
        }

        // Validate contact URLs/emails
        for contact in &self.contact {
            // Allow mailto: and https:// URLs
            if contact.starts_with("mailto:") {
                // Basic email validation for mailto: URLs
                if !contact.contains('@') {
                    return Err(DataError::InvalidUrl(
                        "Invalid email in contact".to_string(),
                    ));
                }
            } else {
                validation::validate_url(contact)?;
            }
        }

        // Validate expiration date
        validation::validate_date(&self.expires)?;

        // Validate optional URLs if present
        if !self.acknowledgments.is_empty() {
            validation::validate_url(&self.acknowledgments)?;
        }
        if !self.canonical.is_empty() {
            validation::validate_url(&self.canonical)?;
        }
        if !self.policy.is_empty() {
            validation::validate_url(&self.policy)?;
        }
        if !self.hiring.is_empty() {
            validation::validate_url(&self.hiring)?;
        }
        if !self.encryption.is_empty() {
            validation::validate_url(&self.encryption)?;
        }

        // Validate preferred languages
        if !self.preferred_languages.is_empty() {
            for lang in self.preferred_languages.split(',') {
                validation::validate_language_code(lang.trim())?;
            }
        }

        // Validate text lengths
        for contact in &self.contact {
            validation::validate_text_length(
                contact,
                MAX_SHORT_TEXT_LENGTH,
                "contact",
            )?;
        }
        validation::validate_text_length(
            &self.acknowledgments,
            MAX_SHORT_TEXT_LENGTH,
            "acknowledgments",
        )?;
        validation::validate_text_length(
            &self.canonical,
            MAX_SHORT_TEXT_LENGTH,
            "canonical",
        )?;
        validation::validate_text_length(
            &self.policy,
            MAX_SHORT_TEXT_LENGTH,
            "policy",
        )?;
        validation::validate_text_length(
            &self.hiring,
            MAX_SHORT_TEXT_LENGTH,
            "hiring",
        )?;
        validation::validate_text_length(
            &self.encryption,
            MAX_SHORT_TEXT_LENGTH,
            "encryption",
        )?;
        validation::validate_text_length(
            &self.preferred_languages,
            MAX_SHORT_TEXT_LENGTH,
            "preferred languages",
        )?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_meta_tag_validation() {
        let valid = MetaTag::new(
            "description".to_string(),
            "A valid description".to_string(),
        );
        assert!(valid.validate().is_ok());

        let invalid = MetaTag::new(
            "description\"".to_string(),
            "Invalid \"quote\"".to_string(),
        );
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn test_meta_tag_generation() {
        let tag = MetaTag::new(
            "description".to_string(),
            "Test description".to_string(),
        );
        assert_eq!(
            tag.generate(),
            r#"<meta name="description" content="Test description">"#
        );
    }

    #[test]
    fn test_rss_data_validation() {
        let mut rss = RssData::new();
        assert!(rss.validate().is_err()); // Missing required fields

        rss.title = "Test Feed".to_string();
        rss.link = "https://example.com".to_string();
        rss.description = "Test description".to_string();
        assert!(rss.validate().is_ok());
    }

    #[test]
    fn test_humans_data_validation() {
        let valid = HumansData::new(
            "Author Name".to_string(),
            "Thanks".to_string(),
        );
        assert!(valid.validate().is_ok());

        let invalid =
            HumansData::new("".to_string(), "Thanks".to_string());
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn test_cname_data() {
        // Test valid cases
        let valid_cases = vec![
            "example.com",
            "sub.example.com",
            "my-site.example.com",
            "example.co.uk",
        ];

        for domain in valid_cases {
            let cname = CnameData::new(domain.to_string());
            assert!(
                cname.validate().is_ok(),
                "Domain should be valid: {}",
                domain
            );
        }

        // Test invalid cases
        let binding = "a".repeat(64);
        let invalid_cases = vec![
            "",             // Empty
            "invalid",      // No TLD
            "-example.com", // Starts with hyphen
            "example-.com", // Ends with hyphen
            "ex ample.com", // Contains space
            "example..com", // Double dots
            &binding,       // Too long label
            "exam@ple.com", // Invalid character
        ];

        for domain in invalid_cases {
            let cname = CnameData::new(domain.to_string());
            assert!(
                cname.validate().is_err(),
                "Domain should be invalid: {}",
                domain
            );
        }
    }

    #[test]
    fn test_page_data() {
        // Test valid case
        let valid_page = PageData::new(
            "Test Title".to_string(),
            "Test Description".to_string(),
            "2024-02-20T12:00:00Z".to_string(),
            "/test-page".to_string(),
        );
        assert!(valid_page.validate().is_ok());

        // Test required fields
        let missing_title = PageData::new(
            "".to_string(),
            "Description".to_string(),
            "2024-02-20T12:00:00Z".to_string(),
            "/page".to_string(),
        );
        assert!(matches!(
            missing_title.validate(),
            Err(DataError::MissingField(field)) if field == "title"
        ));

        // Test invalid date
        let invalid_date = PageData::new(
            "Title".to_string(),
            "Description".to_string(),
            "invalid-date".to_string(),
            "/page".to_string(),
        );
        assert!(matches!(
            invalid_date.validate(),
            Err(DataError::InvalidDate(_))
        ));

        // Test invalid permalink
        let invalid_permalink = PageData::new(
            "Title".to_string(),
            "Description".to_string(),
            "2024-02-20T12:00:00Z".to_string(),
            "invalid-path".to_string(),
        );
        assert!(matches!(
            invalid_permalink.validate(),
            Err(DataError::InvalidUrl(_))
        ));
    }

    #[test]
    fn test_file_data() {
        // Test valid case
        let valid_file = FileData::new(
            "test.md".to_string(),
            "# Test Content".to_string(),
        );
        assert!(valid_file.validate().is_ok());

        // Test empty fields
        let empty_file = FileData::new("".to_string(), "".to_string());
        assert!(matches!(
            empty_file.validate(),
            Err(DataError::MissingField(field)) if field == "name"
        ));

        // Test invalid extension
        let invalid_ext = FileData::new(
            "test.invalid".to_string(),
            "content".to_string(),
        );
        assert!(matches!(
            invalid_ext.validate(),
            Err(DataError::InvalidFileName(_))
        ));

        // Test extension helper
        assert_eq!(valid_file.extension(), Some("md"));
        assert!(valid_file.is_markdown());
    }

    #[test]
    fn test_tags_data() {
        // Test valid case
        let valid_tags = TagsData::new(
            "2024-02-20T12:00:00Z".to_string(),
            "Test Title".to_string(),
            "Test Description".to_string(),
            "/test-page".to_string(),
            "test, example".to_string(),
        );
        assert!(valid_tags.validate().is_ok());

        // Test invalid date
        let invalid_date = TagsData::new(
            "invalid-date".to_string(),
            "Title".to_string(),
            "Description".to_string(),
            "/page".to_string(),
            "tags".to_string(),
        );
        assert!(invalid_date.validate().is_err());

        // Test invalid permalink
        let invalid_permalink = TagsData::new(
            "2024-02-20T12:00:00Z".to_string(),
            "Title".to_string(),
            "Description".to_string(),
            "invalid-path".to_string(),
            "tags".to_string(),
        );
        assert!(invalid_permalink.validate().is_err());

        // Test keywords list
        let tags = TagsData::new(
            "".to_string(),
            "".to_string(),
            "".to_string(),
            "".to_string(),
            "tag1, tag2, tag3".to_string(),
        );
        assert_eq!(tags.keywords_list(), vec!["tag1", "tag2", "tag3"]);
    }

    #[test]
    fn test_sw_file_data() {
        // Test valid case
        let valid_sw = SwFileData::new("/offline.html".to_string());
        assert!(valid_sw.validate().is_ok());

        // Test empty URL
        let empty_sw = SwFileData::new("".to_string());
        assert!(matches!(
            empty_sw.validate(),
            Err(DataError::MissingField(field)) if field == "offline_page_url"
        ));

        // Test invalid URL format
        let invalid_sw = SwFileData::new("invalid-url".to_string());
        assert!(matches!(
            invalid_sw.validate(),
            Err(DataError::InvalidUrl(_))
        ));
    }

    #[test]
    fn test_icon_data() {
        // Test valid case
        let valid_icon = IconData::new(
            "https://example.com/icon.png".to_string(), // Changed to a fully qualified URL
            "192x192".to_string(),
        );
        assert!(
            valid_icon.validate().is_ok(),
            "Expected valid icon validation to succeed"
        );

        // Test invalid size format
        let invalid_size = IconData::new(
            "https://example.com/icon.png".to_string(),
            "invalid".to_string(),
        );
        assert!(
            matches!(
                invalid_size.validate(),
                Err(DataError::InvalidSize(_))
            ),
            "Expected invalid size format to fail validation"
        );

        // Test invalid purpose
        let mut invalid_purpose = IconData::new(
            "https://example.com/icon.png".to_string(),
            "192x192".to_string(),
        );
        invalid_purpose.purpose = Some("invalid".to_string());
        assert!(
            matches!(
                invalid_purpose.validate(),
                Err(DataError::InvalidMetadata(_))
            ),
            "Expected invalid purpose to fail validation"
        );

        // Test valid purpose
        let mut valid_purpose = IconData::new(
            "https://example.com/icon.png".to_string(),
            "192x192".to_string(),
        );
        valid_purpose.purpose = Some("maskable".to_string());
        assert!(
            valid_purpose.validate().is_ok(),
            "Expected valid purpose validation to succeed"
        );
    }

    #[test]
    fn test_manifest_data() {
        // Test valid case
        let mut valid_manifest = ManifestData::new();
        valid_manifest.name = "Test App".to_string();
        valid_manifest.short_name = "Test".to_string();
        valid_manifest.start_url = "/".to_string();
        valid_manifest.display = "standalone".to_string();
        valid_manifest.background_color = "#ffffff".to_string();
        assert!(valid_manifest.validate().is_ok());

        // Test name length limit
        let mut long_name = ManifestData::new();
        long_name.name = "a".repeat(MAX_MANIFEST_NAME_LENGTH + 1);
        assert!(matches!(
            long_name.validate(),
            Err(DataError::InvalidMetadata(_))
        ));

        // Test short name length limit
        let mut long_short_name = ManifestData::new();
        long_short_name.name = "Test App".to_string();
        long_short_name.short_name =
            "a".repeat(MAX_MANIFEST_SHORT_NAME_LENGTH + 1);
        assert!(matches!(
            long_short_name.validate(),
            Err(DataError::InvalidMetadata(_))
        ));

        // Test invalid display mode
        let mut invalid_display = ManifestData::new();
        invalid_display.name = "Test App".to_string();
        invalid_display.display = "invalid".to_string();
        assert!(matches!(
            invalid_display.validate(),
            Err(DataError::InvalidMetadata(_))
        ));
    }

    #[test]
    fn test_news_data() {
        // Test valid case
        let mut valid_news = NewsData::create_default();
        valid_news.news_title = "Test News".to_string();
        valid_news.news_language = "en".to_string();
        valid_news.news_publication_date =
            "2024-02-20T12:00:00Z".to_string();
        assert!(valid_news.validate().is_ok());

        // Test invalid language
        let mut invalid_lang = NewsData::create_default();
        invalid_lang.news_language = "invalid".to_string();
        assert!(matches!(
            invalid_lang.validate(),
            Err(DataError::InvalidLanguage(_))
        ));

        // Test invalid date
        let mut invalid_date = NewsData::create_default();
        invalid_date.news_publication_date = "invalid-date".to_string();
        assert!(matches!(
            invalid_date.validate(),
            Err(DataError::InvalidDate(_))
        ));

        // Test genres list
        let mut news_with_genres = NewsData::create_default();
        news_with_genres.news_genres = "Blog, OpEd".to_string();
        assert_eq!(
            news_with_genres.genres_list(),
            vec!["Blog", "OpEd"]
        );
    }

    #[test]
    fn test_news_visit_options() {
        // Test valid case
        let valid_options = NewsVisitOptions::new(
            "https://example.com",
            "Blog",
            "news, test",
            "en",
            "2024-02-20T12:00:00Z",
            "Test Publication",
            "Test Title",
        );
        assert!(valid_options.validate().is_ok());

        // Test invalid URL
        let invalid_url = NewsVisitOptions::new(
            "invalid-url",
            "Blog",
            "news, test",
            "en",
            "2024-02-20T12:00:00Z",
            "Test Publication",
            "Test Title",
        );
        assert!(matches!(
            invalid_url.validate(),
            Err(DataError::InvalidUrl(_))
        ));

        // Test missing required fields
        let missing_fields = NewsVisitOptions::new(
            "https://example.com",
            "Blog",
            "news, test",
            "en",
            "2024-02-20T12:00:00Z",
            "",
            "",
        );
        assert!(matches!(
            missing_fields.validate(),
            Err(DataError::MissingField(_))
        ));
    }

    #[test]
    fn test_humans_data() {
        // Test valid case
        let valid_humans = HumansData::new(
            "John Doe".to_string(),
            "Thank you".to_string(),
        );
        assert!(valid_humans.validate().is_ok());

        // Test empty author
        let empty_author =
            HumansData::new("".to_string(), "Thanks".to_string());
        assert!(matches!(
            empty_author.validate(),
            Err(DataError::MissingField(field)) if field == "author"
        ));

        // Test invalid website
        let mut invalid_website =
            HumansData::new("Author".to_string(), "Thanks".to_string());
        invalid_website.author_website = "invalid-url".to_string();
        assert!(matches!(
            invalid_website.validate(),
            Err(DataError::InvalidUrl(_))
        ));

        // Test invalid Twitter handle
        let mut invalid_twitter =
            HumansData::new("Author".to_string(), "Thanks".to_string());
        invalid_twitter.author_twitter = "invalid".to_string();
        assert!(matches!(
            invalid_twitter.validate(),
            Err(DataError::InvalidTwitterHandle(_))
        ));
    }

    #[test]
    fn test_meta_tag_groups() {
        // Test valid case
        let mut valid_groups = MetaTagGroups::new();
        valid_groups.og = r#"og:title="Test" og:type="website" og:url="https://example.com" og:description="Test""#.to_string();
        assert!(valid_groups.validate().is_ok());

        // Test missing OG properties
        let mut invalid_og = MetaTagGroups::new();
        invalid_og.og = "og:title=\"Test\"".to_string();
        assert!(matches!(
            invalid_og.validate(),
            Err(DataError::InvalidMetadata(_))
        ));

        // Test empty check
        assert!(MetaTagGroups::new().is_empty());

        // Test get method
        let mut groups = MetaTagGroups::new();
        groups.twitter = "twitter:card=\"summary\"".to_string();
        assert_eq!(groups.get("twitter").unwrap(), &groups.twitter);
        assert!(groups.get("invalid").is_none());
    }

    #[test]
    fn test_txt_data() {
        // Test valid case
        let valid_txt = TxtData::new("https://example.com".to_string());
        assert!(valid_txt.validate().is_ok());

        // Test empty permalink
        let empty_txt = TxtData::new("".to_string());
        assert!(matches!(
            empty_txt.validate(),
            Err(DataError::MissingField(field)) if field == "permalink"
        ));

        // Test invalid URL
        let invalid_txt = TxtData::new("invalid-url".to_string());
        assert!(matches!(
            invalid_txt.validate(),
            Err(DataError::InvalidUrl(_))
        ));

        // Test content generation
        let txt = TxtData::new("https://example.com/".to_string());
        assert_eq!(
            txt.generate_content(),
            "User-agent: *\nAllow: /\nSitemap: https://example.com/sitemap.xml"
        );
    }

    #[test]
    fn test_rss_data() {
        // Test valid case
        let mut valid_rss = RssData::new();
        valid_rss.title = "Test Feed".to_string();
        valid_rss.link = "https://example.com".to_string();
        valid_rss.description = "Test description".to_string();
        assert!(valid_rss.validate().is_ok());

        // Test missing required fields
        let empty_rss = RssData::new();
        assert!(matches!(
            empty_rss.validate(),
            Err(DataError::MissingField(_))
        ));

        // Test invalid URL
        let mut invalid_url = RssData::new();
        invalid_url.title = "Test".to_string();
        invalid_url.link = "invalid-url".to_string();
        invalid_url.description = "Test".to_string();
        assert!(matches!(
            invalid_url.validate(),
            Err(DataError::InvalidUrl(_))
        ));

        // Test invalid date
        let mut invalid_date = RssData::new();
        invalid_date.title = "Test".to_string();
        invalid_date.link = "https://example.com".to_string();
        invalid_date.description = "Test".to_string();
        invalid_date.pub_date = "invalid-date".to_string();
        assert!(matches!(
            invalid_date.validate(),
            Err(DataError::InvalidDate(_))
        ));
    }

    #[test]
    fn test_meta_tag() {
        // Test valid case
        let valid_tag = MetaTag::new(
            "description".to_string(),
            "Test description".to_string(),
        );
        assert!(valid_tag.validate().is_ok());

        // Test empty name
        let empty_name =
            MetaTag::new("".to_string(), "Test content".to_string());
        assert!(matches!(
            empty_name.validate(),
            Err(DataError::MissingField(field)) if field == "meta tag name"
        ));

        // Test invalid characters
        let invalid_name = MetaTag::new(
            "description\"".to_string(),
            "Test content".to_string(),
        );
        assert!(matches!(
            invalid_name.validate(),
            Err(DataError::InvalidMetadata(_))
        ));

        let invalid_value = MetaTag::new(
            "description".to_string(),
            "Test \"content\"".to_string(),
        );
        assert!(matches!(
            invalid_value.validate(),
            Err(DataError::InvalidMetadata(_))
        ));

        // Test HTML generation
        let tag = MetaTag::new(
            "description".to_string(),
            "Test description".to_string(),
        );
        assert_eq!(
            tag.generate(),
            r#"<meta name="description" content="Test description">"#
        );

        // Test multiple tags generation
        let tags = vec![
            MetaTag::new(
                "description".to_string(),
                "Test description".to_string(),
            ),
            MetaTag::new(
                "keywords".to_string(),
                "test, meta".to_string(),
            ),
        ];
        let html = MetaTag::generate_metatags(&tags);
        assert!(html.contains(
            r#"<meta name="description" content="Test description">"#
        ));
        assert!(html.contains(
            r#"<meta name="keywords" content="test, meta">"#
        ));
    }

    #[test]
    fn test_validation_helpers() {
        // Test URL validation
        assert!(validation::validate_url("https://example.com").is_ok());
        assert!(
            validation::validate_url("http://sub.example.com").is_ok()
        );
        assert!(validation::validate_url("").is_ok()); // Empty URLs are allowed
        assert!(validation::validate_url("invalid-url").is_err());
        assert!(validation::validate_url("http://<script>").is_err());

        // Test date validation
        assert!(
            validation::validate_date("2024-02-20T12:00:00Z").is_ok()
        );
        assert!(validation::validate_date("").is_ok()); // Empty dates are allowed
        assert!(validation::validate_date("2024-02-20").is_err());
        assert!(validation::validate_date("invalid-date").is_err());

        // Test language code validation
        assert!(validation::validate_language_code("en").is_ok());
        assert!(validation::validate_language_code("fr").is_ok());
        assert!(validation::validate_language_code("").is_ok()); // Empty codes are allowed
        assert!(validation::validate_language_code("eng").is_err());
        assert!(validation::validate_language_code("E N").is_err());

        // Test color validation
        assert!(validation::validate_color("#fff").is_ok());
        assert!(validation::validate_color("#ffffff").is_ok());
        assert!(validation::validate_color("rgb(255,255,255)").is_ok());
        assert!(validation::validate_color("").is_ok()); // Empty colors are allowed
        assert!(validation::validate_color("#ff").is_err());
        assert!(validation::validate_color("rgb(256,0,0)").is_err());

        // Test Twitter handle validation
        assert!(
            validation::validate_twitter_handle("@username").is_ok()
        );
        assert!(
            validation::validate_twitter_handle("@user_name").is_ok()
        );
        assert!(validation::validate_twitter_handle("").is_ok()); // Empty handles are allowed
        assert!(
            validation::validate_twitter_handle("username").is_err()
        );
        assert!(
            validation::validate_twitter_handle("@invalid!").is_err()
        );
        assert!(validation::validate_twitter_handle(
            "@toolongusername123"
        )
        .is_err());

        // Test image size validation
        assert!(validation::validate_image_size("192x192").is_ok());
        assert!(validation::validate_image_size("512x512").is_ok());
        assert!(validation::validate_image_size("").is_ok()); // Empty sizes are allowed
        assert!(validation::validate_image_size("192").is_err());
        assert!(validation::validate_image_size("192x").is_err());
        assert!(validation::validate_image_size("axb").is_err());
    }

    #[test]
    fn test_error_display() {
        assert_eq!(
            DataError::InvalidDomain("test".to_string()).to_string(),
            "Invalid domain name: test"
        );
        assert_eq!(
            DataError::MissingField("test".to_string()).to_string(),
            "Missing required field: test"
        );
        assert_eq!(
            DataError::InvalidDate("test".to_string()).to_string(),
            "Invalid date format: test"
        );
        assert_eq!(
            DataError::InvalidUrl("test".to_string()).to_string(),
            "Invalid URL: test"
        );
        assert_eq!(
            DataError::InvalidLanguage("test".to_string()).to_string(),
            "Invalid language code: test"
        );
        assert_eq!(
            DataError::InvalidColor("test".to_string()).to_string(),
            "Invalid color code: test"
        );
        assert_eq!(
            DataError::InvalidEmail("test".to_string()).to_string(),
            "Invalid email format: test"
        );
        assert_eq!(
            DataError::InvalidFileName("test".to_string()).to_string(),
            "Invalid file name: test"
        );
        assert_eq!(
            DataError::InvalidContent("test".to_string()).to_string(),
            "Invalid content: test"
        );
        assert_eq!(
            DataError::InvalidMetadata("test".to_string()).to_string(),
            "Invalid metadata: test"
        );
        assert_eq!(
            DataError::InvalidSize("test".to_string()).to_string(),
            "Invalid size format: test"
        );
        assert_eq!(
            DataError::InvalidTwitterHandle("test".to_string())
                .to_string(),
            "Invalid Twitter handle: test"
        );
    }

    #[test]
    fn test_combined_validation() {
        let mut manifest = ManifestData::new();
        manifest.name = "Test App".to_string();
        manifest.icons.push(IconData::new(
            "https://example.com/icon.png".to_string(),
            "192x192".to_string(),
        ));
        let result = manifest.validate();
        assert!(
            result.is_ok(),
            "Manifest validation failed: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_text_length_validation() {
        // Test empty text
        assert!(
            validation::validate_text_length("", 10, "test").is_ok()
        );

        // Test text within limit
        assert!(validation::validate_text_length("test", 10, "test")
            .is_ok());

        // Test text at exact limit
        assert!(validation::validate_text_length(
            "1234567890",
            10,
            "test"
        )
        .is_ok());

        // Test text exceeding limit
        let result =
            validation::validate_text_length("12345678901", 10, "test");
        assert!(matches!(result, Err(DataError::ContentTooLong(10))));
    }

    #[test]
    fn test_file_data_content_length() {
        // Test content within limits
        let valid_file = FileData::new(
            "test.md".to_string(),
            "A".repeat(MAX_TEXT_LENGTH),
        );
        assert!(valid_file.validate().is_ok());

        // Test content exceeding limit
        let invalid_file = FileData::new(
            "test.md".to_string(),
            "A".repeat(MAX_TEXT_LENGTH + 1),
        );
        assert!(matches!(
            invalid_file.validate(),
            Err(DataError::ContentTooLong(_))
        ));
    }

    #[test]
    fn test_tags_data_field_lengths() {
        let long_titles = TagsData::new(
            "2024-02-20T12:00:00Z".to_string(),
            "A".repeat(MAX_TEXT_LENGTH + 1),
            "Description".to_string(),
            "/page".to_string(),
            "tags".to_string(),
        );
        assert!(matches!(
            long_titles.validate(),
            Err(DataError::ContentTooLong(_))
        ));

        let long_descriptions = TagsData::new(
            "2024-02-20T12:00:00Z".to_string(),
            "Title".to_string(),
            "A".repeat(MAX_TEXT_LENGTH + 1),
            "/page".to_string(),
            "tags".to_string(),
        );
        assert!(matches!(
            long_descriptions.validate(),
            Err(DataError::ContentTooLong(_))
        ));

        let long_keywords = TagsData::new(
            "2024-02-20T12:00:00Z".to_string(),
            "Title".to_string(),
            "Description".to_string(),
            "/page".to_string(),
            "A".repeat(MAX_METADATA_LENGTH + 1),
        );
        assert!(matches!(
            long_keywords.validate(),
            Err(DataError::ContentTooLong(_))
        ));
    }

    #[test]
    fn test_humans_data_field_lengths() {
        let mut data =
            HumansData::new("Author".to_string(), "Thanks".to_string());

        // Test author location length
        data.author_location = "A".repeat(MAX_SHORT_TEXT_LENGTH + 1);
        assert!(matches!(
            data.validate(),
            Err(DataError::ContentTooLong(_))
        ));

        // Test site components length
        data.author_location = "Location".to_string();
        data.site_components = "A".repeat(MAX_METADATA_LENGTH + 1);
        assert!(matches!(
            data.validate(),
            Err(DataError::ContentTooLong(_))
        ));

        // Test site software length
        data.site_components = "Components".to_string();
        data.site_software = "A".repeat(MAX_METADATA_LENGTH + 1);
        assert!(matches!(
            data.validate(),
            Err(DataError::ContentTooLong(_))
        ));
    }

    #[test]
    fn test_manifest_data_field_lengths() {
        // Test name length (max 45)
        let mut manifest = ManifestData::new();

        // Set a description that's too long
        manifest.description = "A".repeat(MAX_METADATA_LENGTH + 1);
        let result = manifest.validate();

        // Print debug information
        println!("Validation result: {:?}", result);
        println!("Description length: {}", manifest.description.len());
        println!("MAX_METADATA_LENGTH: {}", MAX_METADATA_LENGTH);

        assert!(matches!(
            result,
            Err(DataError::ContentTooLong(MAX_METADATA_LENGTH))
        ));
    }

    #[test]
    fn test_security_data_field_validation() {
        let mut data = SecurityData::new(
            vec!["https://example.com/security".to_string()],
            "2024-12-31T23:59:59Z".to_string(),
        );

        // Test valid acknowledgments URL
        data.acknowledgments = "https://example.com/thanks".to_string();
        assert!(data.validate().is_ok());

        // Test invalid acknowledgments URL
        data.acknowledgments = "invalid-url".to_string();
        assert!(matches!(
            data.validate(),
            Err(DataError::InvalidUrl(_))
        ));

        // Test valid canonical URL
        data.acknowledgments = String::new();
        data.canonical =
            "https://example.com/.well-known/security.txt".to_string();
        assert!(data.validate().is_ok());

        // Test invalid canonical URL
        data.canonical = "not-a-url".to_string();
        assert!(matches!(
            data.validate(),
            Err(DataError::InvalidUrl(_))
        ));
    }

    #[test]
    fn test_validate_text_length_edge_cases() {
        // Test unicode characters
        assert!(validation::validate_text_length(
            "🦀rust🦀",
            10,
            "test"
        )
        .is_ok());

        // Test whitespace
        assert!(
            validation::validate_text_length("   ", 2, "test").is_err()
        );
        assert!(validation::validate_text_length("\n\n\n", 2, "test")
            .is_err());

        // Test zero length limit
        assert!(validation::validate_text_length("", 0, "test").is_ok());
        assert!(
            validation::validate_text_length("a", 0, "test").is_err()
        );
    }
}

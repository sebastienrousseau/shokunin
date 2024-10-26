// Copyright © 2024 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! JSON and data file generation functionality
//!
//! This module provides functions for generating various data files including
//! CNAME records, humans.txt, manifests, news sitemaps, robots.txt, and RSS feeds.

use crate::models::data::{
    CnameData, HumansData, ManifestData, NewsData, NewsVisitOptions,
    SecurityData, TxtData,
};
use serde_json::{json, Map};
use sitemap_gen::SiteMapData;
use std::{
    fs, io,
    path::{Path, PathBuf},
};

/// Generates CNAME file content.
///
/// # Arguments
///
/// * `options` - The CNAME data configuration
///
/// # Returns
///
/// The generated CNAME content as a string
///
/// # Example
///
/// ```
/// use staticrux::models::data::CnameData;
/// use staticrux::modules::json::cname;
///
/// let options = CnameData {
///     cname: "example.com".to_string(),
/// };
/// let content = cname(&options);
/// assert!(content.contains("example.com"));
/// ```
pub fn cname(options: &CnameData) -> String {
    let cname_value = &options.cname;
    let full_domain = format!("www.{}", cname_value);
    format!("{}\n{}", cname_value, full_domain)
}

/// Generates security.txt file content according to RFC 9116.
///
/// # Arguments
///
/// * `options` - The security.txt configuration data
///
/// # Returns
///
/// The generated security.txt content as a string
///
/// # Example
///
/// ```
/// use staticrux::models::data::SecurityData;
/// use staticrux::modules::json::security;
///
/// let options = SecurityData {
///     contact: vec!["https://example.com/security".to_string()],
///     expires: "2024-12-31T23:59:59Z".to_string(),
///     acknowledgments: "https://example.com/thanks".to_string(),
///     preferred_languages: "en, fr".to_string(),
///     canonical: "https://example.com/.well-known/security.txt".to_string(),
///     policy: "https://example.com/security-policy".to_string(),
///     hiring: String::new(),
///     encryption: String::new(),
/// };
///
/// let content = security(&options);
/// assert!(content.contains("Contact:"));
/// assert!(content.contains("Expires:"));
/// ```
pub fn security(options: &SecurityData) -> String {
    // Verify required fields are present
    if options.contact.is_empty() || options.expires.is_empty() {
        return String::new();
    }

    let mut content = String::with_capacity(500);

    // Add required fields
    for contact in &options.contact {
        content.push_str(&format!("Contact: {}\n", contact));
    }
    content.push_str(&format!("Expires: {}\n", options.expires));

    // Add optional fields if present
    if !options.acknowledgments.is_empty() {
        content.push_str(&format!(
            "Acknowledgments: {}\n",
            options.acknowledgments
        ));
    }
    if !options.preferred_languages.is_empty() {
        content.push_str(&format!(
            "Preferred-Languages: {}\n",
            options.preferred_languages
        ));
    }
    if !options.canonical.is_empty() {
        content
            .push_str(&format!("Canonical: {}\n", options.canonical));
    }
    if !options.policy.is_empty() {
        content.push_str(&format!("Policy: {}\n", options.policy));
    }
    if !options.hiring.is_empty() {
        content.push_str(&format!("Hiring: {}\n", options.hiring));
    }
    if !options.encryption.is_empty() {
        content
            .push_str(&format!("Encryption: {}\n", options.encryption));
    }

    content
}

/// Generates humans.txt file content.
///
/// # Arguments
///
/// * `options` - The humans.txt configuration data
///
/// # Returns
///
/// The generated humans.txt content
///
/// # Example
///
/// ```
/// use staticrux::models::data::HumansData;
/// use staticrux::modules::json::human;
///
/// let options = HumansData {
///     author: "John Doe".to_string(),
///     author_website: "https://example.com".to_string(),
///     author_twitter: "@johndoe".to_string(),
///     author_location: "New York".to_string(),
///     thanks: "Contributors".to_string(),
///     site_last_updated: "2024-01-01".to_string(),
///     site_standards: "HTML5, CSS3".to_string(),
///     site_components: "Rust, SSG".to_string(),
///     site_software: "Shokunin".to_string(),
/// };
/// let content = human(&options);
/// assert!(content.contains("TEAM"));
/// ```
pub fn human(options: &HumansData) -> String {
    let mut content = String::from("/* TEAM */\n");

    if !options.author.is_empty() {
        content.push_str(&format!("    Name: {}\n", options.author));
    }
    if !options.author_website.is_empty() {
        content.push_str(&format!(
            "    Website: {}\n",
            options.author_website
        ));
    }
    if !options.author_twitter.is_empty() {
        content.push_str(&format!(
            "    Twitter: {}\n",
            options.author_twitter
        ));
    }
    if !options.author_location.is_empty() {
        content.push_str(&format!(
            "    Location: {}\n",
            options.author_location
        ));
    }
    content.push_str("\n/* THANKS */\n");
    if !options.thanks.is_empty() {
        content.push_str(&format!("    Thanks: {}\n", options.thanks));
    }
    content.push_str("\n/* SITE */\n");
    if !options.site_last_updated.is_empty() {
        content.push_str(&format!(
            "    Last update: {}\n",
            options.site_last_updated
        ));
    }
    if !options.site_standards.is_empty() {
        content.push_str(&format!(
            "    Standards: {}\n",
            options.site_standards
        ));
    }
    if !options.site_components.is_empty() {
        content.push_str(&format!(
            "    Components: {}\n",
            options.site_components
        ));
    }
    if !options.site_software.is_empty() {
        content.push_str(&format!(
            "    Software: {}\n",
            options.site_software
        ));
    }
    content
}

/// Generates web app manifest content.
///
/// # Arguments
///
/// * `options` - The manifest configuration data
///
/// # Returns
///
/// Result containing the generated manifest JSON string
pub fn manifest(
    options: &ManifestData,
) -> Result<String, serde_json::Error> {
    let mut json_map = Map::new();
    json_map.insert(
        "background_color".to_string(),
        json!(options.background_color),
    );
    json_map
        .insert("description".to_string(), json!(options.description));
    json_map.insert("display".to_string(), json!(options.display));

    let mut icons_vec = vec![];
    for icon in &options.icons {
        let mut icon_map = Map::new();
        icon_map.insert("src".to_string(), json!(icon.src));
        icon_map.insert("sizes".to_string(), json!(icon.sizes));
        if let Some(icon_type) = &icon.icon_type {
            icon_map.insert("type".to_string(), json!(icon_type));
        }
        if let Some(purpose) = &icon.purpose {
            icon_map.insert("purpose".to_string(), json!(purpose));
        }
        icons_vec.push(json!(icon_map));
    }
    json_map.insert("icons".to_string(), json!(icons_vec));
    json_map.insert("name".to_string(), json!(options.name));
    json_map
        .insert("orientation".to_string(), json!(options.orientation));
    json_map.insert("scope".to_string(), json!(options.scope));
    json_map
        .insert("short_name".to_string(), json!(options.short_name));
    json_map.insert("start_url".to_string(), json!(options.start_url));
    json_map
        .insert("theme_color".to_string(), json!(options.theme_color));

    serde_json::to_string_pretty(&json_map)
}

/// Generates a news sitemap in XML format.
///
/// # Arguments
///
/// * `options` - The news sitemap configuration
///
/// # Returns
///
/// The generated news sitemap XML content
pub fn news_sitemap(options: NewsData) -> String {
    let mut urls = vec![];
    if let Err(e) = news_visit_dirs(&options, &mut urls) {
        log::error!("Error generating news sitemap: {}", e);
    }
    format!(
        r#"<?xml version='1.0' encoding='UTF-8'?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9" 
        xmlns:news="http://www.google.com/schemas/sitemap-news/0.9" 
        xmlns:image="http://www.google.com/schemas/sitemap-image/1.1">
    {}</urlset>"#,
        urls.join("\n")
    )
}

/// Helper function to visit directories for sitemap generation
fn visit_dirs(
    base_dir: &Path,
    dir: &Path,
    base_url: &str,
    changefreq: &str,
    lastmod: &str,
    urls: &mut Vec<String>,
) -> io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                visit_dirs(
                    base_dir, &path, base_url, changefreq, lastmod,
                    urls,
                )?;
            } else if path.file_name().unwrap_or_default()
                == "index.html"
            {
                if let Ok(stripped_path) = path.strip_prefix(base_dir) {
                    if let Some(url) = stripped_path.to_str() {
                        urls.push(format!(
                            r#"<url><changefreq>{}</changefreq><lastmod>{}</lastmod><loc>{}/{}</loc></url>"#,
                            changefreq, lastmod, base_url, url
                        ));
                    }
                }
            }
        }
    }
    Ok(())
}

/// Helper function to visit directories for news sitemap generation
fn news_visit_dirs(
    options: &NewsData,
    urls: &mut Vec<String>,
) -> io::Result<()> {
    urls.push(format!(
        r#"<url>
    <loc>{}</loc>
    <news:news>
        <news:publication>
            <news:name>{}</news:name>
            <news:language>{}</news:language>
        </news:publication>
        <news:genres>{}</news:genres>
        <news:publication_date>{}</news:publication_date>
        <news:title>{}</news:title>
        <news:keywords>{}</news:keywords>
    </news:news>
    <image:image>
        <image:loc>{}</image:loc>
    </image:image>
</url>"#,
        options.news_loc,
        options.news_publication_name,
        options.news_language,
        options.news_genres,
        options.news_publication_date,
        options.news_title,
        options.news_keywords,
        options.news_image_loc,
    ));

    Ok(())
}

/// Generates a single news sitemap entry
pub fn generate_news_sitemap_entry(
    options: &NewsVisitOptions<'_>,
) -> String {
    format!(
        r#"<url>
    <loc>{}</loc>
    <lastmod>{}</lastmod>
    <news:news>
        <news:publication>
            <news:name>{}</news:name>
            <news:language>{}</news:language>
        </news:publication>
        <news:publication_date>{}</news:publication_date>
        <news:title>{}</news:title>
    </news:news>
</url>"#,
        options.base_url,
        options.news_publication_date,
        options.news_publication_name,
        options.news_language,
        options.news_publication_date,
        options.news_title,
    )
}

/// Generates a sitemap based on provided configuration
pub fn sitemap(options: SiteMapData, dir: &Path) -> String {
    let base_dir = PathBuf::from(dir);
    let mut urls = vec![];
    if let Err(e) = visit_dirs(
        &base_dir,
        &base_dir,
        options.loc.as_str(),
        &options.changefreq.to_string(),
        &options.lastmod,
        &mut urls,
    ) {
        log::error!("Error generating sitemap: {}", e);
    }

    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9" 
        xmlns:news="http://www.google.com/schemas/sitemap-news/0.9" 
        xmlns:xhtml="http://www.w3.org/1999/xhtml" 
        xmlns:mobile="http://www.google.com/schemas/sitemap-mobile/1.0" 
        xmlns:image="http://www.google.com/schemas/sitemap-image/1.1" 
        xmlns:video="http://www.google.com/schemas/sitemap-video/1.1">
    {}</urlset>"#,
        urls.join("\n")
    )
}

/// Generates robots.txt content
pub fn txt(options: &TxtData) -> String {
    format!("User-agent: *\nSitemap: {}/sitemap.xml", options.permalink)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cname_generation() {
        let options = CnameData {
            cname: "example.com".to_string(),
        };
        let content = cname(&options);
        assert_eq!(content, "example.com\nwww.example.com");
    }

    #[test]
    fn test_txt_generation() {
        let options = TxtData {
            permalink: "https://example.com".to_string(),
        };
        let content = txt(&options);
        assert_eq!(
            content,
            "User-agent: *\nSitemap: https://example.com/sitemap.xml"
        );
    }

    #[test]
    fn test_human_txt_generation() {
        let options = HumansData {
            author: "Test Author".to_string(),
            author_website: "https://example.com".to_string(),
            author_twitter: "@test".to_string(),
            author_location: "Test Location".to_string(),
            thanks: "Test Thanks".to_string(),
            site_last_updated: "2024-01-01".to_string(),
            site_standards: "HTML5".to_string(),
            site_components: "Test Components".to_string(),
            site_software: "Test Software".to_string(),
        };
        let content = human(&options);
        assert!(content.contains("Test Author"));
        assert!(content.contains("https://example.com"));
        assert!(content.contains("@test"));
    }

    #[test]
    fn test_security_txt_generation() {
        let options = SecurityData {
            contact: vec![
                "https://example.com/security".to_string(),
                "mailto:security@example.com".to_string(),
            ],
            expires: "2024-12-31T23:59:59Z".to_string(),
            acknowledgments: "https://example.com/thanks".to_string(),
            preferred_languages: "en, fr, de".to_string(),
            canonical: "https://example.com/.well-known/security.txt"
                .to_string(),
            policy: "https://example.com/security-policy".to_string(),
            hiring: "https://example.com/security-jobs".to_string(),
            encryption: "https://example.com/pgp-key.txt".to_string(),
        };

        let content = security(&options);

        // Check required fields
        assert!(
            content.contains("Contact: https://example.com/security")
        );
        assert!(
            content.contains("Contact: mailto:security@example.com")
        );
        assert!(content.contains("Expires: 2024-12-31T23:59:59Z"));

        // Check optional fields
        assert!(content
            .contains("Acknowledgments: https://example.com/thanks"));
        assert!(content.contains("Preferred-Languages: en, fr, de"));
        assert!(content.contains(
            "Canonical: https://example.com/.well-known/security.txt"
        ));
        assert!(content
            .contains("Policy: https://example.com/security-policy"));
        assert!(content
            .contains("Hiring: https://example.com/security-jobs"));
        assert!(content
            .contains("Encryption: https://example.com/pgp-key.txt"));
    }

    #[test]
    fn test_security_txt_missing_required_fields() {
        let options = SecurityData {
            contact: vec![],
            expires: String::new(),
            acknowledgments: "https://example.com/thanks".to_string(),
            preferred_languages: "en".to_string(),
            canonical: String::new(),
            policy: String::new(),
            hiring: String::new(),
            encryption: String::new(),
        };

        let content = security(&options);
        assert!(content.is_empty());
    }

    #[test]
    fn test_security_txt_minimal() {
        let options = SecurityData {
            contact: vec!["https://example.com/security".to_string()],
            expires: "2024-12-31T23:59:59Z".to_string(),
            acknowledgments: String::new(),
            preferred_languages: String::new(),
            canonical: String::new(),
            policy: String::new(),
            hiring: String::new(),
            encryption: String::new(),
        };

        let content = security(&options);
        assert!(
            content.contains("Contact: https://example.com/security")
        );
        assert!(content.contains("Expires: 2024-12-31T23:59:59Z"));
        assert!(!content.contains("Acknowledgments:"));
        assert!(!content.contains("Preferred-Languages:"));
    }

    #[test]
    fn test_security_txt_multiple_contacts() {
        let options = SecurityData {
            contact: vec![
                "https://example.com/security".to_string(),
                "mailto:security@example.com".to_string(),
                "tel:+1-201-555-0123".to_string(),
            ],
            expires: "2024-12-31T23:59:59Z".to_string(),
            ..Default::default()
        };

        let content = security(&options);
        assert!(
            content.contains("Contact: https://example.com/security")
        );
        assert!(
            content.contains("Contact: mailto:security@example.com")
        );
        assert!(content.contains("Contact: tel:+1-201-555-0123"));
        assert_eq!(
            content.matches("Contact:").count(),
            3,
            "Should have exactly three Contact fields"
        );
    }
}

// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright © 2024 Shokunin Static Site Generator. All rights reserved.

use crate::models::data::{
    CnameData, HumansData, ManifestData, NewsData, NewsVisitOptions,
    SiteMapData, TxtData,
};
use serde_json::{json, Map};
use std::{
    fs, io,
    path::{Path, PathBuf},
};

/// Generates a CNAME file for a web app based on the provided `CnameData` options.
///
/// # Arguments
///
/// * `options` - The `CnameData` object containing the CNAME value.
///
/// # Returns
///
/// A string containing the CNAME file content.
///
/// # Example
///
/// ```
/// use ssg_core::models::data::CnameData;
/// use ssg_core::modules::json::cname;
///
/// let options = CnameData {
///     cname: "example.com".to_string(),
/// };
///
/// let cname_content = cname(&options);
/// assert_eq!(cname_content, "example.com\nwww.example.com");
/// ```
pub fn cname(options: &CnameData) -> String {
    let cname_value = &options.cname;
    let full_domain = format!("www.{}", cname_value);
    format!("{}\n{}", cname_value, full_domain)
}

/// Generates a humans.txt for a web app based on the provided `HumansData` options.
///
/// # Arguments
///
/// * `options` - The `HumansData` object containing the human-readable data.
///
/// # Returns
///
/// A string containing the humans.txt file content.
pub fn human(options: &HumansData) -> String {
    let mut s = String::from("/* TEAM */\n");

    if !options.author.is_empty() {
        s.push_str(&format!("    Name: {}\n", options.author));
    }
    if !options.author_website.is_empty() {
        s.push_str(&format!(
            "    Website: {}\n",
            options.author_website
        ));
    }
    if !options.author_twitter.is_empty() {
        s.push_str(&format!(
            "    Twitter: {}\n",
            options.author_twitter
        ));
    }
    if !options.author_location.is_empty() {
        s.push_str(&format!(
            "    Location: {}\n",
            options.author_location
        ));
    }
    s.push_str("\n/* THANKS */\n");
    if !options.thanks.is_empty() {
        s.push_str(&format!("    Thanks: {}\n", options.thanks));
    }
    s.push_str("\n/* SITE */\n");
    if !options.site_last_updated.is_empty() {
        s.push_str(&format!(
            "    Last update: {}\n",
            options.site_last_updated
        ));
    }
    if !options.site_standards.is_empty() {
        s.push_str(&format!(
            "    Standards: {}\n",
            options.site_standards
        ));
    }
    if !options.site_components.is_empty() {
        s.push_str(&format!(
            "    Components: {}\n",
            options.site_components
        ));
    }
    if !options.site_software.is_empty() {
        s.push_str(&format!(
            "    Software: {}\n",
            options.site_software
        ));
    }
    s
}

/// Generates a JSON manifest for a web app based on the provided `ManifestData` options.
///
/// # Arguments
///
/// * `options` - The `ManifestData` object containing the manifest data.
///
/// # Returns
///
/// A JSON string containing the manifest.
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

/// Generates a news sitemap for a web app based on the provided `NewsData` options.
///
/// # Arguments
///
/// * `options` - The `NewsData` object containing the news sitemap data.
///
/// # Returns
///
/// A string containing the news sitemap XML.
pub fn news_sitemap(options: NewsData) -> String {
    let mut urls = vec![];
    if let Err(e) = news_visit_dirs(&options, &mut urls) {
        eprintln!("Error generating news sitemap: {}", e);
    }
    format!(
        r#"<?xml version='1.0' encoding='UTF-8'?><urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9" xmlns:news="http://www.google.com/schemas/sitemap-news/0.9" xmlns:image="http://www.google.com/schemas/sitemap-image/1.1">{}</urlset>"#,
        urls.join("\n")
    )
}

/// Recursively visits all directories in a tree and adds the URLs of all index.html files to the `urls` vector.
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
                match path.strip_prefix(base_dir) {
                    Ok(stripped_path) => {
                        if let Some(url) = stripped_path.to_str() {
                            urls.push(format!(
                                r#"<url><changefreq>{}</changefreq><lastmod>{}</lastmod><loc>{}/{}</loc></url>"#,
                                changefreq, lastmod, base_url, url
                            ));
                        }
                    }
                    Err(err) => {
                        return Err(io::Error::new(
                            io::ErrorKind::Other,
                            err,
                        ))
                    }
                }
            }
        }
    }
    Ok(())
}

/// Recursively visits directories to generate news sitemap entries.
fn news_visit_dirs(
    options: &NewsData,
    urls: &mut Vec<String>,
) -> io::Result<()> {
    let base_url = &options.news_loc;
    let news_publication_name = &options.news_publication_name;
    let news_language = &options.news_language;
    let news_image_loc = &options.news_image_loc;
    let news_genres = &options.news_genres;
    let news_publication_date = &options.news_publication_date;
    let news_title = &options.news_title;
    let news_keywords = &options.news_keywords;

    urls.push(format!(
        r#"<url><loc>{}</loc><news:news><news:publication><news:name>{}</news:name><news:language>{}</news:language></news:publication><news:genres>{}</news:genres><news:publication_date>{}</news:publication_date><news:title>{}</news:title><news:keywords>{}</news:keywords></news:news><image:image><image:loc>{}</image:loc></image:image></url>"#,
        base_url,
        news_publication_name,
        news_language,
        news_genres,
        news_publication_date,
        news_title,
        news_keywords,
        news_image_loc,
    ));

    Ok(())
}

/// Generates a single news sitemap entry for all directories recursively visited.
///
/// The `options` parameter contains the necessary fields for each sitemap entry.
///
/// Returns a string containing the news sitemap XML.
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

/// Generates a sitemap for a web app based on the provided `SiteMapData` options.
///
/// # Arguments
///
/// * `options` - The `SiteMapData` object containing the sitemap data.
/// * `dir` - The directory containing the web app.
///
/// # Returns
///
/// A string containing the sitemap.xml file.
pub fn sitemap(options: SiteMapData, dir: &Path) -> String {
    let base_dir = PathBuf::from(dir);
    let mut urls = vec![];
    if let Err(e) = visit_dirs(
        &base_dir,
        &base_dir,
        &options.loc,
        &options.changefreq,
        &options.lastmod,
        &mut urls,
    ) {
        eprintln!("Error generating sitemap: {}", e);
    }

    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?><urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9" xmlns:news="http://www.google.com/schemas/sitemap-news/0.9" xmlns:xhtml="http://www.w3.org/1999/xhtml" xmlns:mobile="http://www.google.com/schemas/sitemap-mobile/1.0" xmlns:image="http://www.google.com/schemas/sitemap-image/1.1" xmlns:video="http://www.google.com/schemas/sitemap-video/1.1">{}</urlset>"#,
        urls.join("\n")
    )
}

/// Generates a robots.txt for a web app based on the provided `TxtData` options.
///
/// # Arguments
///
/// * `options` - The `TxtData` object containing the robots.txt data.
///
/// # Returns
///
/// A string containing the robots.txt file.
pub fn txt(options: &TxtData) -> String {
    let url = format!("{}/sitemap.xml", options.permalink);
    format!("User-agent: *\nSitemap: {}", url)
}

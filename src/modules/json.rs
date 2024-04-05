// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright © 2024 Shokunin Static Site Generator. All rights reserved.

use std::{
    fs,
    path::{Path, PathBuf},
};
use serde_json::{json, Map};
use crate::models::data::NewsVisitOptions;
use crate::models::data::{CnameData, HumansData, ManifestData, NewsSiteMapData, SiteMapData, TxtData};

/// ## Function: `cname` - Generate a CNAME for a web app
///
/// The `CnameData` object contains the following fields:
///
/// * `cname`: The CNAME value of the web app.
///
/// Returns a string containing the CNAME file.
pub fn cname(options: &CnameData) -> String {
    let cname_value = options.cname.clone();
    let full_domain = format!("www.{}", cname_value);
    let base_domain = cname_value;
    format!("{}\n{}", base_domain, full_domain)
}

/// ## Function: `human` - Generate a humans.txt for a web app
///
/// The `HumansData` object contains the following fields:
///
/// * `author_location`: The location of the author.
/// * `author_twitter`: The Twitter handle of the author.
/// * `author_website`: The website of the author.
/// * `author`: The author of the web app.
/// * `site_components`: The components that the web app uses.
/// * `site_last_updated`: The date on which the web app was last updated.
/// * `site_software`: The software that the web app uses.
/// * `site_standards`: The standards that the web app follows.
/// * `thanks`: A list of people or organizations to thank for their contributions to the web app.
///
/// Returns a string containing the humans.txt file.
pub fn human(options: &HumansData) -> String {
    let mut s = String::from("/* TEAM */\n");

    if !options.author.is_empty() {
        s.push_str(&format!("    Name: {}\n", options.author));
    }
    if !options.author_website.is_empty() {
        s.push_str(&format!("    Website: {}\n", options.author_website));
    }
    if !options.author_twitter.is_empty() {
        s.push_str(&format!("    Twitter: {}\n", options.author_twitter));
    }
    if !options.author_location.is_empty() {
        s.push_str(&format!("    Location: {}\n", options.author_location));
    }
    s.push_str("\n/* THANKS */\n");
    if !options.thanks.is_empty() {
        s.push_str(&format!("    Thanks: {}\n", options.thanks));
    }
    s.push_str("\n/* SITE */\n");
    if !options.site_last_updated.is_empty() {
        s.push_str(
            &format!("    Last update: {}\n", options.site_last_updated)
        );
    }
    if !options.site_standards.is_empty() {
        s.push_str(&format!("    Standards: {}\n", options.site_standards));
    }
    if !options.site_components.is_empty() {
        s.push_str(
            &format!("    Components: {}\n", options.site_components)
        );
    }
    if !options.site_software.is_empty() {
        s.push_str(&format!("    Software: {}\n", options.site_software));
    }
    s
}

/// ## Function: `manifest` - Generate a JSON manifest for a web app
///
/// The `ManifestData` object contains the following fields:
///
/// * `background_color`: The background color of the web app.
/// * `description`: The description of the web app.
/// * `display`: The display mode of the web app.
/// * `icons`: A list of icons for the web app.
/// * `name`: The name of the web app.
/// * `orientation`: The orientation of the web app.
/// * `scope`: The scope of the web app.
/// * `short_name`: The short name of the web app.
/// * `start_url`: The start URL of the web app.
/// * `theme_color`: The theme color of the web app.
///
/// Returns a JSON string containing the manifest.
pub fn manifest(options: &ManifestData) -> String {
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

    serde_json::to_string_pretty(&json_map).unwrap()
}

/// ## Function: `news_sitemap` - Generate a news sitemap for a web app.
///
/// The `NewsSiteMapData` object contains the necessary fields for each sitemap entry.
///
/// Returns a string containing the news sitemap XML.
pub fn news_sitemap(options: NewsSiteMapData) -> String {
    let genres = options.news_genres.clone();
    let keywords = options.news_keywords.clone();
    let language = options.news_language.clone();
    let loc = options.news_loc.clone();
    let mut urls = vec![];
    let publication_date = options.news_publication_date.clone();
    let publication_name = options.news_publication_name.clone();
    let title = options.news_title.clone();

    let _ = news_visit_dirs(
        &NewsVisitOptions {
            base_url: &loc,
            news_genres: &genres,
            news_keywords: &keywords,
            news_language: &language,
            news_publication_name: &publication_name,
            news_publication_date: &publication_date,
            news_title: &title,
        },
        &mut urls,
    );
    let urls_str = urls.join("\n");

    format!(
        r#"<?xml version='1.0' encoding='UTF-8'?><urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9" xmlns:news="http://www.google.com/schemas/sitemap-news/0.9" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xsi:schemaLocation="http://www.sitemaps.org/schemas/sitemap/0.9 http://www.sitemaps.org/schemas/sitemap/0.9/sitemap.xsd http://www.google.com/schemas/sitemap-news/0.9 http://www.google.com/schemas/sitemap-news/0.9/sitemap-news.xsd">{}</urlset>"#,
        urls_str
    )
}

/// Generates a news sitemap for a web app.
///
/// The `SiteMapData` object contains the necessary fields for each sitemap entry.
///
/// Returns a string containing the news sitemap XML.
pub fn news_visit_dirs(
    options: &NewsVisitOptions<'_>,
    urls: &mut Vec<String>,
) -> std::io::Result<()> {
    let base_url = options.base_url;
    let news_publication_name = options.news_publication_name;
    let news_language = options.news_language;
    let news_genres = options.news_genres;
    let news_publication_date = options.news_publication_date;
    let news_title = options.news_title;
    let news_keywords = options.news_keywords;

    urls.push(format!(
        r#"<url><loc>{}</loc><news:news><news:publication><news:name>{}</news:name><news:language>{}</news:language></news:publication><news:genres>{}</news:genres><news:publication_date>{}</news:publication_date><news:title>{}</news:title><news:keywords>{}</news:keywords></news:news></url>"#,
        base_url,
        news_publication_name,
        news_language,
        news_genres,
        news_publication_date,
        news_title,
        news_keywords
    ));

    Ok(())
}

/// ## Function: `sitemap` - Generate a sitemap for a web app
///
/// The `SiteMapData` object contains the following fields:
///
/// * `changefreq`: The change frequency of the web app.
/// * `lastmod`: The last modified date of the web app.
/// * `loc`: The base URL of the web app.
///
/// The `dir` parameter is the directory containing the web app.
///
/// Returns a string containing the sitemap.xml file.
pub fn sitemap(options: SiteMapData, dir: &Path) -> String {
    let lastmod = options.lastmod.clone();
    let changefreq = options.changefreq.clone();
    let base_url = options.loc.clone();
    let base_dir = PathBuf::from(dir);
    let mut urls = vec![];

    visit_dirs(
        &base_dir,
        &base_dir,
        &base_url,
        &changefreq,
        &lastmod,
        &mut urls,
    )
    .unwrap();

    let urls_str = urls.join("\n");

    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?><urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9" xmlns:news="http://www.google.com/schemas/sitemap-news/0.9" xmlns:xhtml="http://www.w3.org/1999/xhtml" xmlns:mobile="http://www.google.com/schemas/sitemap-mobile/1.0" xmlns:image="http://www.google.com/schemas/sitemap-image/1.1" xmlns:video="http://www.google.com/schemas/sitemap-video/1.1">{}</urlset>"#,
        urls_str.replace("'\n'", "").replace("    ", "")
    )
}

/// ## Function: `txt` - Generate a robots.txt for a web app
///
/// The `TxtData` object contains the following fields:
///
/// * `permalink`: The permalink of the web app.
///
/// Returns a string containing the robots.txt file.
pub fn txt(options: &TxtData) -> String {
    let permalink = options.permalink.clone();
    let url = format!("{}/sitemap.xml", permalink);
    "User-agent: *\nSitemap: {{url}}".replace("{{url}}", &url)
}

/// ## Function: `visit_dirs` - Recursively visit all directories in a tree and add the URLs of all index.html files to the `urls` vector.
///
/// The `base_dir` parameter is the root directory of the tree.
///
/// The `dir` parameter is the current directory being visited.
///
/// The `base_url` parameter is the base URL of the website.
///
/// The `changefreq` parameter is the change frequency of the website.
///
/// The `lastmod` parameter is the last modified date of the website.
///
/// The `urls` parameter is a vector of URLs that will be populated by the function.
///
/// Returns a `std::io::Result` value.
pub fn visit_dirs(
    base_dir: &Path,
    dir: &Path,
    base_url: &str,
    changefreq: &str,
    lastmod: &str,
    urls: &mut Vec<String>,
) -> std::io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                visit_dirs(
                    base_dir, &path, base_url, changefreq, lastmod,
                    urls,
                )?;
            } else if path.file_name().unwrap() == "index.html" {
                let url = path
                    .strip_prefix(base_dir)
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .replace("'\\'", "/");
                urls.push(format!(
                    r#"<url><changefreq>{}</changefreq><lastmod>{}</lastmod><loc>{}/{}</loc></url>"#,
                    changefreq, lastmod, base_url, url
                ));
            }
        }
    }
    Ok(())
}

// Copyright © 2023 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::{
    fs,
    path::{Path, PathBuf}
};
use serde_json::{json, Map};

use crate::data::{
    CnameData, HumansData, ManifestData,
    SiteMapData, TxtData
};

/// ## Function: `manifest` - Generate a JSON manifest for a web app
pub fn manifest(options: &ManifestData) -> String {
    let mut json_map = Map::new();
    json_map.insert("name".to_string(), json!(options.name));
    json_map
        .insert("short_name".to_string(), json!(options.short_name));
    json_map.insert("start_url".to_string(), json!(options.start_url));
    json_map.insert("display".to_string(), json!(options.display));
    json_map.insert(
        "background_color".to_string(),
        json!(options.background_color),
    );
    json_map
        .insert("description".to_string(), json!(options.description));

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

    json_map
        .insert("orientation".to_string(), json!(options.orientation));
    json_map.insert("scope".to_string(), json!(options.scope));
    json_map
        .insert("theme_color".to_string(), json!(options.theme_color));

    serde_json::to_string_pretty(&json_map).unwrap()
}

/// ## Function: `txt` - Generate a robots.txt for a web app
pub fn txt(options: &TxtData) -> String {
    let permalink = options.permalink.clone();
    let url = format!("{}/sitemap.xml", permalink);
    "User-agent: *\nSitemap: {{url}}".replace("{{url}}", &url)
}

/// ## Function: `cname` - Generate a CNAME for a web app
pub fn cname(options: &CnameData) -> String {
    let cname_value = options.cname.clone();
    let full_domain = format!("www.{}", cname_value);
    let base_domain = cname_value;
    format!("{}\n{}", base_domain, full_domain)
}

/// ## Function: `human` - Generate a humans.txt for a web app
pub fn human(options: &HumansData) -> String {
    let mut s = String::from("/* TEAM */\n");

    if !options.author.is_empty() {
        s.push_str(&format!("	Name: {}\n", options.author));
    }
    if !options.author_website.is_empty() {
        s.push_str(&format!("	Website: {}\n", options.author_website));
    }
    if !options.author_twitter.is_empty() {
        s.push_str(&format!("	Twitter: {}\n", options.author_twitter));
    }
    if !options.author_location.is_empty() {
        s.push_str(&format!("	Location: {}\n", options.author_location));
    }
    s.push_str("\n/* THANKS */\n");
    if !options.thanks.is_empty() {
        s.push_str(&format!("	Thanks: {}\n", options.thanks));
    }
    s.push_str("\n/* SITE */\n");
    if !options.site_last_updated.is_empty() {
        s.push_str(&format!(
            "	Last update: {}\n",
            options.site_last_updated
        ));
    }
    if !options.site_standards.is_empty() {
        s.push_str(&format!("	Standards: {}\n", options.site_standards));
    }
    if !options.site_components.is_empty() {
        s.push_str(&format!(
            "	Components: {}\n",
            options.site_components
        ));
    }
    if !options.site_software.is_empty() {
        s.push_str(&format!("	Software: {}\n", options.site_software));
    }
    s
}

/// ## Function: `sitemap` - Generate a sitemap for a web app
pub fn sitemap(options: SiteMapData, dir: &Path) -> String {
    let changefreq = options.changefreq.clone();
    let base_url = options.loc.clone();
    let base_dir = PathBuf::from(dir);
    let mut urls = vec![];

    visit_dirs(&base_dir, &base_dir, &base_url, &changefreq, &mut urls)
        .unwrap();

    let urls_str = urls.join("\n");

    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?><urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9" xmlns:news="http://www.google.com/schemas/sitemap-news/0.9" xmlns:xhtml="http://www.w3.org/1999/xhtml" xmlns:mobile="http://www.google.com/schemas/sitemap-mobile/1.0" xmlns:image="http://www.google.com/schemas/sitemap-image/1.1" xmlns:video="http://www.google.com/schemas/sitemap-video/1.1">{}</urlset>"#,
        urls_str.replace("'\n'", "").replace("    ", "")
    )
}

fn visit_dirs(
    base_dir: &Path,
    dir: &Path,
    base_url: &str,
    changefreq: &str,
    urls: &mut Vec<String>,
) -> std::io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                visit_dirs(
                    base_dir, &path, base_url, changefreq, urls,
                )?;
            } else if path.file_name().unwrap() == "index.html" {
                let url = path
                    .strip_prefix(base_dir)
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .replace("'\\'", "/");
                urls.push(format!(
                    r#"
    <url>
        <loc>{}/{}</loc>
        <changefreq>{}</changefreq>
    </url>"#,
                    base_url, url, changefreq
                ));
            }
        }
    }
    Ok(())
}

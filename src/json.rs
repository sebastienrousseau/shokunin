// Copyright © 2023 Shokunin (職人) Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

use serde_json::{json, Map};

#[derive(Debug, Default, PartialEq, Eq, Hash, Clone)]

/// ## Struct: `CnameOptions` - Options for the `cname` function
pub struct CnameOptions {
    /// A string representing the domain of the web app
    pub cname: String,
}

#[derive(Debug, Default, PartialEq, Eq, Hash, Clone)]

/// ## Struct: `TxtOptions` - Options for the `txt` function
pub struct TxtOptions {
    /// A string representing the permalink of the web app
    pub permalink: String,
}

#[derive(Debug, Default, PartialEq, Eq, Hash, Clone)]
/// ## Struct: `IconOptions` - Options for the `manifest` function
pub struct IconOptions {
    /// A string representing the source of the icon
    pub src: String,
    /// A string representing the sizes of the icon
    pub sizes: String,
    /// A string representing the type of the icon
    pub icon_type: Option<String>,
    /// A string representing the purpose of the icon
    pub purpose: Option<String>,
}

#[derive(Debug, Default, PartialEq, Eq, Hash, Clone)]
/// ## Struct: `ManifestOptions` - Options for the `manifest` function
pub struct ManifestOptions {
    /// A string representing the background color of the web app
    pub background_color: String,
    /// A string representing the text direction of the web app
    pub description: String,
    /// A string representing the text direction of the web app
    pub dir: String,
    /// A string representing the display mode of the web app
    pub display: String,
    /// A Vector representing the icons of the web app
    pub icons: Vec<IconOptions>,
    /// A string representing the identity of the web app
    pub identity: String,
    /// A string representing the language of the web app
    pub lang: String,
    /// A string representing the name of the web app
    pub name: String,
    /// A string representing the orientation of the web app
    pub orientation: String,
    /// A string representing the scope of the web app
    pub scope: String,
    /// A string representing the short name of the web app
    pub short_name: String,
    /// A string representing the start URL of the web app
    pub start_url: String,
    /// A string representing the theme color of the web app
    pub theme_color: String,
}

/// ## Function: `manifest` - Generate a JSON manifest for a web app
pub fn manifest(options: &ManifestOptions) -> String {
    let mut json_map = Map::new();
    json_map.insert(
        "background_color".to_string(),
        json!(options.background_color),
    );
    json_map
        .insert("description".to_string(), json!(options.description));
    json_map.insert("dir".to_string(), json!(options.dir));
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

    json_map.insert("identity".to_string(), json!(options.identity));
    json_map.insert("lang".to_string(), json!(options.lang));
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

/// ## Function: `txt` - Generate a robots.txt for a web app
pub fn txt(options: &TxtOptions) -> String {
    let permalink = options.permalink.clone();
    let url = format!("{}/sitemap.xml", permalink);
    "User-agent: *\nSitemap: {{url}}".replace("{{url}}", &url)
}

/// ## Function: `cname` - Generate a CNAME for a web app
pub fn cname(options: &CnameOptions) -> String {
    let cname_value = options.cname.clone();
    let full_domain = format!("www.{}", cname_value);
    let base_domain = cname_value;
    format!("{}\n{}", full_domain, base_domain)
}

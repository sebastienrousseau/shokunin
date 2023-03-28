// Copyright © 2023 Shokunin (職人). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

use serde_json::{json, Map};

#[derive(Debug, Default, PartialEq, Eq, Hash, Clone)]
/// ## Struct: `ManifestOptions` - Options for the `manifest` function
///
/// The parameters are used to populate the fields of a Web Application
/// Manifest file, which is a a JSON-based file format that contains
/// startup parameters and application defaults for when a web
/// application is launched.
///
/// The manifest is used by web browsers to provide a native application
/// -like experience for web applications.
///
/// # Arguments
///
/// * `background_color` - A string representing the background color of
///                        the web app (in hexadecimal format).
/// * `dir`              - A string representing the text direction of
///                        the web app (e.g. "ltr", "rtl", "auto" etc.).
/// * `description`      - A string representing a description of the
///                        web app (used by search engines).
/// * `display`          - A string representing the display mode of the
///                        web app (e.g. "fullscreen", "standalone",
///                        etc.).
/// * `icons`            - A string representing the icons of the web
///                        app. (e.g. "icons": [ { "src":
///                        "icon/lowres.webp", "sizes": "64x64",
///                        "type": "image/webp" }, { "src":
///                        "icon/lowres.png", "sizes": "64x64" } ])
/// * `identity`         - A string representing the identity of the web
///                        app. The identity takes the form of a URL,
///                        which is same origin as the start URL.
/// * `lang`             - A string representing the language of the web
///                        app. This should be a valid BCP 47 language
///                        tag.
/// * `name`             - A string representing the name of the web app.
/// * `orientation`      - A string representing the orientation of the
///                        web app (e.g. "any", "natural", "landscape",
///                        "landscape-primary", "landscape-secondary",
///                        "portrait", "portrait-primary", or "portrait-
///                        secondary").
/// * `scope`            - A string representing the scope of the web app.
/// * `start_url`        - A string representing the start URL of the web
///                        app.
/// * `short_name`       - A string representing the short name of the web
///                        app.
/// * `theme_color`      - A string representing the theme color of the
///                        web app.
///
pub struct ManifestOptions {
    /// A string representing the background color of the web app
    pub background_color: String,
    /// A string representing the text direction of the web app
    pub description: String,
    /// A string representing the text direction of the web app
    pub dir: String,
    /// A string representing the display mode of the web app
    pub display: String,
    /// A string representing the icons of the web app
    pub icons: String,
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
/// ## Function: `manifest` - Generates a JSON Web Application Manifest file
///
/// The parameters are used to populate the fields of a Web Application
/// Manifest file, which is a a JSON-based file format that contains
/// startup parameters and application defaults for when a web
/// application is launched.
///
/// The manifest is used by web browsers to provide a native application
/// -like experience for web applications.
///
/// # Arguments
///
/// * `options` - A `ManifestOptions` struct containing the options for
///              the manifest.
///
/// # Returns
///
/// A string containing the JSON Web Application Manifest file.
///
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
    json_map.insert("icons".to_string(), json!(options.icons));
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

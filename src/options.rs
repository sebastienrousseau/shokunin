// Copyright © 2023 Shokunin (職人) Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

#[derive(Debug, Default, PartialEq, Eq, Hash, Clone)]
/// Options for the `cname` function
pub struct CnameOptions {
    /// A string representing the domain of the web app
    pub cname: String,
}

#[derive(Debug, Default, PartialEq, Eq, Hash, Clone)]
/// Options for the `icon` function
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
/// Options for the `manifest` function
pub struct ManifestOptions {
    /// A string representing the background color of the web app
    pub background_color: String,
    /// A string representing the text direction of the web app
    pub description: String,
    /// A string representing the display mode of the web app
    pub display: String,
    /// A Vector representing the icons of the web app
    pub icons: Vec<IconOptions>,
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

#[derive(Debug, Default, PartialEq, Eq, Hash, Clone)]
/// Options for the `sitemap` function
pub struct SitemapOptions {
    /// A string representing the local
    pub loc: String,
    /// A string representing the lastmod
    pub lastmod: String,
    /// A string representing the changefreq
    pub changefreq: String,
}

#[derive(Debug, Default, PartialEq, Eq, Hash, Clone)]
/// Options for the `txt` function
pub struct TxtOptions {
    /// A string representing the permalink of the web app
    pub permalink: String,
}

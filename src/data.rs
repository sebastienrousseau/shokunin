// Copyright © 2023 Shokunin (職人) Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

use serde::{Deserialize, Serialize};

#[derive(
    Debug, Default, PartialEq, Eq, Hash, Clone, Serialize, Deserialize,
)]
/// Options for the `cname` function
pub struct CnameData {
    /// A string representing the domain of the web app
    pub cname: String,
}

impl CnameData {
    /// Creates a new `CnameData` struct with the given cname.
    pub fn new() -> CnameData {
        CnameData::default()
    }
}

#[derive(
    Debug, Default, PartialEq, Eq, Hash, Clone, Serialize, Deserialize,
)]
/// File struct to hold the name and content of a file.
pub struct FileData {
    /// The content of the file, escaped for CNAME.
    pub cname: String,
    /// The content of the file, escaped for JSON.
    pub json: String,
    /// The name of the file.
    pub name: String,
    /// The content of the file.
    pub content: String,
    /// The content of the file, escaped for RSS.
    pub rss: String,
    /// The content of the file, escaped for sitemap.
    pub sitemap: String,
    /// The content of the file, escaped for TXT.
    pub txt: String,
}

impl FileData {
    /// Creates a new `FileData` struct with the given name and content.
    pub fn new() -> FileData {
        FileData::default()
    }
}

#[derive(
    Debug, Default, PartialEq, Eq, Hash, Clone, Serialize, Deserialize,
)]
/// Options for the `icon` function
pub struct IconData {
    /// A string representing the purpose of the icon
    pub purpose: Option<String>,
    /// A string representing the sizes of the icon
    pub sizes: String,
    /// A string representing the source of the icon
    pub src: String,
    /// A string representing the type of the icon
    pub icon_type: Option<String>,
}

impl IconData {
    /// Creates a new `IconData` struct with the given source and sizes.
    pub fn new() -> IconData {
        IconData::default()
    }
}

#[derive(
    Debug, Default, PartialEq, Eq, Hash, Clone, Serialize, Deserialize,
)]
/// Options for the `manifest` function
pub struct ManifestData {
    /// A string representing the background color of the web app
    pub background_color: String,
    /// A string representing the description of the web app
    pub description: String,
    /// A string representing the display mode of the web app
    pub display: String,
    /// A vector representing the icons of the web app
    pub icons: Vec<IconData>,
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

impl ManifestData {
    /// Creates a new `ManifestData` struct with default values for all fields.
    pub fn new() -> ManifestData {
        ManifestData::default()
    }
}

#[derive(
    Debug, Default, PartialEq, Eq, Hash, Clone, Serialize, Deserialize,
)]
/// Options for the `sitemap` function
pub struct SitemapData {
    /// A string representing the changefreq
    pub changefreq: String,
    /// A string representing the lastmod
    pub lastmod: String,
    /// A string representing the local
    pub loc: String,
}

impl SitemapData {
    /// Creates a new `SitemapData` struct with the given loc, lastmod, and changefreq.
    pub fn new() -> SitemapData {
        SitemapData::default()
    }
}

#[derive(
    Debug, Default, PartialEq, Eq, Hash, Clone, Serialize, Deserialize,
)]
/// Options for the `txt` function
pub struct TxtData {
    /// A string representing the permalink of the web app
    pub permalink: String,
}

impl TxtData {
    /// Creates a new `TxtData` struct with the given permalink.
    pub fn new() -> TxtData {
        TxtData::default()
    }
}

/// The `RssData` struct holds all necessary options and data for an RSS feed.
///
/// This includes everything from metadata about the RSS feed itself, such as its title and language,
/// to information about individual items in the feed, such as their titles and publication dates.
///
/// The values contained in an instance of `RssData` can be used to generate a complete RSS feed in XML format.
#[derive(
    Debug, Default, PartialEq, Eq, Hash, Clone, Serialize, Deserialize,
)]
pub struct RssData {
    /// The Atom link of the RSS feed.
    pub atom_link: String,
    /// The author of the RSS feed.
    pub author: String,
    /// The category of the RSS feed.
    pub category: String,
    /// The copyright notice for the content of the feed.
    pub copyright: String,
    /// The description of the RSS feed.
    pub description: String,
    /// The docs of the RSS feed.
    pub docs: String,
    /// The generator of the RSS feed.
    pub generator: String,
    /// The image of the RSS feed. This can be a URL pointing to an image file.
    pub image: String,
    /// The Guid of the RSS item. This is a unique identifier for the item.
    pub item_guid: String,
    /// The description of the RSS item.
    pub item_description: String,
    /// The link of the RSS item.
    pub item_link: String,
    /// The publication date of the RSS item.
    pub item_pub_date: String,
    /// The title of the RSS item.
    pub item_title: String,
    /// The language of the RSS feed.
    pub language: String,
    /// The last build date of the RSS feed.
    pub last_build_date: String,
    /// The link to the atom feed.
    pub link: String,
    /// The managing editor of the RSS feed.
    pub managing_editor: String,
    /// The publication date of the RSS feed.
    pub pub_date: String,
    /// The title of the RSS feed.
    pub title: String,
    /// Time To Live: the number of minutes the feed should be cached before refreshing.
    pub ttl: String,
    /// The webmaster of the RSS feed.
    pub webmaster: String,
}

impl RssData {
    /// Creates a new `RssData` struct with default values for all fields.
    ///
    /// This is a convenience function that makes it easy to create a new `RssData` without having to specify every field.
    /// Fields can then be set individually on the returned instance.
    pub fn new() -> RssData {
        RssData::default()
    }

    /// Sets the value of a field.
    pub fn set(&mut self, key: &str, value: &str) {
        match key {
            "atom_link" => self.atom_link = value.to_string(),
            "author" => self.author = value.to_string(),
            "category" => self.category = value.to_string(),
            "copyright" => self.copyright = value.to_string(),
            "description" => self.description = value.to_string(),
            "docs" => self.docs = value.to_string(),
            "generator" => self.generator = value.to_string(),
            "image" => self.image = value.to_string(),
            "item_guid" => self.item_guid = value.to_string(),
            "item_description" => {
                self.item_description = value.to_string()
            }
            "item_link" => self.item_link = value.to_string(),
            "item_pub_date" => self.item_pub_date = value.to_string(),
            "item_title" => self.item_title = value.to_string(),
            "language" => self.language = value.to_string(),
            "last_build_date" => {
                self.last_build_date = value.to_string()
            }
            "link" => self.link = value.to_string(),
            "managing_editor" => {
                self.managing_editor = value.to_string()
            }
            "pub_date" => self.pub_date = value.to_string(),
            "title" => self.title = value.to_string(),
            "ttl" => self.ttl = value.to_string(),
            "webmaster" => self.webmaster = value.to_string(),
            _ => (),
        }
    }
}

#[derive(
    Debug, Default, PartialEq, Eq, Hash, Clone, Serialize, Deserialize,
)]
/// The `MetatagsData` struct holds all necessary data for a single metatag.
/// This includes everything from the name of the metatag to its content.
/// The values contained in an instance of `MetatagsData` can be used to
/// generate a complete metatag in HTML format.
/// The `MetatagsData` struct is used in the `Metatags` struct.
pub struct MetatagsData {
    /// The name of the metatag.
    name: &'static str,
    /// The content of the metatag.
    value: String,
}

impl MetatagsData {
    /// Creates a new `MetatagsData` struct with the given name and value.
    ///
    /// This includes all the information about the metatags of a web page.
    pub fn new(name: &'static str, value: String) -> Self {
        MetatagsData { name, value }
    }

    /// Generates a complete metatag in HTML format.
    pub fn generate(&self) -> String {
        format!(
            r#"<meta name="{}" content="{}">"#,
            self.name, self.value
        )
    }

    /// Generates a complete list of metatags in HTML format.
    pub fn generate_metatags(metatags: &[MetatagsData]) -> String {
        metatags.iter().map(|metatag| metatag.generate()).collect()
    }
}

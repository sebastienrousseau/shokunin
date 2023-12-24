// Copyright © 2023-2024 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

use serde::{Deserialize, Serialize};
use std::fmt;

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
    pub fn new(cname: String) -> Self {
        CnameData { cname }
    }
}

#[derive(Debug, Default, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
/// File struct to hold the title, permalink of a file.
pub struct PageData {
    /// The title of the file.
    pub title: String,
    /// The description of the file.
    pub description: String,
    /// The publication date of the file.
    pub date: String,
    /// The permalink of the file.
    pub permalink: String,
}

impl fmt::Display for PageData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {} {} {}",
        self.title,
        self.description,
        self.date,
        self.permalink)
    }
}

#[derive(
    Debug, Default, PartialEq, Eq, Hash, Clone, Serialize, Deserialize,
)]
/// File struct to hold the name and content of a file.
pub struct FileData {
    /// The name of the file.
    pub name: String,
    /// The content of the file.
    pub content: String,
    /// The content of the file, escaped for CNAME.
    pub cname: String,
    /// The content of the file, escaped for JSON.
    pub json: String,
    /// The content of the file, escaped for HUMANS.
    pub human: String,
    /// The content of the file, escaped for keywords.
    pub keyword: String,
    /// The content of the file, escaped for RSS.
    pub rss: String,
    /// The content of the file, escaped for sitemap.
    pub sitemap: String,
    //  The content of the file, escaped for tags.
    // pub tags: String,
    /// The content of the file, escaped for TXT.
    pub txt: String,
}

impl FileData {
    /// Creates a new `FileData` struct with the given name and content.
    pub fn new(name: String, content: String) -> Self {
        FileData {
            name,
            content,
            cname: String::new(),
            json: String::new(),
            human: String::new(),
            keyword: String::new(),
            rss: String::new(),
            sitemap: String::new(),
            // tags: String::new(),
            txt: String::new(),
        }
    }
}

#[derive(Debug, Default, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
/// Options for the `tags` function
pub struct TagsData {
    /// A string representing the publication date of the web app
    pub dates: String,
    /// A string representing the title of the web app
    pub titles: String,
    /// A string representing the description of the web app
    pub descriptions: String,
    /// A string representing the permalink of the web app
    pub permalinks: String,
    /// A string representing the keywords of the web app
    pub keywords: String,
}

#[derive(
    Debug, Default, PartialEq, Eq, Hash, Clone, Serialize, Deserialize,
)]
/// Options for the `sw_file` function
pub struct SwFileData {
    /// A string representing the offline page of the web app
    pub offline_page_url: String,
}

impl SwFileData {
    /// Creates a new `SwFileData` struct with the given offline page.
    pub fn new(offline_page_url: String) -> Self {
        SwFileData { offline_page_url }
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
    pub fn new(src: String, sizes: String) -> Self {
        IconData {
            purpose: None,
            sizes,
            src,
            icon_type: None,
        }
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
    pub fn new() -> Self {
        ManifestData::default()
    }
}

#[derive(
    Debug, Default, PartialEq, Eq, Hash, Clone, Serialize, Deserialize,
)]
/// Options for the `sitemap` function
pub struct SiteMapData {
    /// A string representing the changefreq
    pub changefreq: String,
    /// A string representing the lastmod
    pub lastmod: String,
    /// A string representing the local
    pub loc: String,
}

impl SiteMapData {
    /// Creates a new `SiteMapData` struct with the given loc, lastmod, and changefreq.
    pub fn new(
        loc: String,
        lastmod: String,
        changefreq: String,
    ) -> Self {
        SiteMapData {
            changefreq,
            lastmod,
            loc,
        }
    }
}

#[derive(
    Debug, Default, PartialEq, Eq, Hash, Clone, Serialize, Deserialize,
)]
/// Options for the `human` function
pub struct HumanData {
    /// A string representing the author of the web app
    pub author: Option<String>,
    /// A string representing the website of the author
    pub author_website: Option<String>,
    /// A string representing the twitter of the author
    pub author_twitter: Option<String>,
    /// A string representing the location of the author
    pub author_location: Option<String>,
    /// A string representing the thanks of the author (name or url)
    pub thanks: Option<String>,
    /// A string representing the site last updated date
    pub site_last_updated: Option<String>,
    /// A string representing the site standards of the web app
    pub site_standards: Option<String>,
    /// A string representing the site components of the web app
    pub site_components: Option<String>,
    /// A string representing the site software of the web app
    pub site_software: Option<String>,
}

impl HumanData {
    /// Creates a new `HumanData` struct with default values for all fields.
    pub fn new() -> Self {
        HumanData::default()
    }
}

/// The `MetaTagGroups` struct holds collections of meta tags for different platforms and categories.
///
/// The struct includes fields for Apple-specific meta tags, primary meta tags, Open Graph meta tags,
/// Microsoft-specific meta tags, and Twitter-specific meta tags. Each field contains a string
/// representation of the HTML meta tags for its respective category or platform.
#[derive(Debug, Default, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
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
    /// Creates a new `MetaTagGroups` instance with default values for all fields.
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
    /// Returns true if all fields are empty.
    pub fn is_empty(&self) -> bool {
        self.apple.is_empty() &&
        self.primary.is_empty() &&
        self.og.is_empty() &&
        self.ms.is_empty() &&
        self.twitter.is_empty()
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
    pub fn new(permalink: String) -> Self {
        TxtData { permalink }
    }
}

#[derive(
    Debug, Default, PartialEq, Eq, Hash, Clone, Serialize, Deserialize,
)]
/// Options for the `humans` function
pub struct HumansData {
    /// A string representing the author of the web app
    pub author: String,
    /// A string representing the website of the author
    pub author_website: String,
    /// A string representing the twitter of the author
    pub author_twitter: String,
    /// A string representing the location of the author
    pub author_location: String,
    /// A string representing the thanks of the author (name or url)
    pub thanks: String,
    /// A string representing the site last updated date
    pub site_last_updated: String,
    /// A string representing the site standards of the web app
    pub site_standards: String,
    /// A string representing the site components of the web app
    pub site_components: String,
    /// A string representing the site software of the web app
    pub site_software: String,
}

impl HumansData {
    /// Creates a new `HumansData` struct with the given author and thanks.
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
    pub fn new() -> Self {
        RssData::default()
    }

    /// Sets the value of a field.
    pub fn set<T: Into<String>>(&mut self, key: &str, value: T) {
        match key {
            "atom_link" => self.atom_link = value.into(),
            "author" => self.author = value.into(),
            "category" => self.category = value.into(),
            "copyright" => self.copyright = value.into(),
            "description" => self.description = value.into(),
            "docs" => self.docs = value.into(),
            "generator" => self.generator = value.into(),
            "image" => self.image = value.into(),
            "item_guid" => self.item_guid = value.into(),
            "item_description" => self.item_description = value.into(),
            "item_link" => self.item_link = value.into(),
            "item_pub_date" => self.item_pub_date = value.into(),
            "item_title" => self.item_title = value.into(),
            "language" => self.language = value.into(),
            "last_build_date" => self.last_build_date = value.into(),
            "link" => self.link = value.into(),
            "managing_editor" => self.managing_editor = value.into(),
            "pub_date" => self.pub_date = value.into(),
            "title" => self.title = value.into(),
            "ttl" => self.ttl = value.into(),
            "webmaster" => self.webmaster = value.into(),
            _ => (),
        }
    }
}

/// The `MetaTag` struct holds all necessary data for a single metatag.
///
/// This includes everything from the name of the metatag to its content.
/// The values contained in an instance of `MetaTag` can be used to
/// generate a complete metatag in HTML format.
/// The `MetaTag` struct is used in the `Metatags` struct.
#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
pub struct MetaTag {
    /// The name of the metatag.
    pub name: String,
    /// The content of the metatag.
    pub value: String,
}

impl MetaTag {
    /// Creates a new `MetaTag` struct with the given name and value.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the metatag.
    /// * `value` - The content of the metatag.
    ///
    /// # Returns
    ///
    /// A new `MetaTag` struct instance.
    pub fn new(name: String, value: String) -> Self {
        MetaTag { name, value }
    }

    /// Generates a complete metatag in HTML format.
    ///
    /// # Returns
    ///
    /// A string representing the complete metatag in HTML format.
    pub fn generate(&self) -> String {
        format!(
            "<meta content=\"{}\" name=\"{}\">",
            self.value, self.name
        )
    }

    /// Generates a complete list of metatags in HTML format.
    ///
    /// # Arguments
    ///
    /// * `metatags` - A slice containing the `MetaTag` instances.
    ///
    /// # Returns
    ///
    /// A string representing the complete list of metatags in HTML format.
    pub fn generate_metatags(metatags: &[MetaTag]) -> String {
        metatags.iter().map(MetaTag::generate).collect()
    }
}

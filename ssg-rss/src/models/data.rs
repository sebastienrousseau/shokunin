use serde::{Deserialize, Serialize};

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

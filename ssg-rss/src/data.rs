// Copyright © 2024 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

use crate::version::RssVersion;
use dtt::datetime::DateTime;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// The `RssData` struct holds all the necessary options and data for an RSS feed.
/// This includes metadata about the RSS feed itself, such as its title and language,
/// as well as information about individual items in the feed, such as their titles and publication dates.
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
    /// The docs link of the RSS feed.
    pub docs: String,
    /// The generator of the RSS feed.
    pub generator: String,
    /// The image URL of the RSS feed.
    pub image: String,
    /// The GUID of the RSS item (unique identifier).
    pub item_guid: String,
    /// The description of the RSS item.
    pub item_description: String,
    /// The link to the RSS item.
    pub item_link: String,
    /// The publication date of the RSS item.
    pub item_pub_date: String,
    /// The title of the RSS item.
    pub item_title: String,
    /// The language of the RSS feed.
    pub language: String,
    /// The last build date of the RSS feed.
    pub last_build_date: String,
    /// The main link to the RSS feed.
    pub link: String,
    /// The managing editor of the RSS feed.
    pub managing_editor: String,
    /// The publication date of the RSS feed.
    pub pub_date: String,
    /// The title of the RSS feed.
    pub title: String,
    /// Time To Live (TTL), the number of minutes the feed should be cached before refreshing.
    pub ttl: String,
    /// The webmaster of the RSS feed.
    pub webmaster: String,
    /// A collection of additional items in the RSS feed.
    pub items: Vec<RssItem>,
    /// The version of the RSS feed.
    pub version: RssVersion,
}

/// Represents an item in the RSS feed.
#[derive(
    Debug, Default, PartialEq, Eq, Hash, Clone, Serialize, Deserialize,
)]
pub struct RssItem {
    /// The GUID of the RSS item (unique identifier).
    pub guid: String,
    /// The description of the RSS item.
    pub description: String,
    /// The link to the RSS item.
    pub link: String,
    /// The publication date of the RSS item.
    pub pub_date: String,
    /// The title of the RSS item.
    pub title: String,
    /// The author of the RSS item.
    pub author: String,
}

impl RssData {
    /// Creates a new `RssData` instance with default values and a specified RSS version.
    /// If no version is provided, the default is RSS 2.0.
    ///
    /// # Arguments
    ///
    /// * `version` - An optional `RssVersion` specifying the RSS version for the feed.
    ///
    /// # Returns
    ///
    /// A new `RssData` instance.
    pub fn new(version: Option<RssVersion>) -> Self {
        RssData {
            version: version.unwrap_or(RssVersion::RSS2_0), // Default to RSS 2.0 if not provided
            ..Default::default()
        }
    }

    /// Sorts the RSS items by their publication date in descending order.
    pub fn sort_items_by_pub_date(&mut self) {
        self.items.sort_by(|a, b| {
            let date_a = DateTime::parse(&a.pub_date).ok();
            let date_b = DateTime::parse(&b.pub_date).ok();
            date_b.cmp(&date_a) // Sort in descending order (newest first)
        });
    }

    /// Sets the value of a specified field and returns the `RssData` instance for method chaining.
    ///
    /// # Arguments
    /// * `key` - The field to set.
    /// * `value` - The value to assign to the field.
    pub fn set<T: Into<String>>(mut self, key: &str, value: T) -> Self {
        let value = value.into();
        match key {
            "atom_link" => self.atom_link = value,
            "author" => self.author = value,
            "category" => self.category = value,
            "copyright" => self.copyright = value,
            "description" => self.description = value,
            "docs" => self.docs = value,
            "generator" => self.generator = value,
            "image" => self.image = value,
            "item_guid" => self.item_guid = value,
            "item_description" => self.item_description = value,
            "item_link" => self.item_link = value,
            "item_pub_date" => self.item_pub_date = value,
            "item_title" => self.item_title = value,
            "language" => self.language = value,
            "last_build_date" => self.last_build_date = value,
            "link" => self.link = value,
            "managing_editor" => self.managing_editor = value,
            "pub_date" => self.pub_date = value,
            "title" => self.title = value,
            "ttl" => self.ttl = value,
            "webmaster" => self.webmaster = value,
            _ => eprintln!(
                "Warning: Attempt to set unknown field '{}'",
                key
            ),
        }
        self
    }

    /// Adds an item to the RSS feed.
    pub fn add_item(&mut self, item: RssItem) {
        self.items.push(item);
    }

    /// Removes an item from the RSS feed by its GUID.
    ///
    /// Returns `true` if an item was removed, `false` otherwise.
    pub fn remove_item(&mut self, guid: &str) -> bool {
        let initial_len = self.items.len();
        self.items.retain(|item| item.guid != guid);
        self.items.len() < initial_len
    }

    /// Returns the number of items in the RSS feed.
    pub fn item_count(&self) -> usize {
        self.items.len()
    }

    /// Clears all items from the RSS feed.
    pub fn clear_items(&mut self) {
        self.items.clear();
    }

    /// Validates the `RssData` to ensure that all required fields are set.
    ///
    /// Returns `Ok(())` if valid, otherwise returns a `Vec<String>` with error messages.
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        let required_fields = [
            ("title", &self.title),
            ("link", &self.link),
            ("description", &self.description),
        ];

        for (field, value) in required_fields.iter() {
            if value.is_empty() {
                errors.push(format!(
                    "Required field '{}' is empty",
                    field
                ));
            }
        }

        // Validate each RssItem in the items vector
        for (index, item) in self.items.iter().enumerate() {
            if let Err(item_errors) = item.validate() {
                errors.push(format!(
                    "Errors in item {}: {:?}",
                    index, item_errors
                ));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Converts the `RssData` into a `HashMap<String, String>` for easier manipulation.
    pub fn to_hash_map(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        map.insert("atom_link".to_string(), self.atom_link.clone());
        map.insert("author".to_string(), self.author.clone());
        map.insert("category".to_string(), self.category.clone());
        map.insert("copyright".to_string(), self.copyright.clone());
        map.insert("description".to_string(), self.description.clone());
        map.insert("docs".to_string(), self.docs.clone());
        map.insert("generator".to_string(), self.generator.clone());
        map.insert("image".to_string(), self.image.clone());
        map.insert("item_guid".to_string(), self.item_guid.clone());
        map.insert(
            "item_description".to_string(),
            self.item_description.clone(),
        );
        map.insert("item_link".to_string(), self.item_link.clone());
        map.insert(
            "item_pub_date".to_string(),
            self.item_pub_date.clone(),
        );
        map.insert("item_title".to_string(), self.item_title.clone());
        map.insert("language".to_string(), self.language.clone());
        map.insert(
            "last_build_date".to_string(),
            self.last_build_date.clone(),
        );
        map.insert("link".to_string(), self.link.clone());
        map.insert(
            "managing_editor".to_string(),
            self.managing_editor.clone(),
        );
        map.insert("pub_date".to_string(), self.pub_date.clone());
        map.insert("title".to_string(), self.title.clone());
        map.insert("ttl".to_string(), self.ttl.clone());
        map.insert("webmaster".to_string(), self.webmaster.clone());
        map
    }

    // Field setter methods

    /// The `version` field setter method.
    pub fn version(mut self, version: RssVersion) -> Self {
        self.version = version;
        self
    }

    /// The `atom_link` field setter method.
    pub fn atom_link<T: Into<String>>(self, value: T) -> Self {
        self.set("atom_link", value)
    }
    /// The `author` field setter method.
    pub fn author<T: Into<String>>(self, value: T) -> Self {
        self.set("author", value)
    }
    /// The `category` field setter method.
    pub fn category<T: Into<String>>(self, value: T) -> Self {
        self.set("category", value)
    }
    /// The `copyright` field setter method.
    pub fn copyright<T: Into<String>>(self, value: T) -> Self {
        self.set("copyright", value)
    }
    /// The `description` field setter method.
    pub fn description<T: Into<String>>(self, value: T) -> Self {
        self.set("description", value)
    }
    /// The `docs` field setter method.
    pub fn docs<T: Into<String>>(self, value: T) -> Self {
        self.set("docs", value)
    }
    /// The `generator` field setter method.
    pub fn generator<T: Into<String>>(self, value: T) -> Self {
        self.set("generator", value)
    }
    /// The `image` field setter method.
    pub fn image<T: Into<String>>(self, value: T) -> Self {
        self.set("image", value)
    }
    /// The `item_guid` field setter method.
    pub fn item_guid<T: Into<String>>(self, value: T) -> Self {
        self.set("item_guid", value)
    }
    /// The `item_description` field setter method.
    pub fn item_description<T: Into<String>>(self, value: T) -> Self {
        self.set("item_description", value)
    }
    /// The `item_link` field setter method.
    pub fn item_link<T: Into<String>>(self, value: T) -> Self {
        self.set("item_link", value)
    }
    /// The `item_pub_date` field setter method.
    pub fn item_pub_date<T: Into<String>>(self, value: T) -> Self {
        self.set("item_pub_date", value)
    }
    /// The `item_title` field setter method.
    pub fn item_title<T: Into<String>>(self, value: T) -> Self {
        self.set("item_title", value)
    }
    /// The `language` field setter method.
    pub fn language<T: Into<String>>(self, value: T) -> Self {
        self.set("language", value)
    }
    /// The `last_build_date` field setter method.
    pub fn last_build_date<T: Into<String>>(self, value: T) -> Self {
        self.set("last_build_date", value)
    }
    /// The `link` field setter method.
    pub fn link<T: Into<String>>(self, value: T) -> Self {
        self.set("link", value)
    }
    /// The `managing_editor` field setter method.
    pub fn managing_editor<T: Into<String>>(self, value: T) -> Self {
        self.set("managing_editor", value)
    }
    /// The `pub_date` field setter method.
    pub fn pub_date<T: Into<String>>(self, value: T) -> Self {
        self.set("pub_date", value)
    }
    /// The `title` field setter method.
    pub fn title<T: Into<String>>(self, value: T) -> Self {
        self.set("title", value)
    }
    /// The `ttl` field setter method.
    pub fn ttl<T: Into<String>>(self, value: T) -> Self {
        self.set("ttl", value)
    }
    /// The `webmaster` field setter method.
    pub fn webmaster<T: Into<String>>(self, value: T) -> Self {
        self.set("webmaster", value)
    }
}

impl RssItem {
    /// Creates a new `RssItem` with default values.
    pub fn new() -> Self {
        RssItem::default()
    }

    /// Sets the value of a field and returns the `RssItem` instance for method chaining.
    ///
    /// # Arguments
    ///
    /// * `key` - The field to set.
    /// * `value` - The value to assign to the field.
    pub fn set<T: Into<String>>(mut self, key: &str, value: T) -> Self {
        let value = value.into();
        match key {
            "guid" => self.guid = value,
            "description" => self.description = value,
            "link" => self.link = value,
            "pub_date" => self.pub_date = value,
            "title" => self.title = value,
            "author" => self.author = value,
            _ => eprintln!(
                "Warning: Attempt to set unknown field '{}'",
                key
            ),
        }
        self
    }

    /// Validates the `RssItem` to ensure all required fields are set and valid.
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        let required_fields = [
            ("title", &self.title),
            ("link", &self.link),
            ("description", &self.description),
            ("guid", &self.guid),
        ];

        for (field, value) in required_fields.iter() {
            if value.is_empty() {
                errors.push(format!(
                    "Required field '{}' is empty",
                    field
                ));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    // Field setter methods

    /// The `guid` field setter method.
    pub fn guid<T: Into<String>>(self, value: T) -> Self {
        self.set("guid", value)
    }
    /// The `description` field setter method.
    pub fn description<T: Into<String>>(self, value: T) -> Self {
        self.set("description", value)
    }
    /// The `link` field setter method.
    pub fn link<T: Into<String>>(self, value: T) -> Self {
        self.set("link", value)
    }
    /// The `pub_date` field setter method.
    pub fn pub_date<T: Into<String>>(self, value: T) -> Self {
        self.set("pub_date", value)
    }
    /// The `title` field setter method.
    pub fn title<T: Into<String>>(self, value: T) -> Self {
        self.set("title", value)
    }
    /// The `author` field setter method.
    pub fn author<T: Into<String>>(self, value: T) -> Self {
        self.set("author", value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rss_data_new_and_set() {
        let rss_data = RssData::new(None)
            .title("Test RSS Feed")
            .link("https://example.com")
            .description("A test RSS feed")
            .version(RssVersion::RSS2_0);

        assert_eq!(rss_data.title, "Test RSS Feed");
        assert_eq!(rss_data.link, "https://example.com");
        assert_eq!(rss_data.description, "A test RSS feed");
        assert_eq!(rss_data.version, RssVersion::RSS2_0);
    }

    #[test]
    fn test_rss_data_validate() {
        let valid_rss_data = RssData::new(None)
            .title("Test RSS Feed")
            .link("https://example.com")
            .description("A test RSS feed");

        assert!(valid_rss_data.validate().is_ok());

        let invalid_rss_data = RssData::new(None)
            .title("Test RSS Feed")
            .description("A test RSS feed");

        assert!(invalid_rss_data.validate().is_err());
    }

    #[test]
    fn test_add_item() {
        let mut rss_data = RssData::new(None)
            .title("Test RSS Feed")
            .link("https://example.com")
            .description("A test RSS feed");

        let item = RssItem::new()
            .title("Test Item")
            .link("https://example.com/item")
            .description("A test item")
            .guid("unique-id-1")
            .pub_date("2024-03-21");

        rss_data.add_item(item);

        assert_eq!(rss_data.items.len(), 1);
        assert_eq!(rss_data.items[0].title, "Test Item");
        assert_eq!(rss_data.items[0].link, "https://example.com/item");
        assert_eq!(rss_data.items[0].description, "A test item");
        assert_eq!(rss_data.items[0].guid, "unique-id-1");
        assert_eq!(rss_data.items[0].pub_date, "2024-03-21");
    }

    #[test]
    fn test_remove_item() {
        let mut rss_data = RssData::new(None)
            .title("Test RSS Feed")
            .link("https://example.com")
            .description("A test RSS feed");

        let item1 = RssItem::new()
            .title("Item 1")
            .link("https://example.com/item1")
            .description("First item")
            .guid("guid1");

        let item2 = RssItem::new()
            .title("Item 2")
            .link("https://example.com/item2")
            .description("Second item")
            .guid("guid2");

        rss_data.add_item(item1);
        rss_data.add_item(item2);

        assert_eq!(rss_data.item_count(), 2);

        assert!(rss_data.remove_item("guid1"));
        assert_eq!(rss_data.item_count(), 1);
        assert_eq!(rss_data.items[0].title, "Item 2");

        assert!(!rss_data.remove_item("non-existent-guid"));
        assert_eq!(rss_data.item_count(), 1);
    }

    #[test]
    fn test_clear_items() {
        let mut rss_data = RssData::new(None)
            .title("Test RSS Feed")
            .link("https://example.com")
            .description("A test RSS feed");

        rss_data.add_item(RssItem::new().title("Item 1").guid("guid1"));
        rss_data.add_item(RssItem::new().title("Item 2").guid("guid2"));

        assert_eq!(rss_data.item_count(), 2);

        rss_data.clear_items();

        assert_eq!(rss_data.item_count(), 0);
    }

    #[test]
    fn test_rss_item_validate() {
        let valid_item = RssItem::new()
            .title("Valid Item")
            .link("https://example.com/valid")
            .description("A valid item")
            .guid("valid-guid");

        assert!(valid_item.validate().is_ok());

        let invalid_item = RssItem::new()
            .title("Invalid Item")
            .description("An invalid item");

        let result = invalid_item.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 2); // Missing link and guid
    }

    #[test]
    fn test_rss_data_validate_with_items() {
        let mut rss_data = RssData::new(None)
            .title("Test RSS Feed")
            .link("https://example.com")
            .description("A test RSS feed");

        let valid_item = RssItem::new()
            .title("Valid Item")
            .link("https://example.com/valid")
            .description("A valid item")
            .guid("valid-guid");

        let invalid_item = RssItem::new()
            .title("Invalid Item")
            .description("An invalid item");

        rss_data.add_item(valid_item);
        rss_data.add_item(invalid_item);

        let result = rss_data.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1); // One error for the invalid item
        assert!(errors[0].contains("Errors in item 1")); // The second item (index 1) is invalid
    }

    #[test]
    fn test_sort_items_by_pub_date() {
        let mut rss_data = RssData::new(None)
            .title("Test RSS Feed")
            .link("https://example.com")
            .description("A test RSS feed");

        let item1 = RssItem::new()
            .title("Item 1")
            .link("https://example.com/item1")
            .description("First item")
            .guid("guid1")
            .pub_date("2024-03-20T12:00:00Z");

        let item2 = RssItem::new()
            .title("Item 2")
            .link("https://example.com/item2")
            .description("Second item")
            .guid("guid2")
            .pub_date("2024-03-22T12:00:00Z");

        let item3 = RssItem::new()
            .title("Item 3")
            .link("https://example.com/item3")
            .description("Third item")
            .guid("guid3")
            .pub_date("2024-03-21T12:00:00Z");

        rss_data.add_item(item1);
        rss_data.add_item(item2);
        rss_data.add_item(item3);

        rss_data.sort_items_by_pub_date();

        assert_eq!(rss_data.items[0].title, "Item 2");
        assert_eq!(rss_data.items[1].title, "Item 3");
        assert_eq!(rss_data.items[2].title, "Item 1");
    }

    #[test]
    fn test_to_hash_map() {
        let rss_data = RssData::new(None)
            .title("Test RSS Feed")
            .link("https://example.com")
            .description("A test RSS feed")
            .language("en-US")
            .pub_date("2024-03-21")
            .last_build_date("2024-03-21")
            .ttl("60");

        let hash_map = rss_data.to_hash_map();

        assert_eq!(
            hash_map.get("title"),
            Some(&"Test RSS Feed".to_string())
        );
        assert_eq!(
            hash_map.get("link"),
            Some(&"https://example.com".to_string())
        );
        assert_eq!(
            hash_map.get("description"),
            Some(&"A test RSS feed".to_string())
        );
        assert_eq!(
            hash_map.get("language"),
            Some(&"en-US".to_string())
        );
        assert_eq!(
            hash_map.get("pub_date"),
            Some(&"2024-03-21".to_string())
        );
        assert_eq!(
            hash_map.get("last_build_date"),
            Some(&"2024-03-21".to_string())
        );
        assert_eq!(hash_map.get("ttl"), Some(&"60".to_string()));
    }

    #[test]
    fn test_rss_data_version() {
        let rss_data = RssData::new(None).version(RssVersion::RSS1_0);
        assert_eq!(rss_data.version, RssVersion::RSS1_0);
    }

    #[test]
    fn test_rss_data_default_version() {
        let rss_data = RssData::new(None);
        assert_eq!(rss_data.version, RssVersion::RSS2_0);
    }

    #[test]
    fn test_rss_item_new_and_set() {
        let item = RssItem::new()
            .title("Test Item")
            .link("https://example.com/item")
            .description("A test item")
            .guid("unique-id")
            .pub_date("2024-03-21")
            .author("John Doe");

        assert_eq!(item.title, "Test Item");
        assert_eq!(item.link, "https://example.com/item");
        assert_eq!(item.description, "A test item");
        assert_eq!(item.guid, "unique-id");
        assert_eq!(item.pub_date, "2024-03-21");
        assert_eq!(item.author, "John Doe");
    }
}

//! Module containing data structures for metadata handling in the SSG system.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents metadata for a page or content item.
///
/// This struct wraps a `HashMap` to store key-value pairs of metadata.
/// It provides methods to interact with the underlying data in a controlled manner.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Metadata {
    inner: HashMap<String, String>,
}

impl Metadata {
    /// Creates a new `Metadata` instance with the given data.
    ///
    /// # Arguments
    ///
    /// * `data` - A `HashMap` containing the initial metadata key-value pairs.
    ///
    /// # Returns
    ///
    /// A new `Metadata` instance.
    pub fn new(data: HashMap<String, String>) -> Self {
        Metadata { inner: data }
    }

    /// Retrieves the value associated with the given key.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to look up.
    ///
    /// # Returns
    ///
    /// An `Option` containing a reference to the value if the key exists, or `None` if it doesn't.
    pub fn get(&self, key: &str) -> Option<&String> {
        self.inner.get(key)
    }

    /// Retrieves a mutable reference to the value associated with the given key.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to look up.
    ///
    /// # Returns
    ///
    /// An `Option` containing a mutable reference to the value if the key exists, or `None` if it doesn't.
    pub fn get_mut(&mut self, key: &str) -> Option<&mut String> {
        self.inner.get_mut(key)
    }

    /// Inserts a key-value pair into the metadata.
    ///
    /// If the key already exists, the old value is replaced and returned.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to insert.
    /// * `value` - The value to associate with the key.
    ///
    /// # Returns
    ///
    /// An `Option` containing the old value if the key existed, or `None` if it didn't.
    pub fn insert(
        &mut self,
        key: String,
        value: String,
    ) -> Option<String> {
        self.inner.insert(key, value)
    }

    /// Checks if the metadata contains the given key.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to check for.
    ///
    /// # Returns
    ///
    /// `true` if the key exists, `false` otherwise.
    pub fn contains_key(&self, key: &str) -> bool {
        self.inner.contains_key(key)
    }

    /// Consumes the `Metadata` instance and returns the inner `HashMap`.
    ///
    /// # Returns
    ///
    /// The inner `HashMap<String, String>` containing all metadata key-value pairs.
    pub fn into_inner(self) -> HashMap<String, String> {
        self.inner
    }

    /// Returns a reference to the inner `HashMap`.
    ///
    /// # Returns
    ///
    /// A reference to the inner `HashMap<String, String>` containing all metadata key-value pairs.
    pub fn as_inner_ref(&self) -> &HashMap<String, String> {
        &self.inner
    }
}

/// Holds collections of meta tags for different platforms and categories.
///
/// This struct includes fields for Apple-specific meta tags, primary meta tags, Open Graph meta tags,
/// Microsoft-specific meta tags, and Twitter-specific meta tags. Each field contains a string
/// representation of the HTML meta tags for its respective category or platform.
#[derive(
    Debug, Default, PartialEq, Eq, Hash, Clone, Serialize, Deserialize,
)]
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
    ///
    /// # Returns
    ///
    /// A new `MetaTagGroups` instance with empty strings for all fields.
    pub fn new() -> Self {
        MetaTagGroups::default()
    }

    /// Returns the value for the given key, if it exists.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to look up. Valid keys are "apple", "primary", "og", "ms", and "twitter".
    ///
    /// # Returns
    ///
    /// An `Option` containing a reference to the value if the key exists, or `None` if it doesn't.
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

    /// Checks if all fields are empty.
    ///
    /// # Returns
    ///
    /// `true` if all fields are empty strings, `false` otherwise.
    pub fn is_empty(&self) -> bool {
        self.apple.is_empty()
            && self.primary.is_empty()
            && self.og.is_empty()
            && self.ms.is_empty()
            && self.twitter.is_empty()
    }
}

/// Represents a single meta tag.
///
/// This struct holds the name and content of a meta tag, which can be used to
/// generate a complete meta tag in HTML format.
#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
pub struct MetaTag {
    /// The name of the meta tag.
    pub name: String,
    /// The content of the meta tag.
    pub value: String,
}

impl MetaTag {
    /// Creates a new `MetaTag` instance with the given name and value.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the meta tag.
    /// * `value` - The content of the meta tag.
    ///
    /// # Returns
    ///
    /// A new `MetaTag` instance.
    pub fn new(name: String, value: String) -> Self {
        MetaTag { name, value }
    }

    /// Generates a complete meta tag in HTML format.
    ///
    /// # Returns
    ///
    /// A string representing the complete meta tag in HTML format.
    pub fn generate(&self) -> String {
        format!(
            "<meta content=\"{}\" name=\"{}\">",
            self.value, self.name
        )
    }

    /// Generates a complete list of meta tags in HTML format.
    ///
    /// # Arguments
    ///
    /// * `metatags` - A slice containing the `MetaTag` instances.
    ///
    /// # Returns
    ///
    /// A string representing the complete list of meta tags in HTML format.
    pub fn generate_metatags(metatags: &[MetaTag]) -> String {
        metatags.iter().map(MetaTag::generate).collect()
    }
}

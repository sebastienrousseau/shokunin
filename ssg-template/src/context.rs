use std::collections::HashMap;

/// Represents the context for template rendering.
///
/// `Context` holds key-value pairs that can be used to populate
/// placeholders in a template during the rendering process.
#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct Context {
    /// The internal storage for context key-value pairs.
    pub elements: HashMap<String, String>,
}

impl Context {
    /// Creates a new, empty `Context`.
    ///
    /// # Examples
    ///
    /// ```
    /// use ssg_template::Context;
    ///
    /// let context = Context::new();
    /// assert!(context.elements.is_empty());
    /// ```
    pub fn new() -> Context {
        Context {
            elements: HashMap::new(),
        }
    }

    /// Sets a key-value pair in the context.
    ///
    /// If the key already exists, its value will be updated.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to set.
    /// * `value` - The value to associate with the key.
    ///
    /// # Examples
    ///
    /// ```
    /// use ssg_template::Context;
    ///
    /// let mut context = Context::new();
    /// context.set("name".to_string(), "Alice".to_string()); // Corrected to use String
    /// assert_eq!(context.get("name"), Some(&"Alice".to_string()));
    /// ```
    ///
    pub fn set(&mut self, key: String, value: String) {
        self.elements.insert(key, value);
    }

    /// Retrieves the value associated with a key from the context.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to look up.
    ///
    /// # Returns
    ///
    /// An `Option` containing a reference to the value if the key exists,
    /// or `None` if it doesn't.
    ///
    /// # Examples
    ///
    /// ```
    /// use ssg_template::Context;
    ///
    /// let mut context = Context::new();
    /// context.set("name".to_string(), "Bob".to_string());
    /// assert_eq!(context.get("name"), Some(&"Bob".to_string()));
    /// assert_eq!(context.get("age"), None);
    /// ```
    pub fn get(&self, key: &str) -> Option<&String> {
        self.elements.get(key)
    }

    /// Removes a key-value pair from the context.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to remove.
    ///
    /// # Returns
    ///
    /// The value associated with the key, or `None` if the key didn't exist.
    ///
    /// # Examples
    ///
    /// ```
    /// use ssg_template::Context;
    ///
    /// let mut context = Context::new();
    /// context.set("name".to_string(), "Alice".to_string());
    /// assert_eq!(context.remove("name"), Some("Alice".to_string()));
    /// assert_eq!(context.get("name"), None);
    /// ```
    pub fn remove(&mut self, key: &str) -> Option<String> {
        self.elements.remove(key)
    }
}

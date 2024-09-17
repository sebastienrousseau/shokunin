use std::collections::HashMap;

/// Represents the context for template rendering.
///
/// `Context` holds key-value pairs that can be used to populate
/// placeholders in a template during the rendering process.
#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct Context<'a> {
    /// The internal storage for context key-value pairs.
    pub elements: HashMap<&'a str, &'a str>,
}

impl<'a> Context<'a> {
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
    pub fn new() -> Context<'a> {
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
    /// context.set("name", "Alice");
    /// assert_eq!(context.get("name"), Some(&"Alice"));
    /// ```
    pub fn set(&mut self, key: &'a str, value: &'a str) {
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
    /// context.set("name", "Bob");
    /// assert_eq!(context.get("name"), Some(&"Bob"));
    /// assert_eq!(context.get("age"), None);
    /// ```
    pub fn get(&self, key: &'a str) -> Option<&&'a str> {
        self.elements.get(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_operations() {
        let mut context = Context::new();
        assert!(context.elements.is_empty());

        context.set("name", "Charlie");
        assert_eq!(context.get("name"), Some(&"Charlie"));

        context.set("name", "David");
        assert_eq!(context.get("name"), Some(&"David"));

        assert_eq!(context.get("age"), None);
    }
}

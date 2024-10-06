use ssg_template::Context;

/// Unit tests for Context operations.
#[cfg(test)]
mod context_tests {
    use super::*;

    /// Test creating a new, empty context.
    #[test]
    fn test_context_new() {
        let context = Context::new();
        assert!(context.elements.is_empty());
    }

    /// Test setting and getting a key-value pair in the context.
    #[test]
    fn test_context_set_and_get() {
        let mut context = Context::new();
        context.set("name".to_string(), "Alice".to_string());
        assert_eq!(context.get("name"), Some(&"Alice".to_string()));
    }

    /// Test updating an existing key's value in the context.
    #[test]
    fn test_context_update_existing_key() {
        let mut context = Context::new();
        context.set("name".to_string(), "Alice".to_string());
        context.set("name".to_string(), "Bob".to_string());
        assert_eq!(context.get("name"), Some(&"Bob".to_string()));
    }

    /// Test setting a key with the same value to ensure no issues arise.
    #[test]
    fn test_context_set_same_value() {
        let mut context = Context::new();
        context.set("name".to_string(), "Alice".to_string());
        context.set("name".to_string(), "Alice".to_string()); // Set same value again
        assert_eq!(context.get("name"), Some(&"Alice".to_string()));
    }

    /// Test retrieving a nonexistent key from the context.
    #[test]
    fn test_context_get_nonexistent_key() {
        let context = Context::new();
        assert_eq!(context.get("nonexistent"), None);
    }

    /// Test setting and getting multiple entries in the context.
    #[test]
    fn test_context_multiple_entries() {
        let mut context = Context::new();
        context.set("name".to_string(), "Alice".to_string());
        context.set("age".to_string(), "30".to_string());
        assert_eq!(context.get("name"), Some(&"Alice".to_string()));
        assert_eq!(context.get("age"), Some(&"30".to_string()));
    }

    /// Test removing a key from the context.
    #[test]
    fn test_context_remove_key() {
        let mut context = Context::new();
        context.set("name".to_string(), "Alice".to_string());
        assert_eq!(context.remove("name"), Some("Alice".to_string()));
        assert_eq!(context.get("name"), None);
    }

    /// Test attempting to remove a non-existent key.
    #[test]
    fn test_context_remove_nonexistent_key() {
        let mut context = Context::new();
        assert_eq!(context.remove("nonexistent"), None); // Key doesn't exist
    }

    /// Test setting an empty key and value.
    #[test]
    fn test_context_empty_key_value() {
        let mut context = Context::new();
        context.set("".to_string(), "".to_string());
        assert_eq!(context.get(""), Some(&"".to_string()));
    }

    /// Test handling non-ASCII keys and values.
    #[test]
    fn test_context_non_ascii_keys_values() {
        let mut context = Context::new();
        context.set("名前".to_string(), "アリス".to_string()); // Japanese for "name" and "Alice"
        assert_eq!(context.get("名前"), Some(&"アリス".to_string()));
    }

    /// Test cloning the context and ensuring independence.
    #[test]
    fn test_context_clone() {
        let mut context = Context::new();
        context.set("name".to_string(), "Alice".to_string());
        let mut cloned_context = context.clone();
        cloned_context.set("name".to_string(), "Bob".to_string());

        assert_eq!(context.get("name"), Some(&"Alice".to_string())); // Original unchanged
        assert_eq!(
            cloned_context.get("name"),
            Some(&"Bob".to_string())
        ); // Cloned modified
    }

    /// Test getting a removed key to ensure it returns None.
    #[test]
    fn test_context_get_after_remove() {
        let mut context = Context::new();
        context.set("name".to_string(), "Alice".to_string());
        context.remove("name");
        assert_eq!(context.get("name"), None); // After removal, should be None
    }

    /// Test large number of entries in context.
    #[test]
    fn test_context_large_entries() {
        let mut context = Context::new();
        for i in 0..1000 {
            let key = format!("key{}", i);
            let value = format!("value{}", i);
            context.set(key, value);
        }
        assert_eq!(
            context.get("key999"),
            Some(&"value999".to_string())
        );
        assert_eq!(context.get("key1000"), None);
    }

    /// Test context behaviour after mutation (ensuring immutability of previous states).
    #[test]
    fn test_context_mutation_preserves_old_state() {
        let mut context = Context::new();
        context.set("name".to_string(), "Alice".to_string());
        assert_eq!(context.get("name"), Some(&"Alice".to_string()));

        context.set("age".to_string(), "30".to_string());
        assert_eq!(context.get("name"), Some(&"Alice".to_string())); // Name should still be "Alice"
        assert_eq!(context.get("age"), Some(&"30".to_string()));
    }
}

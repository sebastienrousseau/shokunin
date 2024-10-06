use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Represents a cached item with its value and expiration time.
struct CachedItem<T> {
    value: T,
    expiration: Instant,
}

/// A simple cache implementation with expiration.
pub struct Cache<K, V> {
    items: HashMap<K, CachedItem<V>>,
    ttl: Duration,
}

impl<K: std::hash::Hash + Eq, V: Clone> Cache<K, V> {
    /// Creates a new Cache with the specified time-to-live (TTL) for items.
    ///
    /// # Arguments
    ///
    /// * `ttl` - The time-to-live for cached items.
    ///
    /// # Example
    ///
    /// ```
    /// use ssg_template::cache::Cache;
    /// use std::time::Duration;
    ///
    /// let cache: Cache<String, String> = Cache::new(Duration::from_secs(60));
    /// ```
    pub fn new(ttl: Duration) -> Self {
        Cache {
            items: HashMap::new(),
            ttl,
        }
    }

    /// Inserts a key-value pair into the cache.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to insert.
    /// * `value` - The value to insert.
    ///
    /// # Example
    ///
    /// ```
    /// use ssg_template::cache::Cache;
    /// use std::time::Duration;
    ///
    /// let mut cache = Cache::new(Duration::from_secs(60));
    /// cache.insert("key".to_string(), "value".to_string());
    /// ```
    pub fn insert(&mut self, key: K, value: V) {
        let expiration = Instant::now() + self.ttl;
        self.items.insert(key, CachedItem { value, expiration });
    }

    /// Retrieves a value from the cache if it exists and hasn't expired.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to look up.
    ///
    /// # Returns
    ///
    /// An `Option` containing the value if it exists and hasn't expired, or `None` otherwise.
    ///
    /// # Example
    ///
    /// ```
    /// use ssg_template::cache::Cache;
    /// use std::time::Duration;
    ///
    /// let mut cache = Cache::new(Duration::from_secs(60));
    /// cache.insert("key".to_string(), "value".to_string());
    ///
    /// assert_eq!(cache.get(&"key".to_string()), Some(&"value".to_string()));
    /// ```
    pub fn get(&self, key: &K) -> Option<&V> {
        self.items.get(key).and_then(|item| {
            if item.expiration > Instant::now() {
                Some(&item.value)
            } else {
                None
            }
        })
    }

    /// Removes expired items from the cache.
    ///
    /// This method should be called periodically to clean up the cache.
    ///
    pub fn remove_expired(&mut self) {
        self.items
            .retain(|_, item| item.expiration > Instant::now());
    }
}

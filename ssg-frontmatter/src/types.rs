use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::fmt;
use std::str::FromStr;

/// Format enum represents different formats that can be used for frontmatter serialization/deserialization.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    /// YAML format.
    Yaml,
    /// TOML format.
    Toml,
    /// JSON format.
    Json,
    /// Unsupported format.
    Unsupported,
}

/// A flexible value type that can hold various types such as null, strings,
/// numbers, booleans, arrays, objects (in the form of frontmatter), and tagged values.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum Value {
    /// Represents a null value.
    Null,
    /// Represents a string value.
    String(String),
    /// Represents a numeric value.
    Number(f64),
    /// Represents a boolean value.
    Boolean(bool),
    /// Represents an array of values.
    Array(Vec<Value>),
    /// Represents an object (frontmatter).
    Object(Box<Frontmatter>),
    /// Represents a tagged value, containing a tag and a value.
    Tagged(String, Box<Value>),
}

/// Represents the frontmatter, a collection of key-value pairs where the value
/// is represented using the `Value` enum to support various data types.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Frontmatter(pub HashMap<String, Value>);

impl Frontmatter {
    /// Creates a new, empty frontmatter.
    #[must_use]
    pub fn new() -> Self {
        Frontmatter(HashMap::new())
    }

    /// Inserts a key-value pair into the frontmatter.
    ///
    /// # Arguments
    ///
    /// * `key` - The key for the entry.
    /// * `value` - The value associated with the key.
    ///
    /// # Returns
    ///
    /// An option containing the old value if it was replaced.
    #[must_use]
    pub fn insert(
        &mut self,
        key: String,
        value: Value,
    ) -> Option<Value> {
        self.0.insert(key, value)
    }

    /// Retrieves a reference to a value associated with a key.
    #[must_use]
    pub fn get(&self, key: &str) -> Option<&Value> {
        self.0.get(key)
    }

    /// Retrieves a mutable reference to a value associated with a key.
    ///
    /// # Arguments
    ///
    /// * `key` - The key for which to retrieve the mutable reference.
    ///
    /// # Returns
    ///
    /// An `Option` containing a mutable reference to the value, or `None` if the key is not present.
    pub fn get_mut(&mut self, key: &str) -> Option<&mut Value> {
        self.0.get_mut(key)
    }

    /// Removes a key-value pair from the frontmatter.
    #[must_use]
    pub fn remove(&mut self, key: &str) -> Option<Value> {
        self.0.remove(key)
    }

    /// Checks if the frontmatter contains a given key.
    pub fn contains_key(&self, key: &str) -> bool {
        self.0.contains_key(key)
    }

    /// Returns the number of entries in the frontmatter.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Checks if the frontmatter is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns an iterator over the key-value pairs of the frontmatter.
    pub fn iter(
        &self,
    ) -> std::collections::hash_map::Iter<String, Value> {
        self.0.iter()
    }

    /// Returns a mutable iterator over the key-value pairs of the frontmatter.
    pub fn iter_mut(
        &mut self,
    ) -> std::collections::hash_map::IterMut<String, Value> {
        self.0.iter_mut()
    }

    /// Merges another frontmatter into this one. If a key exists, it will be overwritten.
    pub fn merge(&mut self, other: Frontmatter) {
        self.0.extend(other.0);
    }

    /// Checks if a given key exists and its value is `null`.
    pub fn is_null(&self, key: &str) -> bool {
        matches!(self.get(key), Some(Value::Null))
    }
}

impl Default for Frontmatter {
    fn default() -> Self {
        Self::new()
    }
}

/// Implement `IntoIterator` for `Frontmatter` to allow idiomatic iteration.
impl IntoIterator for Frontmatter {
    type Item = (String, Value);
    type IntoIter = std::collections::hash_map::IntoIter<String, Value>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

/// Implement `FromIterator` for `Frontmatter` to create a frontmatter from an iterator.
impl FromIterator<(String, Value)> for Frontmatter {
    /// Creates a `Frontmatter` from an iterator of key-value pairs.
    ///
    /// # Arguments
    ///
    /// * `iter` - An iterator of key-value pairs where the key is a `String` and the value is a `Value`.
    ///
    /// # Returns
    ///
    /// A `Frontmatter` containing the key-value pairs from the iterator.
    fn from_iter<I: IntoIterator<Item = (String, Value)>>(
        iter: I,
    ) -> Self {
        let mut fm = Frontmatter::new();
        for (key, value) in iter {
            let _ = fm.insert(key, value);
        }
        fm
    }
}

/// Implement `Display` for `Frontmatter` to allow easy printing with escaped characters.
impl fmt::Display for Frontmatter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{{")?;

        // Use a BTreeMap to ensure consistent key order (sorted by key)
        let mut sorted_map = BTreeMap::new();
        for (key, value) in &self.0 {
            sorted_map.insert(key, value);
        }

        for (i, (key, value)) in sorted_map.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "\"{}\": {}", escape_str(key), value)?;
        }

        write!(f, "}}")
    }
}

/// Implement `Display` for `Value` to allow easy printing with escaped characters.
impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Null => write!(f, "null"),
            Value::String(s) => write!(f, "\"{}\"", escape_str(s)),
            Value::Number(n) => {
                if n.fract() == 0.0 {
                    write!(f, "{:.0}", n)
                } else {
                    write!(f, "{}", n)
                }
            }
            Value::Boolean(b) => write!(f, "{}", b),
            Value::Array(arr) => {
                write!(f, "[")?;
                for (i, v) in arr.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", v)?;
                }
                write!(f, "]")
            }
            Value::Object(obj) => write!(f, "{}", obj),
            Value::Tagged(tag, val) => {
                write!(f, "\"{}\": {}", escape_str(tag), val)
            }
        }
    }
}

/// Escapes special characters in a string (e.g., backslashes and quotes).
fn escape_str(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

impl Value {
    /// Returns the value as a string, if it is of type `String`.
    pub fn as_str(&self) -> Option<&str> {
        if let Value::String(ref s) = self {
            Some(s)
        } else {
            None
        }
    }

    /// Returns the value as a float, if it is of type `Number`.
    pub fn as_f64(&self) -> Option<f64> {
        if let Value::Number(n) = self {
            Some(*n)
        } else {
            None
        }
    }

    /// Returns the value as a boolean, if it is of type `Boolean`.
    pub fn as_bool(&self) -> Option<bool> {
        if let Value::Boolean(b) = self {
            Some(*b)
        } else {
            None
        }
    }

    /// Returns the value as an array, if it is of type `Array`.
    pub fn as_array(&self) -> Option<&Vec<Value>> {
        if let Value::Array(ref arr) = self {
            Some(arr)
        } else {
            None
        }
    }

    /// Returns the value as an object (frontmatter), if it is of type `Object`.
    pub fn as_object(&self) -> Option<&Frontmatter> {
        if let Value::Object(ref obj) = self {
            Some(obj)
        } else {
            None
        }
    }

    /// Returns the value as a tagged value, if it is of type `Tagged`.
    pub fn as_tagged(&self) -> Option<(&str, &Value)> {
        if let Value::Tagged(ref tag, ref val) = self {
            Some((tag.as_str(), val.as_ref()))
        } else {
            None
        }
    }

    /// Checks if the value is of type `Null`.
    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }

    /// Checks if the value is of type `String`.
    ///
    /// # Returns
    ///
    /// `true` if the value is a `String`, otherwise `false`.
    pub fn is_string(&self) -> bool {
        matches!(self, Value::String(_))
    }

    /// Checks if the value is of type `Number`.
    ///
    /// # Returns
    ///
    /// `true` if the value is a `Number`, otherwise `false`.
    pub fn is_number(&self) -> bool {
        matches!(self, Value::Number(_))
    }

    /// Checks if the value is of type `Boolean`.
    ///
    /// # Returns
    ///
    /// `true` if the value is a `Boolean`, otherwise `false`.
    pub fn is_boolean(&self) -> bool {
        matches!(self, Value::Boolean(_))
    }

    /// Checks if the value is of type `Array`.
    ///
    /// # Returns
    ///
    /// `true` if the value is an `Array`, otherwise `false`.
    pub fn is_array(&self) -> bool {
        matches!(self, Value::Array(_))
    }

    /// Checks if the value is of type `Object`.
    ///
    /// # Returns
    ///
    /// `true` if the value is an `Object`, otherwise `false`.
    pub fn is_object(&self) -> bool {
        matches!(self, Value::Object(_))
    }

    /// Checks if the value is of type `Tagged`.
    ///
    /// # Returns
    ///
    /// `true` if the value is `Tagged`, otherwise `false`.
    pub fn is_tagged(&self) -> bool {
        matches!(self, Value::Tagged(_, _))
    }

    /// Returns the length of the array if the value is an array, otherwise returns `None`.
    pub fn array_len(&self) -> Option<usize> {
        if let Value::Array(ref arr) = self {
            Some(arr.len())
        } else {
            None
        }
    }

    /// Attempts to convert the value into a `Frontmatter`.
    pub fn to_object(self) -> Result<Frontmatter, String> {
        if let Value::Object(obj) = self {
            Ok(*obj)
        } else {
            Err("Value is not an object".into())
        }
    }

    /// Converts the value to a string representation regardless of its type.
    pub fn to_string_representation(&self) -> String {
        format!("{}", self)
    }

    /// Attempts to convert the value into a `String`.
    ///
    /// # Returns
    ///
    /// A `Result` containing the string value, or an error message if the value is not a string.
    pub fn into_string(self) -> Result<String, String> {
        if let Value::String(s) = self {
            Ok(s)
        } else {
            Err("Value is not a string".into())
        }
    }

    /// Attempts to convert the value into an `f64`.
    ///
    /// # Returns
    ///
    /// A `Result` containing the float value, or an error message if the value is not a number.
    pub fn into_f64(self) -> Result<f64, String> {
        if let Value::Number(n) = self {
            Ok(n)
        } else {
            Err("Value is not a number".into())
        }
    }

    /// Attempts to convert the value into a `bool`.
    ///
    /// # Returns
    ///
    /// A `Result` containing the boolean value, or an error message if the value is not a boolean.
    pub fn into_bool(self) -> Result<bool, String> {
        if let Value::Boolean(b) = self {
            Ok(b)
        } else {
            Err("Value is not a boolean".into())
        }
    }

    /// Attempts to get a mutable reference to the array if the value is an array.
    pub fn get_mut_array(&mut self) -> Option<&mut Vec<Value>> {
        if let Value::Array(ref mut arr) = self {
            Some(arr)
        } else {
            None
        }
    }
}

impl From<&str> for Value {
    fn from(s: &str) -> Self {
        Value::String(s.to_string())
    }
}

impl From<String> for Value {
    fn from(s: String) -> Self {
        Value::String(s)
    }
}

impl From<f64> for Value {
    fn from(n: f64) -> Self {
        Value::Number(n)
    }
}

impl From<bool> for Value {
    fn from(b: bool) -> Self {
        Value::Boolean(b)
    }
}

impl FromIterator<Value> for Value {
    fn from_iter<I: IntoIterator<Item = Value>>(iter: I) -> Self {
        Value::Array(iter.into_iter().collect())
    }
}

/// Implement the Default trait for `Value`, with the default being `Null`.
impl Default for Value {
    fn default() -> Self {
        Value::Null
    }
}

/// Implement the Default trait for `Format`, with the default being `Json`.
impl Default for Format {
    fn default() -> Self {
        Format::Json
    }
}

/// Implement `FromStr` for `Value` to allow parsing of simple types from strings.
impl FromStr for Value {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.eq_ignore_ascii_case("null") {
            Ok(Value::Null)
        } else if s.eq_ignore_ascii_case("true") {
            Ok(Value::Boolean(true))
        } else if s.eq_ignore_ascii_case("false") {
            Ok(Value::Boolean(false))
        } else if let Ok(n) = s.parse::<f64>() {
            Ok(Value::Number(n))
        } else {
            Ok(Value::String(s.to_string()))
        }
    }
}

/// Implement conversion from `Value` to `serde_json::Value` for ease of use in JSON-related operations.
impl From<Value> for serde_json::Value {
    fn from(value: Value) -> Self {
        match value {
            Value::Null => serde_json::Value::Null,
            Value::String(s) => serde_json::Value::String(s),
            Value::Number(n) => serde_json::Number::from_f64(n)
                .map(serde_json::Value::Number)
                .unwrap_or(serde_json::Value::Null),
            Value::Boolean(b) => serde_json::Value::Bool(b),
            Value::Array(arr) => serde_json::Value::Array(
                arr.into_iter().map(serde_json::Value::from).collect(),
            ),
            Value::Object(obj) => serde_json::Value::Object(
                obj.0
                    .into_iter()
                    .map(|(k, v)| (k, serde_json::Value::from(v)))
                    .collect(),
            ),
            Value::Tagged(tag, v) => {
                let mut map = serde_json::Map::new();
                map.insert(tag, serde_json::Value::from(*v));
                serde_json::Value::Object(map)
            }
        }
    }
}

/// Implement conversion from `Value` to `serde_yml::Value` for YAML-related operations.
impl From<Value> for serde_yml::Value {
    fn from(value: Value) -> Self {
        match value {
            Value::Null => serde_yml::Value::Null,
            Value::String(s) => serde_yml::Value::String(s),
            Value::Number(n) => serde_yml::Value::Number(n.into()),
            Value::Boolean(b) => serde_yml::Value::Bool(b),
            Value::Array(arr) => serde_yml::Value::Sequence(
                arr.into_iter().map(serde_yml::Value::from).collect(),
            ),
            Value::Object(obj) => {
                let map = obj
                    .0
                    .into_iter()
                    .map(|(k, v)| {
                        (
                            serde_yml::Value::String(k),
                            serde_yml::Value::from(v),
                        )
                    })
                    .collect();
                serde_yml::Value::Mapping(map)
            }
            Value::Tagged(tag, v) => {
                let mut map = serde_yml::Mapping::new();
                map.insert(
                    serde_yml::Value::String(tag),
                    serde_yml::Value::from(*v),
                );
                serde_yml::Value::Mapping(map)
            }
        }
    }
}

/// Implement conversion from `Value` to `toml::Value` for TOML-related operations.
impl From<Value> for toml::Value {
    fn from(value: Value) -> Self {
        match value {
            Value::Null => toml::Value::String(String::new()), // TOML has no explicit null, empty string as a placeholder.
            Value::String(s) => toml::Value::String(s),
            Value::Number(n) => toml::Value::Float(n),
            Value::Boolean(b) => toml::Value::Boolean(b),
            Value::Array(arr) => toml::Value::Array(
                arr.into_iter().map(toml::Value::from).collect(),
            ),
            Value::Object(obj) => toml::Value::Table(
                obj.0
                    .into_iter()
                    .map(|(k, v)| (k, toml::Value::from(v)))
                    .collect(),
            ),
            Value::Tagged(tag, v) => {
                let mut map = toml::map::Map::new();
                map.insert(tag, toml::Value::from(*v));
                toml::Value::Table(map)
            }
        }
    }
}

impl From<toml::Value> for Value {
    fn from(value: toml::Value) -> Self {
        match value {
            toml::Value::String(s) if s.is_empty() => Value::Null, // Treat empty strings as Null
            toml::Value::String(s) => Value::String(s),
            toml::Value::Float(n) => Value::Number(n),
            toml::Value::Integer(n) => Value::Number(n as f64),
            toml::Value::Boolean(b) => Value::Boolean(b),
            toml::Value::Array(arr) => {
                Value::Array(arr.into_iter().map(Value::from).collect())
            }
            toml::Value::Table(obj) => {
                Value::Object(Box::new(Frontmatter(
                    obj.into_iter()
                        .map(|(k, v)| (k, Value::from(v)))
                        .collect(),
                )))
            }
            _ => Value::Null, // Handle other TOML types as Null
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;
    use serde_yml;
    use std::f64::consts::PI;
    use toml;

    #[test]
    fn test_frontmatter_new() {
        let fm = Frontmatter::new();
        assert!(fm.is_empty());
        assert_eq!(fm.len(), 0);
    }

    #[test]
    fn test_frontmatter_insert_and_get() {
        let mut fm = Frontmatter::new();
        let key = "title".to_string();
        let value = Value::String("Hello World".to_string());
        let _ = fm.insert(key.clone(), value.clone());

        assert_eq!(fm.get(&key), Some(&value));
    }

    #[test]
    fn test_frontmatter_remove() {
        let mut fm = Frontmatter::new();
        let key = "title".to_string();
        let value = Value::String("Hello World".to_string());
        let _ = fm.insert(key.clone(), value.clone());

        let removed = fm.remove(&key);
        assert_eq!(removed, Some(value));
        assert!(fm.get(&key).is_none());
    }

    #[test]
    fn test_frontmatter_contains_key() {
        let mut fm = Frontmatter::new();
        let key = "title".to_string();
        let value = Value::String("Hello World".to_string());
        let _ = fm.insert(key.clone(), value.clone());

        assert!(fm.contains_key(&key));
        let _ = fm.remove(&key);
        assert!(!fm.contains_key(&key));
    }

    #[test]
    fn test_frontmatter_len_and_is_empty() {
        let mut fm = Frontmatter::new();
        assert_eq!(fm.len(), 0);
        assert!(fm.is_empty());

        let _ = fm.insert("key1".to_string(), Value::Null);
        assert_eq!(fm.len(), 1);
        assert!(!fm.is_empty());

        let _ = fm.insert("key2".to_string(), Value::Boolean(true));
        assert_eq!(fm.len(), 2);

        let _ = fm.remove("key1");
        assert_eq!(fm.len(), 1);

        let _ = fm.remove("key2");
        assert_eq!(fm.len(), 0);
        assert!(fm.is_empty());
    }

    #[test]
    fn test_frontmatter_iter() {
        let mut fm = Frontmatter::new();
        let _ = fm.insert(
            "title".to_string(),
            Value::String("Hello".to_string()),
        );
        let _ = fm.insert("views".to_string(), Value::Number(100.0));

        let mut keys = vec![];
        let mut values = vec![];

        for (k, v) in fm.iter() {
            keys.push(k.clone());
            values.push(v.clone());
        }

        keys.sort();
        values.sort_by(|a, b| {
            format!("{:?}", a).cmp(&format!("{:?}", b))
        });

        assert_eq!(
            keys,
            vec!["title".to_string(), "views".to_string()]
        );
        assert_eq!(
            values,
            vec![
                Value::Number(100.0),
                Value::String("Hello".to_string())
            ]
        );
    }

    #[test]
    fn test_frontmatter_iter_mut() {
        let mut fm = Frontmatter::new();
        let _ = fm.insert("count".to_string(), Value::Number(1.0));

        for (_, v) in fm.iter_mut() {
            if let Value::Number(n) = v {
                *n += 1.0;
            }
        }

        assert_eq!(fm.get("count"), Some(&Value::Number(2.0)));
    }

    #[test]
    fn test_value_as_str() {
        let value = Value::String("Hello".to_string());
        assert_eq!(value.as_str(), Some("Hello"));

        let value = Value::Number(42.0);
        assert_eq!(value.as_str(), None);
    }

    #[test]
    fn test_value_as_f64() {
        let value = Value::Number(42.0);
        assert_eq!(value.as_f64(), Some(42.0));

        let value = Value::String("Not a number".to_string());
        assert_eq!(value.as_f64(), None);
    }

    #[test]
    fn test_value_as_bool() {
        let value = Value::Boolean(true);
        assert_eq!(value.as_bool(), Some(true));

        let value = Value::String("Not a bool".to_string());
        assert_eq!(value.as_bool(), None);
    }

    #[test]
    fn test_value_as_array() {
        let value =
            Value::Array(vec![Value::Null, Value::Boolean(false)]);
        assert!(value.as_array().is_some());
        let array = value.as_array().unwrap();
        assert_eq!(array.len(), 2);
        assert_eq!(array[0], Value::Null);
        assert_eq!(array[1], Value::Boolean(false));

        let value = Value::String("Not an array".to_string());
        assert!(value.as_array().is_none());
    }

    #[test]
    fn test_value_as_object() {
        let mut fm = Frontmatter::new();
        let _ = fm.insert(
            "key".to_string(),
            Value::String("value".to_string()),
        );
        let value = Value::Object(Box::new(fm.clone()));
        assert!(value.as_object().is_some());
        assert_eq!(value.as_object().unwrap(), &fm);

        let value = Value::String("Not an object".to_string());
        assert!(value.as_object().is_none());
    }

    #[test]
    fn test_value_as_tagged() {
        let inner_value = Value::Boolean(true);
        let value = Value::Tagged(
            "isActive".to_string(),
            Box::new(inner_value.clone()),
        );
        assert!(value.as_tagged().is_some());
        let (tag, val) = value.as_tagged().unwrap();
        assert_eq!(tag, "isActive");
        assert_eq!(val, &inner_value);

        let value = Value::String("Not tagged".to_string());
        assert!(value.as_tagged().is_none());
    }

    #[test]
    fn test_value_is_null() {
        let value = Value::Null;
        assert!(value.is_null());

        let value = Value::String("Not null".to_string());
        assert!(!value.is_null());
    }

    #[test]
    fn test_from_traits() {
        let s: Value = "Hello".into();
        assert_eq!(s, Value::String("Hello".to_string()));

        let s: Value = "Hello".to_string().into();
        assert_eq!(s, Value::String("Hello".to_string()));

        let n: Value = Value::Number(PI);
        assert_eq!(n, Value::Number(PI));

        let b: Value = true.into();
        assert_eq!(b, Value::Boolean(true));
    }

    #[test]
    fn test_default_traits() {
        let default_value: Value = Default::default();
        assert_eq!(default_value, Value::Null);

        let default_format: Format = Default::default();
        assert_eq!(default_format, Format::Json);
    }

    #[test]
    fn test_value_conversion_to_serde_json() {
        let mut fm = Frontmatter::new();
        let _ = fm.insert(
            "title".to_string(),
            Value::String("My Post".to_string()),
        );
        let _ = fm.insert("views".to_string(), Value::Number(100.0));
        let _ =
            fm.insert("published".to_string(), Value::Boolean(true));
        let _ = fm.insert(
            "tags".to_string(),
            Value::Array(vec![
                Value::String("rust".to_string()),
                Value::String("serde".to_string()),
            ]),
        );

        let value = Value::Object(Box::new(fm.clone()));
        let json_value: serde_json::Value = value.into();

        let expected = serde_json::json!({
            "title": "My Post",
            "views": 100.0,
            "published": true,
            "tags": ["rust", "serde"]
        });

        assert_eq!(json_value, expected);
    }

    #[test]
    fn test_value_conversion_to_serde_yml() {
        let mut fm = Frontmatter::new();
        let _ = fm.insert(
            "title".to_string(),
            Value::String("My Post".to_string()),
        );
        let _ = fm.insert("views".to_string(), Value::Number(100.0));
        let _ =
            fm.insert("published".to_string(), Value::Boolean(true));
        let _ = fm.insert(
            "tags".to_string(),
            Value::Array(vec![
                Value::String("rust".to_string()),
                Value::String("serde".to_string()),
            ]),
        );

        let value = Value::Object(Box::new(fm.clone()));
        let yml_value: serde_yml::Value = value.into();

        let mut expected_map = serde_yml::Mapping::new();
        expected_map.insert(
            serde_yml::Value::String("title".to_string()),
            serde_yml::Value::String("My Post".to_string()),
        );
        expected_map.insert(
            serde_yml::Value::String("views".to_string()),
            serde_yml::Value::Number(100.0.into()),
        );
        expected_map.insert(
            serde_yml::Value::String("published".to_string()),
            serde_yml::Value::Bool(true),
        );
        expected_map.insert(
            serde_yml::Value::String("tags".to_string()),
            serde_yml::Value::Sequence(vec![
                serde_yml::Value::String("rust".to_string()),
                serde_yml::Value::String("serde".to_string()),
            ]),
        );

        let expected = serde_yml::Value::Mapping(expected_map);
        assert_eq!(yml_value, expected);
    }

    #[test]
    fn test_value_conversion_to_toml() {
        let mut fm = Frontmatter::new();
        let _ = fm.insert(
            "title".to_string(),
            Value::String("My Post".to_string()),
        );
        let _ = fm.insert("views".to_string(), Value::Number(100.0));
        let _ =
            fm.insert("published".to_string(), Value::Boolean(true));
        let _ = fm.insert(
            "tags".to_string(),
            Value::Array(vec![
                Value::String("rust".to_string()),
                Value::String("serde".to_string()),
            ]),
        );

        let value = Value::Object(Box::new(fm.clone()));
        let toml_value: toml::Value = value.into();

        let mut expected_table = toml::map::Map::new();
        expected_table.insert(
            "title".to_string(),
            toml::Value::String("My Post".to_string()),
        );
        expected_table
            .insert("views".to_string(), toml::Value::Float(100.0));
        expected_table.insert(
            "published".to_string(),
            toml::Value::Boolean(true),
        );
        expected_table.insert(
            "tags".to_string(),
            toml::Value::Array(vec![
                toml::Value::String("rust".to_string()),
                toml::Value::String("serde".to_string()),
            ]),
        );

        let expected = toml::Value::Table(expected_table);
        assert_eq!(toml_value, expected);
    }

    #[test]
    fn test_serialization_deserialization_json() {
        let mut fm = Frontmatter::new();
        let _ = fm.insert(
            "title".to_string(),
            Value::String("JSON Test".to_string()),
        );
        let _ = fm.insert("count".to_string(), Value::Number(10.0));

        let value = Value::Object(Box::new(fm.clone()));

        // Serialize to JSON
        let serialized = serde_json::to_string(&value).unwrap();

        // Deserialize back
        let deserialized: Value =
            serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized, value);
    }

    #[test]
    fn test_serialization_deserialization_yaml() {
        let mut fm = Frontmatter::new();
        let _ = fm.insert(
            "title".to_string(),
            Value::String("YAML Test".to_string()),
        );
        let _ = fm.insert("active".to_string(), Value::Boolean(false));

        let value = Value::Object(Box::new(fm.clone()));

        // Serialize to YAML
        let serialized = serde_yml::to_string(&value).unwrap();

        // Deserialize back
        let deserialized: Value =
            serde_yml::from_str(&serialized).unwrap();

        assert_eq!(deserialized, value);
    }

    #[test]
    fn test_serialization_deserialization_toml() {
        let mut fm = Frontmatter::new();
        let _ = fm.insert(
            "title".to_string(),
            Value::String("TOML Test".to_string()),
        );
        let _ = fm.insert("score".to_string(), Value::Number(95.5));

        let value = Value::Object(Box::new(fm.clone()));

        // Serialize to TOML
        let serialized = toml::to_string(&value).unwrap();

        // Deserialize back
        // Note: Since TOML doesn't support all Value variants directly (e.g., Tagged), ensure your Value type can handle it or adjust accordingly.
        let deserialized: toml::Value =
            toml::from_str(&serialized).unwrap();
        let converted_back: Value = Value::from(deserialized);

        // Due to TOML's limitations, the deserialized structure might differ.
        // Adjust the assertion based on how you handle TOML deserialization.
        // Here, we check if essential fields are correctly deserialized.
        if let Value::Object(obj) = converted_back {
            assert_eq!(
                obj.get("title"),
                Some(&Value::String("TOML Test".to_string()))
            );
            assert_eq!(obj.get("score"), Some(&Value::Number(95.5)));
        } else {
            panic!("Deserialized TOML value is not an object");
        }
    }

    #[test]
    fn test_tagged_value_conversion() {
        let tagged_value = Value::Tagged(
            "custom_tag".to_string(),
            Box::new(Value::String("Tagged".to_string())),
        );

        // Convert to serde_json::Value
        let json_value: serde_json::Value = tagged_value.clone().into();
        let expected_json = serde_json::json!({
            "custom_tag": "Tagged"
        });
        assert_eq!(json_value, expected_json);

        // Convert to serde_yml::Value
        let yaml_value: serde_yml::Value = tagged_value.clone().into();
        let mut expected_yaml_map = serde_yml::Mapping::new();
        expected_yaml_map.insert(
            serde_yml::Value::String("custom_tag".to_string()),
            serde_yml::Value::String("Tagged".to_string()),
        );
        let expected_yaml =
            serde_yml::Value::Mapping(expected_yaml_map);
        assert_eq!(yaml_value, expected_yaml);

        // Convert to toml::Value
        let toml_value: toml::Value = tagged_value.into();
        let mut expected_toml_map = toml::map::Map::new();
        expected_toml_map.insert(
            "custom_tag".to_string(),
            toml::Value::String("Tagged".to_string()),
        );
        let expected_toml = toml::Value::Table(expected_toml_map);
        assert_eq!(toml_value, expected_toml);
    }

    #[test]
    fn test_empty_array_and_object() {
        let empty_array = Value::Array(vec![]);
        assert!(empty_array.as_array().unwrap().is_empty());

        let empty_object = Value::Object(Box::new(Frontmatter::new()));
        assert!(empty_object.as_object().unwrap().is_empty());
    }

    #[test]
    fn test_nested_frontmatter() {
        let mut inner_fm = Frontmatter::new();
        let _ = inner_fm
            .insert("inner_key".to_string(), Value::Boolean(true));

        let mut outer_fm = Frontmatter::new();
        let _ = outer_fm.insert(
            "outer_key".to_string(),
            Value::Object(Box::new(inner_fm.clone())),
        );

        let value = Value::Object(Box::new(outer_fm.clone()));
        let json_value: serde_json::Value = value.into();

        let expected = serde_json::json!({
            "outer_key": {
                "inner_key": true
            }
        });

        assert_eq!(json_value, expected);
    }

    #[test]
    fn test_value_from_toml_null_placeholder() {
        let toml_null = toml::Value::String(String::new());
        let value: Value = toml_null.into();
        assert_eq!(value, Value::Null);
    }

    #[test]
    fn test_frontmatter_merge() {
        let mut fm1 = Frontmatter::new();
        let _ = fm1.insert(
            "key1".to_string(),
            Value::String("value1".to_string()),
        );
        let _ = fm1.insert(
            "key2".to_string(),
            Value::String("value2".to_string()),
        );

        let mut fm2 = Frontmatter::new();
        let _ = fm2.insert(
            "key2".to_string(),
            Value::String("overwritten".to_string()),
        );
        let _ = fm2.insert(
            "key3".to_string(),
            Value::String("value3".to_string()),
        );

        fm1.merge(fm2);

        assert_eq!(
            fm1.get("key1"),
            Some(&Value::String("value1".to_string()))
        );
        assert_eq!(
            fm1.get("key2"),
            Some(&Value::String("overwritten".to_string()))
        );
        assert_eq!(
            fm1.get("key3"),
            Some(&Value::String("value3".to_string()))
        );
    }

    #[test]
    fn test_frontmatter_is_null() {
        let mut fm = Frontmatter::new();
        let _ = fm.insert("key1".to_string(), Value::Null);
        let _ = fm.insert(
            "key2".to_string(),
            Value::String("value2".to_string()),
        );

        assert!(fm.is_null("key1"));
        assert!(!fm.is_null("key2"));
    }

    #[test]
    fn test_value_array_len() {
        let value =
            Value::Array(vec![Value::Null, Value::Boolean(true)]);
        assert_eq!(value.array_len(), Some(2));

        let non_array_value = Value::String("Not an array".to_string());
        assert_eq!(non_array_value.array_len(), None);
    }

    #[test]
    fn test_value_to_object() {
        let mut fm = Frontmatter::new();
        let _ = fm.insert(
            "key".to_string(),
            Value::String("value".to_string()),
        );
        let value = Value::Object(Box::new(fm.clone()));

        assert_eq!(value.to_object().unwrap(), fm);

        let non_object_value =
            Value::String("Not an object".to_string());
        assert!(non_object_value.to_object().is_err());
    }

    #[test]
    fn test_value_to_string_representation() {
        let value = Value::String("Hello World".to_string());
        assert_eq!(value.to_string_representation(), "\"Hello World\"");

        let value = Value::Null;
        assert_eq!(value.to_string_representation(), "null");

        let value = Value::Array(vec![
            Value::Boolean(true),
            Value::Number(42.0),
        ]);
        assert_eq!(value.to_string_representation(), "[true, 42]");
    }

    #[test]
    fn test_value_get_mut_array() {
        let mut value =
            Value::Array(vec![Value::Null, Value::Boolean(false)]);
        let array = value.get_mut_array().unwrap();
        array.push(Value::String("new value".to_string()));

        assert_eq!(
            value,
            Value::Array(vec![
                Value::Null,
                Value::Boolean(false),
                Value::String("new value".to_string())
            ])
        );
    }

    #[test]
    fn test_value_display() {
        let value = Value::String("Hello World".to_string());
        assert_eq!(format!("{}", value), "\"Hello World\"");

        let value = Value::Number(42.0);
        assert_eq!(format!("{}", value), "42");

        let value =
            Value::Array(vec![Value::Boolean(true), Value::Null]);
        assert_eq!(format!("{}", value), "[true, null]");
    }

    #[test]
    fn test_frontmatter_display() {
        let mut fm = Frontmatter::new();
        let _ = fm.insert(
            "key1".to_string(),
            Value::String("value1".to_string()),
        );
        let _ = fm.insert("key2".to_string(), Value::Number(42.0));

        assert_eq!(
            format!("{}", fm),
            "{\"key1\": \"value1\", \"key2\": 42}"
        );
    }

    #[test]
    fn test_from_str_for_value() {
        assert_eq!(Value::from_str("null").unwrap(), Value::Null);
        assert_eq!(
            Value::from_str("true").unwrap(),
            Value::Boolean(true)
        );
        assert_eq!(
            Value::from_str("false").unwrap(),
            Value::Boolean(false)
        );
        assert_eq!(
            Value::from_str("42.0").unwrap(),
            Value::Number(42.0)
        );
        assert_eq!(
            Value::from_str("Hello World").unwrap(),
            Value::String("Hello World".to_string())
        );
    }

    #[test]
    fn test_escape_str() {
        assert_eq!(
            escape_str("Hello \"World\""),
            "Hello \\\"World\\\""
        );
        assert_eq!(escape_str("Path\\to\\file"), "Path\\\\to\\\\file");
    }

    #[test]
    fn test_display_for_value() {
        let value = Value::String("Hello \"World\"".to_string());
        assert_eq!(format!("{}", value), "\"Hello \\\"World\\\"\"");

        let value = Value::Number(42.0);
        assert_eq!(format!("{}", value), "42");

        let value =
            Value::Array(vec![Value::Boolean(true), Value::Null]);
        assert_eq!(format!("{}", value), "[true, null]");
    }

    #[test]
    fn test_display_for_frontmatter() {
        let mut fm = Frontmatter::new();
        let _ = fm.insert(
            "key1".to_string(),
            Value::String("value1".to_string()),
        );
        let _ = fm.insert("key2".to_string(), Value::Number(42.0));

        let output = format!("{}", fm);

        // Check that the output contains both key-value pairs without enforcing the order
        assert!(output.contains("\"key1\": \"value1\""));
        assert!(output.contains("\"key2\": 42"));
    }
}

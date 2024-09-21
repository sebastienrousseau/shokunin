use crate::error::MetadataError;
use dtt::datetime::DateTime;
use regex::Regex;
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use toml::Value as TomlValue;
use yaml_rust2::YamlLoader;

/// Represents metadata for a page or content item.
#[derive(Debug, Default, Clone)]
pub struct Metadata {
    inner: HashMap<String, String>,
}

impl Metadata {
    /// Creates a new `Metadata` instance with the given data.
    pub fn new(data: HashMap<String, String>) -> Self {
        Metadata { inner: data }
    }

    /// Retrieves the value associated with the given key.
    pub fn get(&self, key: &str) -> Option<&String> {
        self.inner.get(key)
    }

    /// Inserts a key-value pair into the metadata.
    pub fn insert(
        &mut self,
        key: String,
        value: String,
    ) -> Option<String> {
        self.inner.insert(key, value)
    }

    /// Checks if the metadata contains the given key.
    pub fn contains_key(&self, key: &str) -> bool {
        self.inner.contains_key(key)
    }

    /// Consumes the `Metadata` instance and returns the inner `HashMap`.
    pub fn into_inner(self) -> HashMap<String, String> {
        self.inner
    }
}

/// Extracts metadata from the content string.
pub fn extract_metadata(
    content: &str,
) -> Result<Metadata, MetadataError> {
    println!("Extracting metadata from content:\n{}", content); // Debugging output

    if let Some(yaml_metadata) = extract_yaml_metadata(content) {
        println!("Extracted YAML metadata: {:?}", yaml_metadata); // Debugging output
        Ok(yaml_metadata)
    } else if let Some(toml_metadata) = extract_toml_metadata(content) {
        println!("Extracted TOML metadata: {:?}", toml_metadata); // Debugging output
        Ok(toml_metadata)
    } else if let Some(json_metadata) = extract_json_metadata(content) {
        println!("Extracted JSON metadata: {:?}", json_metadata); // Debugging output
        Ok(json_metadata)
    } else {
        println!("No valid front matter found."); // Debugging output
        Err(MetadataError::ExtractionError(
            "No valid front matter found.".to_string(),
        ))
    }
}

fn extract_yaml_metadata(content: &str) -> Option<Metadata> {
    // More flexible regex for YAML front matter
    let re = Regex::new(r"(?s)^\s*---\s*\n(.*?)\n\s*---\s*").ok()?;
    let captures = re.captures(content)?;

    let yaml_str = captures.get(1)?.as_str().trim();
    println!("Captured YAML content: {:?}", yaml_str); // Debugging output

    let docs = YamlLoader::load_from_str(yaml_str).ok()?;

    if docs.is_empty() {
        println!("Failed to parse YAML content."); // Debugging output
        return None;
    }

    let yaml = docs.into_iter().next()?;

    let metadata: HashMap<String, String> = yaml
        .as_hash()?
        .iter()
        .filter_map(|(k, v)| {
            Some((k.as_str()?.to_string(), v.as_str()?.to_string()))
        })
        .collect();

    println!("Extracted YAML metadata map: {:?}", metadata); // Debugging output

    Some(Metadata::new(metadata))
}

fn extract_toml_metadata(content: &str) -> Option<Metadata> {
    let re = Regex::new(r"(?s)^\s*\+\+\+\s*(.*?)\s*\+\+\+").ok()?;
    let captures = re.captures(content)?;
    let toml_str = captures.get(1)?.as_str().trim(); // Trim to remove unnecessary spaces

    let toml_value: TomlValue = toml::from_str(toml_str).ok()?;
    let toml_table = toml_value.as_table()?;

    let metadata: HashMap<String, String> = toml_table
        .iter()
        .filter_map(|(k, v)| {
            v.as_str().map(|s| (k.clone(), s.to_string()))
        })
        .collect();

    Some(Metadata::new(metadata))
}

fn extract_json_metadata(content: &str) -> Option<Metadata> {
    let re = Regex::new(r"(?s)^\s*\{\s*(.*?)\s*\}").ok()?;
    let captures = re.captures(content)?;
    let json_str = format!("{{{}}}", captures.get(1)?.as_str().trim());

    let json_value: JsonValue = serde_json::from_str(&json_str).ok()?;
    let json_object = json_value.as_object()?;

    let metadata: HashMap<String, String> = json_object
        .iter()
        .filter_map(|(k, v)| {
            v.as_str().map(|s| (k.clone(), s.to_string()))
        })
        .collect();

    Some(Metadata::new(metadata))
}

/// Processes the extracted metadata.
pub fn process_metadata(
    metadata: &Metadata,
) -> Result<Metadata, MetadataError> {
    let mut processed = metadata.clone();

    // Convert dates to a standard format
    if let Some(date) = processed.get("date").cloned() {
        let standardized_date = standardize_date(&date)?;
        processed.insert("date".to_string(), standardized_date);
    }

    // Ensure required fields are present
    ensure_required_fields(&processed)?;

    // Generate derived fields
    generate_derived_fields(&mut processed);

    Ok(processed)
}

fn standardize_date(date: &str) -> Result<String, MetadataError> {
    println!("🦀 Standardizing Date: {} 🦀", date);

    // Handle edge cases with empty or too-short dates
    if date.trim().is_empty() {
        return Err(MetadataError::DateParseError(
            "Date string is empty.".to_string(),
        ));
    }

    if date.len() < 8 {
        return Err(MetadataError::DateParseError(
            "Date string is too short.".to_string(),
        ));
    }

    // Check if the date is in the DD/MM/YYYY format and reformat to YYYY-MM-DD
    let date = if date.contains('/') && date.len() == 10 {
        let parts: Vec<&str> = date.split('/').collect();
        if parts.len() == 3 {
            if parts[0].len() == 2
                && parts[1].len() == 2
                && parts[2].len() == 4
            {
                format!("{}-{}-{}", parts[2], parts[1], parts[0]) // Reformat to YYYY-MM-DD
            } else {
                return Err(MetadataError::DateParseError(
                    "Invalid DD/MM/YYYY date format.".to_string(),
                ));
            }
        } else {
            return Err(MetadataError::DateParseError(
                "Date string could not be split into three parts."
                    .to_string(),
            ));
        }
    } else {
        date.to_string()
    };

    // Attempt to parse the date in different formats using DateTime methods
    let parsed_date = DateTime::parse(&date)
        .or_else(|_| {
            println!("Failed with default parse, trying custom format YYYY-MM-DD.");
            DateTime::parse_custom_format(&date, "[year]-[month]-[day]")
        })
        .or_else(|_| {
            println!("Failed with YYYY-MM-DD, trying MM/DD/YYYY.");
            DateTime::parse_custom_format(&date, "[month]/[day]/[year]") // Handle MM/DD/YYYY
        })
        .map_err(|e| MetadataError::DateParseError(format!("Failed to parse date: {}", e)))?;

    println!("Parsed date: ✅ {:?}", parsed_date);

    // Convert Month enum to numeric value
    let month_number = match parsed_date.month() {
        time::Month::January => 1,
        time::Month::February => 2,
        time::Month::March => 3,
        time::Month::April => 4,
        time::Month::May => 5,
        time::Month::June => 6,
        time::Month::July => 7,
        time::Month::August => 8,
        time::Month::September => 9,
        time::Month::October => 10,
        time::Month::November => 11,
        time::Month::December => 12,
    };

    // Format the date to the standardized YYYY-MM-DD format
    let formatted_date = format!(
        "{:04}-{:02}-{:02}",
        parsed_date.year(),
        month_number,
        parsed_date.day()
    );

    println!("Formatted date: ✅ {}", formatted_date);

    Ok(formatted_date)
}

fn ensure_required_fields(
    metadata: &Metadata,
) -> Result<(), MetadataError> {
    let required_fields = vec!["title", "date"];

    for field in required_fields {
        if !metadata.contains_key(field) {
            return Err(MetadataError::MissingFieldError(
                field.to_string(),
            ));
        }
    }

    Ok(())
}

fn generate_derived_fields(metadata: &mut Metadata) {
    // Generate a URL slug from the title if not present
    if !metadata.contains_key("slug") {
        if let Some(title) = metadata.get("title") {
            let slug = generate_slug(title);
            metadata.insert("slug".to_string(), slug);
        }
    }
}

fn generate_slug(title: &str) -> String {
    title.to_lowercase().replace(' ', "-")
}

#[cfg(test)]
mod tests {
    use super::*;
    use dtt::dtt_parse;

    #[test]
    fn test_standardize_date() {
        let test_cases = vec![
            ("2023-05-20T15:30:00Z", "2023-05-20"),
            ("2023-05-20", "2023-05-20"),
            ("20/05/2023", "2023-05-20"), // European format DD/MM/YYYY
        ];

        for (input, expected) in test_cases {
            let result = standardize_date(input);
            assert!(result.is_ok(), "Failed for input: {}", input);
            assert_eq!(result.unwrap(), expected);
        }
    }

    #[test]
    #[should_panic(expected = "Invalid date format")]
    fn test_standardize_date_fail() {
        let date_string = "2023-02-29"; // Invalid date for non-leap year
        let _ = standardize_date(date_string).unwrap(); // Should panic
    }

    #[test]
    fn test_date_format() {
        let dt = dtt_parse!("2023-01-01T12:00:00+00:00").unwrap();

        // Manually extract year, month, and day from dt
        let year = dt.year(); // Assuming dt has a year() method
        let month = dt.month() as u32; // Assuming dt has a month() method and it returns a numeric value
        let day = dt.day(); // Assuming dt has a day() method

        // Format the date using format! macro
        let formatted = format!("{:04}-{:02}-{:02}", year, month, day);

        // Verify that the formatted date matches the expected value
        assert_eq!(formatted, "2023-01-01");
    }

    #[test]
    fn test_generate_slug() {
        assert_eq!(generate_slug("Hello World"), "hello-world");
        assert_eq!(generate_slug("Test 123"), "test-123");
        assert_eq!(generate_slug("  Spaces  "), "--spaces--");
    }

    #[test]
    fn test_process_metadata() {
        let mut metadata = Metadata::new(HashMap::new());
        metadata.insert("title".to_string(), "Test Title".to_string());
        metadata.insert(
            "date".to_string(),
            "2023-05-20T15:30:00Z".to_string(),
        );

        let processed = process_metadata(&metadata).unwrap();
        assert_eq!(processed.get("title").unwrap(), "Test Title");
        assert_eq!(processed.get("date").unwrap(), "2023-05-20");
        assert_eq!(processed.get("slug").unwrap(), "test-title");
    }

    #[test]
    fn test_extract_metadata() {
        let yaml_content = r#"---
title: YAML Test
date: 2023-05-20
---
Content here"#;

        let toml_content = r#"+++
title = "TOML Test"
date = "2023-05-20"
+++
Content here"#;

        let json_content = r#"{
"title": "JSON Test",
"date": "2023-05-20"
}
Content here"#;

        let yaml_metadata = extract_metadata(yaml_content).unwrap();
        assert_eq!(yaml_metadata.get("title").unwrap(), "YAML Test");

        let toml_metadata = extract_metadata(toml_content).unwrap();
        assert_eq!(toml_metadata.get("title").unwrap(), "TOML Test");

        let json_metadata = extract_metadata(json_content).unwrap();
        assert_eq!(json_metadata.get("title").unwrap(), "JSON Test");
    }
}

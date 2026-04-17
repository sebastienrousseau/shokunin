// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! # Configuration Schema Generator
//!
//! This module generates a JSON Schema for [`crate::cmd::SsgConfig`], enabling
//! editor auto-completion, validation, and documentation of the
//! configuration format.

use serde_json::{json, Value};
use std::fs;
use std::io;
use std::path::Path;

/// Generates a JSON Schema describing all [`crate::cmd::SsgConfig`] fields.
///
/// The returned schema follows the JSON Schema Draft-07 specification
/// and includes type information, descriptions, and default values for
/// every configuration field.
#[must_use]
pub fn generate_schema() -> Value {
    json!({
        "$schema": "https://json-schema.org/draft-07/schema#",
        "title": "SsgConfig",
        "description": "Configuration for the Static Site Generator (SSG).",
        "type": "object",
        "properties": {
            "site_name": {
                "type": "string",
                "description": "Name of the site.",
                "default": "MySsgSite"
            },
            "content_dir": {
                "type": "string",
                "description": "Directory containing content files.",
                "default": "content"
            },
            "output_dir": {
                "type": "string",
                "description": "Directory for generated output files.",
                "default": "public"
            },
            "template_dir": {
                "type": "string",
                "description": "Directory containing template files.",
                "default": "templates"
            },
            "serve_dir": {
                "type": ["string", "null"],
                "description": "Optional directory for development server files.",
                "default": null
            },
            "base_url": {
                "type": "string",
                "description": "Base URL of the site.",
                "default": "http://127.0.0.1:8000",
                "format": "uri"
            },
            "site_title": {
                "type": "string",
                "description": "Title of the site.",
                "default": "My SSG Site"
            },
            "site_description": {
                "type": "string",
                "description": "Description of the site.",
                "default": "A site built with SSG"
            },
            "language": {
                "type": "string",
                "description": "Language code for the site (e.g. en-GB).",
                "default": "en-GB",
                "pattern": "^[a-z]{2}-[A-Z]{2}$"
            }
        },
        "required": [
            "site_name",
            "content_dir",
            "output_dir",
            "template_dir",
            "base_url",
            "site_title",
            "site_description",
            "language"
        ],
        "additionalProperties": false
    })
}

/// Writes the JSON Schema to `path` as pretty-printed JSON.
///
/// # Errors
///
/// Returns an [`io::Error`] if the file cannot be created or written.
///
/// # Panics
///
/// Cannot panic in practice: `generate_schema()` builds a hand-authored
/// `serde_json::Value` tree containing only strings, booleans, arrays
/// and objects — no `f32`/`f64` NaNs — which `to_string_pretty` cannot
/// fail to serialize. The `expect` exists only to satisfy the type
/// system without forcing callers to handle an unreachable `Err`.
pub fn write_schema(path: &Path) -> io::Result<()> {
    let schema = generate_schema();
    // The hand-authored Schema contains only strings/arrays/objects (no
    // NaN floats), so `to_string_pretty` cannot fail. The `expect` is a
    // type-system formality, not a runtime risk.
    #[allow(clippy::expect_used)]
    let content = serde_json::to_string_pretty(&schema)
        .expect("hand-authored Schema is always serializable");
    fs::write(path, content)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::tempdir;

    #[test]
    fn schema_has_correct_title() {
        let schema = generate_schema();
        assert_eq!(schema["title"], "SsgConfig");
    }

    #[test]
    fn schema_has_all_required_fields() {
        let schema = generate_schema();
        let required = schema["required"]
            .as_array()
            .expect("required should be an array");
        let names: Vec<&str> =
            required.iter().map(|v| v.as_str().unwrap()).collect();
        assert!(names.contains(&"site_name"));
        assert!(names.contains(&"content_dir"));
        assert!(names.contains(&"output_dir"));
        assert!(names.contains(&"template_dir"));
        assert!(names.contains(&"base_url"));
        assert!(names.contains(&"site_title"));
        assert!(names.contains(&"site_description"));
        assert!(names.contains(&"language"));
    }

    #[test]
    fn schema_properties_have_types() {
        let schema = generate_schema();
        let props = schema["properties"]
            .as_object()
            .expect("properties should be an object");
        for (key, value) in props {
            assert!(
                value.get("type").is_some(),
                "property '{key}' is missing a type"
            );
        }
    }

    #[test]
    fn schema_defaults_match_config() {
        let schema = generate_schema();
        let props = &schema["properties"];
        assert_eq!(props["site_name"]["default"], "MySsgSite");
        assert_eq!(props["content_dir"]["default"], "content");
        assert_eq!(props["output_dir"]["default"], "public");
        assert_eq!(props["template_dir"]["default"], "templates");
        assert_eq!(props["base_url"]["default"], "http://127.0.0.1:8000");
        assert_eq!(props["site_title"]["default"], "My SSG Site");
        assert_eq!(
            props["site_description"]["default"],
            "A site built with SSG"
        );
        assert_eq!(props["language"]["default"], "en-GB");
    }

    #[test]
    fn schema_language_has_pattern() {
        let schema = generate_schema();
        let pattern = schema["properties"]["language"]["pattern"]
            .as_str()
            .expect("language should have a pattern");
        assert_eq!(pattern, "^[a-z]{2}-[A-Z]{2}$");
    }

    #[test]
    fn serve_dir_allows_null() {
        let schema = generate_schema();
        let types = schema["properties"]["serve_dir"]["type"]
            .as_array()
            .expect("serve_dir type should be an array");
        let type_strs: Vec<&str> =
            types.iter().map(|v| v.as_str().unwrap()).collect();
        assert!(type_strs.contains(&"null"));
        assert!(type_strs.contains(&"string"));
    }

    #[test]
    fn write_schema_creates_valid_json_file() {
        let dir = tempdir().expect("failed to create temp dir");
        let path = dir.path().join("schema.json");
        write_schema(&path).expect("write_schema failed");

        let content =
            fs::read_to_string(&path).expect("failed to read schema file");
        let parsed: Value =
            serde_json::from_str(&content).expect("output is not valid JSON");
        assert_eq!(parsed["title"], "SsgConfig");
    }

    #[test]
    fn write_schema_fails_on_bad_path() {
        let path = PathBuf::from("/nonexistent/dir/schema.json");
        assert!(write_schema(&path).is_err());
    }
}

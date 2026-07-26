// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Integration tests for `ssg::content` (content schemas + validation).

#![allow(clippy::unwrap_used, clippy::expect_used)]
use std::fs;

use ssg::content::{
    load_schemas, parse_schemas, validate_content_dir, ContentSchema,
};
use tempfile::tempdir;

const POST_SCHEMA: &str = r#"
[[schemas]]
name = "post"

[[schemas.fields]]
name = "title"
type = "string"
required = true
"#;

#[test]
fn parse_schemas_accepts_minimal_post_definition() {
    let schemas: Vec<ContentSchema> = parse_schemas(POST_SCHEMA).unwrap();
    assert_eq!(schemas.len(), 1);
    assert_eq!(schemas[0].name, "post");
}

#[test]
fn parse_schemas_rejects_invalid_toml() {
    assert!(parse_schemas("not toml = =").is_err());
}

#[test]
fn load_schemas_reads_from_disk() {
    let dir = tempdir().unwrap();
    let schema_path = dir.path().join("content.schema.toml");
    fs::write(&schema_path, POST_SCHEMA).unwrap();
    let schemas = load_schemas(&schema_path).unwrap();
    assert_eq!(schemas.len(), 1);
}

#[test]
fn validate_content_dir_passes_when_schema_satisfied() {
    let dir = tempdir().unwrap();
    let schema_path = dir.path().join("content.schema.toml");
    fs::write(&schema_path, POST_SCHEMA).unwrap();
    let content = dir.path().join("content");
    fs::create_dir_all(&content).unwrap();
    fs::write(
        content.join("hello.md"),
        "---\nschema: post\ntitle: Hello\n---\n# Hello",
    )
    .unwrap();
    let schemas = load_schemas(&schema_path).unwrap();
    let res = validate_content_dir(&content, &schemas);
    assert!(res.is_ok());
}

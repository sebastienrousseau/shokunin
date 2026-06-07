// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Integration tests for `ssg::schema`.

use ssg::schema::{generate_schema, write_schema};
use tempfile::tempdir;

#[test]
fn generate_schema_emits_a_json_object() {
    let s = generate_schema();
    assert!(s.is_object(), "JSON Schema should be an object");
}

#[test]
fn write_schema_persists_to_disk() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("ssg.schema.json");
    write_schema(&path).unwrap();
    assert!(path.exists());
    let body = std::fs::read_to_string(&path).unwrap();
    assert!(body.contains("$schema") || !body.is_empty());
}

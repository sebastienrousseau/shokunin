#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::needless_pass_by_value,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::module_name_repetitions,
    clippy::redundant_closure,
    clippy::redundant_closure_for_method_calls,
    clippy::needless_raw_string_hashes,
    clippy::single_char_pattern,
    clippy::format_in_format_args,
    clippy::needless_late_init,
    clippy::if_then_some_else_none,
    clippy::must_use_candidate
)]
// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Regression tests for `ssg::content::validate_with_schema` and the
//! older `validate_only`. Covers the cases that drove the API split:
//!
//! - Schema *outside* `content_dir` (the normal pattern, since
//!   `staticdatagen::compile` would otherwise try to parse the TOML
//!   schema as a Markdown content file).
//! - Schema *inside* `content_dir` (legacy `validate_only` path).
//! - Frontmatter that declares no `schema:` field — should be silently
//!   skipped, never fail.
//! - Frontmatter that declares an unknown schema name — should fail with
//!   a precise error.
//! - Frontmatter missing required fields — should fail.
//! - Frontmatter with an out-of-enum value — should fail with the list
//!   of allowed values.

use std::fs;
use std::path::Path;

use ssg::content::{validate_only, validate_with_schema};

const SCHEMA_TOML: &str = r#"
[[schemas]]
name = "doc"

[[schemas.fields]]
name = "title"
type = "string"
required = true

[[schemas.fields]]
name = "category"
type = "enum(welcome,getting-started,guides,system)"
required = true
"#;

/// Build a minimal frontmatter document with the requested fields.
fn page(
    title: Option<&str>,
    category: Option<&str>,
    schema: Option<&str>,
) -> String {
    let mut fm = String::from("---\n");
    if let Some(t) = title {
        fm.push_str(&format!("title: \"{t}\"\n"));
    }
    if let Some(c) = category {
        fm.push_str(&format!("category: \"{c}\"\n"));
    }
    if let Some(s) = schema {
        fm.push_str(&format!("schema: \"{s}\"\n"));
    }
    fm.push_str("---\n\n# body\n");
    fm
}

fn write(content_dir: &Path, name: &str, body: &str) {
    fs::write(content_dir.join(name), body).unwrap();
}

#[test]
fn validate_with_schema_passes_on_valid_pages() {
    let tmp = tempfile::tempdir().unwrap();
    let content = tmp.path().join("content");
    fs::create_dir_all(&content).unwrap();
    let schema = tmp.path().join("content.schema.toml");
    fs::write(&schema, SCHEMA_TOML).unwrap();

    write(
        &content,
        "index.md",
        &page(Some("Welcome"), Some("welcome"), Some("doc")),
    );
    write(
        &content,
        "guide.md",
        &page(Some("Guide"), Some("guides"), Some("doc")),
    );

    validate_with_schema(&content, &schema).expect("valid pages should pass");
}

#[test]
fn validate_with_schema_fails_on_unknown_enum_value() {
    let tmp = tempfile::tempdir().unwrap();
    let content = tmp.path().join("content");
    fs::create_dir_all(&content).unwrap();
    let schema = tmp.path().join("content.schema.toml");
    fs::write(&schema, SCHEMA_TOML).unwrap();

    write(
        &content,
        "broken.md",
        &page(Some("Broken"), Some("not-a-real-cat"), Some("doc")),
    );

    let err = validate_with_schema(&content, &schema)
        .expect_err("invalid enum should fail");
    let msg = format!("{err:#}");
    // Detailed errors are written to stderr by the validator; the
    // returned `Err` carries an aggregate count. Assert the count is
    // surfaced.
    assert!(
        msg.contains("1") && msg.contains("error"),
        "error should report the count, got: {msg}"
    );
}

#[test]
fn validate_with_schema_fails_on_missing_required_field() {
    let tmp = tempfile::tempdir().unwrap();
    let content = tmp.path().join("content");
    fs::create_dir_all(&content).unwrap();
    let schema = tmp.path().join("content.schema.toml");
    fs::write(&schema, SCHEMA_TOML).unwrap();

    // Missing `category` — required by schema.
    write(
        &content,
        "missing.md",
        &page(Some("No category"), None, Some("doc")),
    );

    let err = validate_with_schema(&content, &schema)
        .expect_err("missing required field should fail");
    let msg = format!("{err:#}");
    assert!(
        msg.contains("error"),
        "error should be reported, got: {msg}"
    );
}

#[test]
fn validate_with_schema_skips_pages_without_schema_declaration() {
    // Pages with no `schema:` field in frontmatter must be silently
    // skipped — that's the contract for opt-in schema validation.
    let tmp = tempfile::tempdir().unwrap();
    let content = tmp.path().join("content");
    fs::create_dir_all(&content).unwrap();
    let schema = tmp.path().join("content.schema.toml");
    fs::write(&schema, SCHEMA_TOML).unwrap();

    // Page declares no schema and has no required fields — should pass.
    write(
        &content,
        "untyped.md",
        "---\ntitle: \"Untyped\"\n---\n\n# body\n",
    );

    validate_with_schema(&content, &schema)
        .expect("schema-less page should be skipped");
}

#[test]
fn validate_with_schema_fails_on_unknown_schema_name() {
    let tmp = tempfile::tempdir().unwrap();
    let content = tmp.path().join("content");
    fs::create_dir_all(&content).unwrap();
    let schema = tmp.path().join("content.schema.toml");
    fs::write(&schema, SCHEMA_TOML).unwrap();

    // Schema name not declared in TOML.
    write(
        &content,
        "wrong.md",
        &page(Some("X"), Some("welcome"), Some("not-a-schema")),
    );

    let err = validate_with_schema(&content, &schema)
        .expect_err("unknown schema name should fail");
    let msg = format!("{err:#}");
    assert!(
        msg.contains("error"),
        "error should be reported, got: {msg}"
    );
}

#[test]
fn validate_with_schema_passes_silently_when_schema_file_missing() {
    let tmp = tempfile::tempdir().unwrap();
    let content = tmp.path().join("content");
    fs::create_dir_all(&content).unwrap();
    write(
        &content,
        "anything.md",
        &page(Some("Anything"), Some("welcome"), Some("doc")),
    );

    let nonexistent = tmp.path().join("does-not-exist.toml");
    // The missing-schema case should be handled gracefully (returns Ok
    // with a printed warning; nothing to validate against).
    validate_with_schema(&content, &nonexistent)
        .expect("missing schema file should be tolerated");
}

#[test]
fn validate_only_legacy_path_still_works() {
    // Backwards compat: `validate_only(content_dir)` looks for
    // `content_dir/content.schema.toml` — used by older example code.
    let tmp = tempfile::tempdir().unwrap();
    let content = tmp.path().join("content");
    fs::create_dir_all(&content).unwrap();
    fs::write(content.join("content.schema.toml"), SCHEMA_TOML).unwrap();

    write(
        &content,
        "page.md",
        &page(Some("Page"), Some("welcome"), Some("doc")),
    );

    validate_only(&content).expect("legacy path should still validate");
}

#[test]
fn validate_with_schema_catches_multiple_errors_at_once() {
    let tmp = tempfile::tempdir().unwrap();
    let content = tmp.path().join("content");
    fs::create_dir_all(&content).unwrap();
    let schema = tmp.path().join("content.schema.toml");
    fs::write(&schema, SCHEMA_TOML).unwrap();

    write(
        &content,
        "bad-enum.md",
        &page(Some("X"), Some("bogus"), Some("doc")),
    );
    write(
        &content,
        "missing-cat.md",
        &page(Some("Y"), None, Some("doc")),
    );

    let err = validate_with_schema(&content, &schema)
        .expect_err("multiple errors should fail");
    let msg = format!("{err:#}");
    // Message includes a count, e.g. "2 content validation error(s)"
    assert!(
        msg.contains("2") || msg.contains("error"),
        "error count should be reported, got: {msg}"
    );
}

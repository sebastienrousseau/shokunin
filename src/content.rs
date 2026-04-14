// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! # Typed content collections with frontmatter schema validation
//!
//! Defines content schemas in `content.schema.toml` and validates
//! every Markdown file's frontmatter against the matching schema
//! before compilation begins.
//!
//! ## Schema file format
//!
//! ```toml
//! [[schemas]]
//! name = "post"
//!
//! [[schemas.fields]]
//! name = "title"
//! type = "string"
//! required = true
//!
//! [[schemas.fields]]
//! name = "date"
//! type = "date"
//! required = true
//!
//! [[schemas.fields]]
//! name = "draft"
//! type = "bool"
//! required = false
//! default = "false"
//! ```
//!
//! ## Supported field types
//!
//! `string`, `date`, `bool`, `integer`, `float`, `list`,
//! `enum(value1,value2,...)`.

use anyhow::{Context, Result};
use log::info;
use serde::Deserialize;
use std::{
    collections::HashMap,
    fmt, fs,
    path::{Path, PathBuf},
};

use crate::plugin::{Plugin, PluginContext};

// -----------------------------------------------------------------------
// Schema types
// -----------------------------------------------------------------------

/// The type of a frontmatter field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FieldType {
    /// Free-form string.
    String,
    /// ISO-8601 date (`YYYY-MM-DD`).
    Date,
    /// Boolean (`true` / `false`).
    Bool,
    /// Signed integer.
    Integer,
    /// Floating-point number.
    Float,
    /// YAML/TOML array.
    List,
    /// One of a fixed set of allowed values.
    Enum(Vec<String>),
}

impl fmt::Display for FieldType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::String => write!(f, "string"),
            Self::Date => write!(f, "date"),
            Self::Bool => write!(f, "bool"),
            Self::Integer => write!(f, "integer"),
            Self::Float => write!(f, "float"),
            Self::List => write!(f, "list"),
            Self::Enum(variants) => write!(f, "enum({})", variants.join(",")),
        }
    }
}

/// Parses a type string (from the TOML schema) into a [`FieldType`].
///
/// Accepts `"string"`, `"date"`, `"bool"`, `"integer"`, `"float"`,
/// `"list"`, and `"enum(a,b,c)"`.
fn parse_field_type(s: &str) -> Result<FieldType, String> {
    match s.trim() {
        "string" => Ok(FieldType::String),
        "date" => Ok(FieldType::Date),
        "bool" => Ok(FieldType::Bool),
        "integer" => Ok(FieldType::Integer),
        "float" => Ok(FieldType::Float),
        "list" => Ok(FieldType::List),
        other if other.starts_with("enum(") && other.ends_with(')') => {
            let inner = &other[5..other.len() - 1];
            let variants: Vec<String> =
                inner.split(',').map(|v| v.trim().to_owned()).collect();
            if variants.is_empty() || variants.iter().any(String::is_empty) {
                return Err(format!(
                    "enum type must have non-empty variants: {other}"
                ));
            }
            Ok(FieldType::Enum(variants))
        }
        _ => Err(format!("unknown field type: {s}")),
    }
}

/// Definition of a single frontmatter field.
#[derive(Debug, Clone)]
pub struct FieldDef {
    /// Field name as it appears in frontmatter.
    pub name: String,
    /// Expected type of the field value.
    pub field_type: FieldType,
    /// Whether the field must be present.
    pub required: bool,
    /// Optional default value (serialized as a string).
    pub default: Option<String>,
}

/// A named collection of field definitions.
#[derive(Debug, Clone)]
pub struct ContentSchema {
    /// Schema name (e.g. `"post"`). Content files opt in via a
    /// `schema = "post"` frontmatter key.
    pub name: String,
    /// Expected fields.
    pub fields: Vec<FieldDef>,
}

// -----------------------------------------------------------------------
// TOML deserialization helpers (intermediate representation)
// -----------------------------------------------------------------------

#[derive(Deserialize)]
struct SchemaFile {
    schemas: Vec<RawSchema>,
}

#[derive(Deserialize)]
struct RawSchema {
    name: String,
    fields: Vec<RawField>,
}

#[derive(Deserialize)]
struct RawField {
    name: String,
    #[serde(rename = "type")]
    field_type: String,
    #[serde(default)]
    required: bool,
    default: Option<String>,
}

// -----------------------------------------------------------------------
// Loading schemas
// -----------------------------------------------------------------------

/// Loads all content schemas from a `content.schema.toml` file.
///
/// Returns an empty `Vec` if the file does not exist — schemas are
/// opt-in.
pub fn load_schemas(path: &Path) -> Result<Vec<ContentSchema>> {
    if !path.exists() {
        return Ok(Vec::new());
    }

    let text = fs::read_to_string(path).with_context(|| {
        format!("failed to read schema file: {}", path.display())
    })?;

    parse_schemas(&text)
}

/// Parses the TOML text of a schema file into [`ContentSchema`] values.
pub fn parse_schemas(toml_text: &str) -> Result<Vec<ContentSchema>> {
    let raw: SchemaFile = toml::from_str(toml_text)
        .context("failed to parse content.schema.toml")?;

    raw.schemas
        .into_iter()
        .map(|rs| {
            let fields = rs
                .fields
                .into_iter()
                .map(|rf| {
                    let ft = parse_field_type(&rf.field_type).map_err(|e| {
                        anyhow::anyhow!(
                            "schema '{}', field '{}': {}",
                            rs.name,
                            rf.name,
                            e
                        )
                    })?;
                    Ok(FieldDef {
                        name: rf.name,
                        field_type: ft,
                        required: rf.required,
                        default: rf.default,
                    })
                })
                .collect::<Result<Vec<_>>>()?;
            Ok(ContentSchema {
                name: rs.name,
                fields,
            })
        })
        .collect()
}

// -----------------------------------------------------------------------
// Validation
// -----------------------------------------------------------------------

/// A single validation error with file context.
#[derive(Debug, Clone)]
pub struct ValidationError {
    /// Path to the content file.
    pub file: PathBuf,
    /// One-based line number of the frontmatter block (or 1 if unknown).
    pub line: usize,
    /// Human-readable error message.
    pub message: String,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}: {}", self.file.display(), self.line, self.message)
    }
}

/// Validates a frontmatter map against a [`ContentSchema`].
///
/// Returns a list of errors (empty on success). `file_path` and
/// `fm_start_line` are used only for error reporting.
#[must_use]
pub fn validate_frontmatter(
    fields: &HashMap<String, String>,
    schema: &ContentSchema,
    file_path: &Path,
    fm_start_line: usize,
) -> Vec<ValidationError> {
    let mut errors = Vec::new();

    for field_def in &schema.fields {
        match fields.get(&field_def.name) {
            Some(value) => {
                if let Err(msg) = validate_value(value, &field_def.field_type) {
                    errors.push(ValidationError {
                        file: file_path.to_path_buf(),
                        line: fm_start_line,
                        message: format!(
                            "field '{}': {msg} (expected {})",
                            field_def.name, field_def.field_type
                        ),
                    });
                }
            }
            None => {
                if field_def.required && field_def.default.is_none() {
                    errors.push(ValidationError {
                        file: file_path.to_path_buf(),
                        line: fm_start_line,
                        message: format!(
                            "required field '{}' is missing",
                            field_def.name
                        ),
                    });
                }
            }
        }
    }

    errors
}

/// Returns `true` if `value` matches the YYYY-MM-DD date format.
fn is_valid_date(value: &str) -> bool {
    let parts: Vec<&str> = value.split('-').collect();
    parts.len() == 3
        && parts[0].len() == 4
        && parts[1].len() == 2
        && parts[2].len() == 2
        && parts[0].chars().all(|c| c.is_ascii_digit())
        && parts[1].chars().all(|c| c.is_ascii_digit())
        && parts[2].chars().all(|c| c.is_ascii_digit())
}

/// Checks that `value` is valid for the given [`FieldType`].
fn validate_value(value: &str, ft: &FieldType) -> Result<(), String> {
    match ft {
        FieldType::String => Ok(()),
        FieldType::Date => {
            // Accept YYYY-MM-DD
            if is_valid_date(value) {
                Ok(())
            } else {
                Err(format!(
                    "'{value}' is not a valid date (expected YYYY-MM-DD)"
                ))
            }
        }
        FieldType::Bool => match value {
            "true" | "false" => Ok(()),
            _ => Err(format!("'{value}' is not a valid bool")),
        },
        FieldType::Integer => {
            let _: i64 = value
                .parse::<i64>()
                .map_err(|_| format!("'{value}' is not a valid integer"))?;
            Ok(())
        }
        FieldType::Float => {
            let _: f64 = value
                .parse::<f64>()
                .map_err(|_| format!("'{value}' is not a valid float"))?;
            Ok(())
        }
        FieldType::List => {
            // Accept comma-separated or YAML-style bracket syntax
            // Anything non-empty is accepted; empty is fine too.
            Ok(())
        }
        FieldType::Enum(variants) => {
            if variants.iter().any(|v| v == value) {
                Ok(())
            } else {
                Err(format!(
                    "'{value}' is not one of the allowed values: {}",
                    variants.join(", ")
                ))
            }
        }
    }
}

// -----------------------------------------------------------------------
// Helpers: extract frontmatter as flat HashMap<String, String>
// -----------------------------------------------------------------------

/// Extracts frontmatter from Markdown content as a flat string map.
///
/// Returns `(fields, frontmatter_start_line)`. The start line is 1 for
/// files beginning with `---`.
fn extract_frontmatter_map(
    content: &str,
) -> Option<(HashMap<String, String>, usize)> {
    let fm_result = frontmatter_gen::extract(content);
    let Ok((fm, _body)) = fm_result else {
        return None;
    };

    let mut map = HashMap::new();
    for (key, value) in &fm.0 {
        let _ = map.insert(key.clone(), fm_value_to_string(value));
    }

    // Frontmatter starts at line 1 for `---` delimited blocks.
    Some((map, 1))
}

/// Converts a frontmatter value to its string representation.
fn fm_value_to_string(value: &frontmatter_gen::Value) -> String {
    match value {
        frontmatter_gen::Value::String(s) => s.clone(),
        frontmatter_gen::Value::Number(n) => format!("{n}"),
        frontmatter_gen::Value::Boolean(b) => format!("{b}"),
        frontmatter_gen::Value::Array(arr) => arr
            .iter()
            .map(fm_value_to_string)
            .collect::<Vec<_>>()
            .join(","),
        frontmatter_gen::Value::Null => String::new(),
        other => format!("{other:?}"),
    }
}

// -----------------------------------------------------------------------
// Validate all content files against schemas
// -----------------------------------------------------------------------

/// Validates every `.md` file in `content_dir` against the loaded schemas.
///
/// Files opt in by including a `schema = "<name>"` field in their
/// frontmatter. Files without the field are silently skipped.
///
/// Returns all validation errors found across all files.
pub fn validate_content_dir(
    content_dir: &Path,
    schemas: &[ContentSchema],
) -> Result<Vec<ValidationError>> {
    if schemas.is_empty() {
        return Ok(Vec::new());
    }

    let schema_map: HashMap<&str, &ContentSchema> =
        schemas.iter().map(|s| (s.name.as_str(), s)).collect();

    let md_files = crate::walk::walk_files_bounded_depth(
        content_dir,
        "md",
        crate::MAX_DIR_DEPTH,
    )?;

    let mut all_errors = Vec::new();

    for md_path in &md_files {
        let content = fs::read_to_string(md_path)
            .with_context(|| format!("failed to read {}", md_path.display()))?;

        let Some((fields, fm_line)) = extract_frontmatter_map(&content) else {
            continue;
        };

        // Determine which schema applies
        let schema_name = match fields.get("schema") {
            Some(name) => name.as_str(),
            None => continue, // no schema declared — skip
        };

        let Some(schema) = schema_map.get(schema_name) else {
            all_errors.push(ValidationError {
                file: md_path.clone(),
                line: fm_line,
                message: format!("unknown schema '{schema_name}'"),
            });
            continue;
        };

        let mut errs = validate_frontmatter(&fields, schema, md_path, fm_line);
        all_errors.append(&mut errs);
    }

    Ok(all_errors)
}

// -----------------------------------------------------------------------
// Plugin
// -----------------------------------------------------------------------

/// Plugin that validates content frontmatter against schemas defined in
/// `content.schema.toml` during the `before_compile` hook.
#[derive(Debug, Clone, Copy)]
pub struct ContentValidationPlugin;

impl Plugin for ContentValidationPlugin {
    fn name(&self) -> &'static str {
        "content-validation"
    }

    fn before_compile(&self, ctx: &PluginContext) -> Result<()> {
        let schema_path = ctx.content_dir.join("content.schema.toml");
        let schemas = load_schemas(&schema_path)?;

        if schemas.is_empty() {
            info!("No content schemas found — skipping validation");
            return Ok(());
        }

        info!(
            "Loaded {} content schema(s), validating {}",
            schemas.len(),
            ctx.content_dir.display()
        );

        let errors = validate_content_dir(&ctx.content_dir, &schemas)?;

        if errors.is_empty() {
            info!("All content files passed schema validation");
            Ok(())
        } else {
            let mut msg = format!(
                "Content validation failed with {} error(s):\n",
                errors.len()
            );
            for err in &errors {
                msg.push_str(&format!("  {err}\n"));
            }
            Err(anyhow::anyhow!("{msg}"))
        }
    }
}

// -----------------------------------------------------------------------
// Standalone validate-only entry point (for --validate flag)
// -----------------------------------------------------------------------

/// Runs content schema validation without building the site.
///
/// Returns `Ok(())` when all files pass, or an error listing every
/// validation failure.
pub fn validate_only(content_dir: &Path) -> Result<()> {
    let schema_path = content_dir.join("content.schema.toml");
    let schemas = load_schemas(&schema_path)?;

    if schemas.is_empty() {
        println!("No content schemas found in {}", schema_path.display());
        return Ok(());
    }

    println!("Loaded {} schema(s)", schemas.len());

    let errors = validate_content_dir(content_dir, &schemas)?;

    if errors.is_empty() {
        println!("All content files passed schema validation.");
        Ok(())
    } else {
        eprintln!("Validation failed with {} error(s):", errors.len());
        for err in &errors {
            eprintln!("  {err}");
        }
        Err(anyhow::anyhow!(
            "{} content validation error(s)",
            errors.len()
        ))
    }
}

// -----------------------------------------------------------------------
// Tests
// -----------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    // -------------------------------------------------------------------
    // FieldType parsing
    // -------------------------------------------------------------------

    #[test]
    fn parse_field_type_string() {
        assert_eq!(parse_field_type("string").unwrap(), FieldType::String);
    }

    #[test]
    fn parse_field_type_date() {
        assert_eq!(parse_field_type("date").unwrap(), FieldType::Date);
    }

    #[test]
    fn parse_field_type_bool() {
        assert_eq!(parse_field_type("bool").unwrap(), FieldType::Bool);
    }

    #[test]
    fn parse_field_type_integer() {
        assert_eq!(parse_field_type("integer").unwrap(), FieldType::Integer);
    }

    #[test]
    fn parse_field_type_float() {
        assert_eq!(parse_field_type("float").unwrap(), FieldType::Float);
    }

    #[test]
    fn parse_field_type_list() {
        assert_eq!(parse_field_type("list").unwrap(), FieldType::List);
    }

    #[test]
    fn parse_field_type_enum() {
        let ft = parse_field_type("enum(draft,published,archived)").unwrap();
        assert_eq!(
            ft,
            FieldType::Enum(vec![
                "draft".to_owned(),
                "published".to_owned(),
                "archived".to_owned(),
            ])
        );
    }

    #[test]
    fn parse_field_type_enum_trimmed() {
        let ft = parse_field_type("enum( a , b )").unwrap();
        assert_eq!(ft, FieldType::Enum(vec!["a".to_owned(), "b".to_owned()]));
    }

    #[test]
    fn parse_field_type_unknown() {
        assert!(parse_field_type("foobar").is_err());
    }

    #[test]
    fn parse_field_type_enum_empty_variants() {
        assert!(parse_field_type("enum()").is_err());
    }

    // -------------------------------------------------------------------
    // FieldType Display
    // -------------------------------------------------------------------

    #[test]
    fn field_type_display() {
        assert_eq!(FieldType::String.to_string(), "string");
        assert_eq!(FieldType::Date.to_string(), "date");
        assert_eq!(FieldType::Bool.to_string(), "bool");
        assert_eq!(FieldType::Integer.to_string(), "integer");
        assert_eq!(FieldType::Float.to_string(), "float");
        assert_eq!(FieldType::List.to_string(), "list");
        assert_eq!(
            FieldType::Enum(vec!["a".into(), "b".into()]).to_string(),
            "enum(a,b)"
        );
    }

    // -------------------------------------------------------------------
    // validate_value
    // -------------------------------------------------------------------

    #[test]
    fn validate_string_always_ok() {
        assert!(validate_value("anything", &FieldType::String).is_ok());
        assert!(validate_value("", &FieldType::String).is_ok());
    }

    #[test]
    fn validate_date_ok() {
        assert!(validate_value("2024-01-15", &FieldType::Date).is_ok());
    }

    #[test]
    fn validate_date_bad() {
        assert!(validate_value("not-a-date", &FieldType::Date).is_err());
        assert!(validate_value("2024/01/15", &FieldType::Date).is_err());
    }

    #[test]
    fn validate_bool_ok() {
        assert!(validate_value("true", &FieldType::Bool).is_ok());
        assert!(validate_value("false", &FieldType::Bool).is_ok());
    }

    #[test]
    fn validate_bool_bad() {
        assert!(validate_value("yes", &FieldType::Bool).is_err());
        assert!(validate_value("1", &FieldType::Bool).is_err());
    }

    #[test]
    fn validate_integer_ok() {
        assert!(validate_value("42", &FieldType::Integer).is_ok());
        assert!(validate_value("-7", &FieldType::Integer).is_ok());
        assert!(validate_value("0", &FieldType::Integer).is_ok());
    }

    #[test]
    fn validate_integer_bad() {
        assert!(validate_value("3.14", &FieldType::Integer).is_err());
        assert!(validate_value("abc", &FieldType::Integer).is_err());
    }

    #[test]
    fn validate_float_ok() {
        assert!(validate_value("3.14", &FieldType::Float).is_ok());
        assert!(validate_value("-1.0", &FieldType::Float).is_ok());
        assert!(validate_value("42", &FieldType::Float).is_ok());
    }

    #[test]
    fn validate_float_bad() {
        assert!(validate_value("abc", &FieldType::Float).is_err());
    }

    #[test]
    fn validate_list_always_ok() {
        assert!(validate_value("a,b,c", &FieldType::List).is_ok());
        assert!(validate_value("", &FieldType::List).is_ok());
    }

    #[test]
    fn validate_enum_ok() {
        let ft = FieldType::Enum(vec!["draft".into(), "published".into()]);
        assert!(validate_value("draft", &ft).is_ok());
        assert!(validate_value("published", &ft).is_ok());
    }

    #[test]
    fn validate_enum_bad() {
        let ft = FieldType::Enum(vec!["draft".into(), "published".into()]);
        assert!(validate_value("archived", &ft).is_err());
    }

    // -------------------------------------------------------------------
    // validate_frontmatter
    // -------------------------------------------------------------------

    #[test]
    fn validate_frontmatter_all_present() {
        let schema = ContentSchema {
            name: "post".into(),
            fields: vec![
                FieldDef {
                    name: "title".into(),
                    field_type: FieldType::String,
                    required: true,
                    default: None,
                },
                FieldDef {
                    name: "date".into(),
                    field_type: FieldType::Date,
                    required: true,
                    default: None,
                },
            ],
        };

        let mut fields = HashMap::new();
        let _ = fields.insert("title".into(), "Hello".into());
        let _ = fields.insert("date".into(), "2024-06-01".into());

        let errors =
            validate_frontmatter(&fields, &schema, Path::new("test.md"), 1);
        assert!(errors.is_empty());
    }

    #[test]
    fn validate_frontmatter_missing_required() {
        let schema = ContentSchema {
            name: "post".into(),
            fields: vec![FieldDef {
                name: "title".into(),
                field_type: FieldType::String,
                required: true,
                default: None,
            }],
        };

        let fields: HashMap<String, String> = HashMap::new();
        let errors =
            validate_frontmatter(&fields, &schema, Path::new("test.md"), 1);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("required"));
    }

    #[test]
    fn validate_frontmatter_missing_with_default() {
        let schema = ContentSchema {
            name: "post".into(),
            fields: vec![FieldDef {
                name: "draft".into(),
                field_type: FieldType::Bool,
                required: true,
                default: Some("false".into()),
            }],
        };

        let fields: HashMap<String, String> = HashMap::new();
        let errors =
            validate_frontmatter(&fields, &schema, Path::new("test.md"), 1);
        // Has a default — no error even though required
        assert!(errors.is_empty());
    }

    #[test]
    fn validate_frontmatter_wrong_type() {
        let schema = ContentSchema {
            name: "post".into(),
            fields: vec![FieldDef {
                name: "date".into(),
                field_type: FieldType::Date,
                required: true,
                default: None,
            }],
        };

        let mut fields = HashMap::new();
        let _ = fields.insert("date".into(), "not-a-date".into());

        let errors =
            validate_frontmatter(&fields, &schema, Path::new("test.md"), 1);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("date"));
    }

    #[test]
    fn validate_frontmatter_optional_missing_ok() {
        let schema = ContentSchema {
            name: "post".into(),
            fields: vec![FieldDef {
                name: "subtitle".into(),
                field_type: FieldType::String,
                required: false,
                default: None,
            }],
        };

        let fields: HashMap<String, String> = HashMap::new();
        let errors =
            validate_frontmatter(&fields, &schema, Path::new("test.md"), 1);
        assert!(errors.is_empty());
    }

    // -------------------------------------------------------------------
    // ValidationError Display
    // -------------------------------------------------------------------

    #[test]
    fn validation_error_display() {
        let err = ValidationError {
            file: PathBuf::from("content/post.md"),
            line: 3,
            message: "field 'title': missing".into(),
        };
        assert_eq!(
            err.to_string(),
            "content/post.md:3: field 'title': missing"
        );
    }

    // -------------------------------------------------------------------
    // Schema loading (parse_schemas)
    // -------------------------------------------------------------------

    #[test]
    fn parse_schemas_basic() {
        let toml = r#"
[[schemas]]
name = "post"

[[schemas.fields]]
name = "title"
type = "string"
required = true

[[schemas.fields]]
name = "date"
type = "date"
required = true

[[schemas.fields]]
name = "draft"
type = "bool"
required = false
default = "false"
"#;
        let schemas = parse_schemas(toml).unwrap();
        assert_eq!(schemas.len(), 1);
        assert_eq!(schemas[0].name, "post");
        assert_eq!(schemas[0].fields.len(), 3);
        assert_eq!(schemas[0].fields[0].name, "title");
        assert!(schemas[0].fields[0].required);
        assert_eq!(schemas[0].fields[2].default, Some("false".to_owned()));
    }

    #[test]
    fn parse_schemas_multiple() {
        let toml = r#"
[[schemas]]
name = "post"

[[schemas.fields]]
name = "title"
type = "string"
required = true

[[schemas]]
name = "page"

[[schemas.fields]]
name = "heading"
type = "string"
required = true
"#;
        let schemas = parse_schemas(toml).unwrap();
        assert_eq!(schemas.len(), 2);
        assert_eq!(schemas[0].name, "post");
        assert_eq!(schemas[1].name, "page");
    }

    #[test]
    fn parse_schemas_enum_field() {
        let toml = r#"
[[schemas]]
name = "post"

[[schemas.fields]]
name = "status"
type = "enum(draft,published,archived)"
required = true
"#;
        let schemas = parse_schemas(toml).unwrap();
        assert_eq!(
            schemas[0].fields[0].field_type,
            FieldType::Enum(vec![
                "draft".into(),
                "published".into(),
                "archived".into()
            ])
        );
    }

    #[test]
    fn parse_schemas_bad_type() {
        let toml = r#"
[[schemas]]
name = "post"

[[schemas.fields]]
name = "x"
type = "unknown_type"
required = true
"#;
        assert!(parse_schemas(toml).is_err());
    }

    #[test]
    fn parse_schemas_bad_toml() {
        assert!(parse_schemas("not valid toml {{{}}}").is_err());
    }

    // -------------------------------------------------------------------
    // load_schemas from file
    // -------------------------------------------------------------------

    #[test]
    fn load_schemas_nonexistent_file() {
        let schemas =
            load_schemas(Path::new("/tmp/does-not-exist/content.schema.toml"))
                .unwrap();
        assert!(schemas.is_empty());
    }

    #[test]
    fn load_schemas_from_file() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("content.schema.toml");
        fs::write(
            &path,
            r#"
[[schemas]]
name = "post"

[[schemas.fields]]
name = "title"
type = "string"
required = true
"#,
        )
        .unwrap();

        let schemas = load_schemas(&path).unwrap();
        assert_eq!(schemas.len(), 1);
    }

    // -------------------------------------------------------------------
    // extract_frontmatter_map
    // -------------------------------------------------------------------

    #[test]
    fn extract_fm_from_yaml() {
        let content = "---\ntitle: Hello\ndate: 2024-01-01\n---\n\nBody text";
        let (map, line) = extract_frontmatter_map(content).unwrap();
        assert_eq!(map.get("title").unwrap(), "Hello");
        assert_eq!(map.get("date").unwrap(), "2024-01-01");
        assert_eq!(line, 1);
    }

    #[test]
    fn extract_fm_no_frontmatter() {
        let content = "Just plain text without frontmatter.";
        assert!(extract_frontmatter_map(content).is_none());
    }

    // -------------------------------------------------------------------
    // fm_value_to_string
    // -------------------------------------------------------------------

    #[test]
    fn fm_value_string() {
        let v = frontmatter_gen::Value::String("hello".into());
        assert_eq!(fm_value_to_string(&v), "hello");
    }

    #[test]
    fn fm_value_number() {
        let v = frontmatter_gen::Value::Number(42.0);
        assert_eq!(fm_value_to_string(&v), "42");
    }

    #[test]
    fn fm_value_bool() {
        let v = frontmatter_gen::Value::Boolean(true);
        assert_eq!(fm_value_to_string(&v), "true");
    }

    #[test]
    fn fm_value_null() {
        let v = frontmatter_gen::Value::Null;
        assert_eq!(fm_value_to_string(&v), "");
    }

    #[test]
    fn fm_value_array() {
        let v = frontmatter_gen::Value::Array(vec![
            frontmatter_gen::Value::String("a".into()),
            frontmatter_gen::Value::String("b".into()),
        ]);
        assert_eq!(fm_value_to_string(&v), "a,b");
    }

    // -------------------------------------------------------------------
    // validate_content_dir integration
    // -------------------------------------------------------------------

    #[test]
    fn validate_content_dir_empty_schemas() {
        let dir = tempdir().unwrap();
        let errors = validate_content_dir(dir.path(), &[]).unwrap();
        assert!(errors.is_empty());
    }

    #[test]
    fn validate_content_dir_no_md_files() {
        let dir = tempdir().unwrap();
        let schema = ContentSchema {
            name: "post".into(),
            fields: vec![FieldDef {
                name: "title".into(),
                field_type: FieldType::String,
                required: true,
                default: None,
            }],
        };
        let errors = validate_content_dir(dir.path(), &[schema]).unwrap();
        assert!(errors.is_empty());
    }

    #[test]
    fn validate_content_dir_valid_file() {
        let dir = tempdir().unwrap();
        let md =
            "---\ntitle: Hello\nschema: post\ndate: 2024-06-01\n---\n\nBody";
        fs::write(dir.path().join("hello.md"), md).unwrap();

        let schema = ContentSchema {
            name: "post".into(),
            fields: vec![
                FieldDef {
                    name: "title".into(),
                    field_type: FieldType::String,
                    required: true,
                    default: None,
                },
                FieldDef {
                    name: "date".into(),
                    field_type: FieldType::Date,
                    required: true,
                    default: None,
                },
            ],
        };
        let errors = validate_content_dir(dir.path(), &[schema]).unwrap();
        assert!(errors.is_empty(), "unexpected errors: {errors:?}");
    }

    #[test]
    fn validate_content_dir_invalid_file() {
        let dir = tempdir().unwrap();
        let md = "---\nschema: post\n---\n\nBody without title";
        fs::write(dir.path().join("bad.md"), md).unwrap();

        let schema = ContentSchema {
            name: "post".into(),
            fields: vec![FieldDef {
                name: "title".into(),
                field_type: FieldType::String,
                required: true,
                default: None,
            }],
        };
        let errors = validate_content_dir(dir.path(), &[schema]).unwrap();
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("title"));
    }

    #[test]
    fn validate_content_dir_unknown_schema() {
        let dir = tempdir().unwrap();
        let md = "---\nschema: nonexistent\ntitle: X\n---\n\nBody";
        fs::write(dir.path().join("x.md"), md).unwrap();

        let schema = ContentSchema {
            name: "post".into(),
            fields: vec![],
        };
        let errors = validate_content_dir(dir.path(), &[schema]).unwrap();
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("unknown schema"));
    }

    #[test]
    fn validate_content_dir_file_without_schema_key() {
        let dir = tempdir().unwrap();
        let md = "---\ntitle: No Schema\n---\n\nBody";
        fs::write(dir.path().join("no_schema.md"), md).unwrap();

        let schema = ContentSchema {
            name: "post".into(),
            fields: vec![FieldDef {
                name: "title".into(),
                field_type: FieldType::String,
                required: true,
                default: None,
            }],
        };
        // Files without `schema` key are silently skipped
        let errors = validate_content_dir(dir.path(), &[schema]).unwrap();
        assert!(errors.is_empty());
    }

    // -------------------------------------------------------------------
    // Plugin trait
    // -------------------------------------------------------------------

    #[test]
    fn plugin_name() {
        let plugin = ContentValidationPlugin;
        assert_eq!(plugin.name(), "content-validation");
    }

    #[test]
    fn plugin_before_compile_no_schema_file() {
        let dir = tempdir().unwrap();
        let ctx = PluginContext::new(
            dir.path(),
            Path::new("build"),
            Path::new("public"),
            Path::new("templates"),
        );
        // No schema file — should succeed silently
        assert!(ContentValidationPlugin.before_compile(&ctx).is_ok());
    }

    #[test]
    fn plugin_before_compile_with_valid_content() {
        let dir = tempdir().unwrap();
        let content_dir = dir.path();

        // Write schema
        fs::write(
            content_dir.join("content.schema.toml"),
            r#"
[[schemas]]
name = "post"

[[schemas.fields]]
name = "title"
type = "string"
required = true
"#,
        )
        .unwrap();

        // Write valid content
        fs::write(
            content_dir.join("valid.md"),
            "---\ntitle: Hello World\nschema: post\n---\n\nContent here.",
        )
        .unwrap();

        let ctx = PluginContext::new(
            content_dir,
            Path::new("build"),
            Path::new("public"),
            Path::new("templates"),
        );
        assert!(ContentValidationPlugin.before_compile(&ctx).is_ok());
    }

    #[test]
    fn plugin_before_compile_with_invalid_content() {
        let dir = tempdir().unwrap();
        let content_dir = dir.path();

        // Write schema
        fs::write(
            content_dir.join("content.schema.toml"),
            r#"
[[schemas]]
name = "post"

[[schemas.fields]]
name = "title"
type = "string"
required = true
"#,
        )
        .unwrap();

        // Write content missing required field
        fs::write(
            content_dir.join("invalid.md"),
            "---\nschema: post\n---\n\nNo title here.",
        )
        .unwrap();

        let ctx = PluginContext::new(
            content_dir,
            Path::new("build"),
            Path::new("public"),
            Path::new("templates"),
        );
        let result = ContentValidationPlugin.before_compile(&ctx);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("title"),
            "error should mention 'title': {err_msg}"
        );
    }

    // -------------------------------------------------------------------
    // validate_only
    // -------------------------------------------------------------------

    #[test]
    fn validate_only_no_schemas() {
        let dir = tempdir().unwrap();
        assert!(validate_only(dir.path()).is_ok());
    }

    #[test]
    fn validate_only_with_errors() {
        let dir = tempdir().unwrap();
        let content_dir = dir.path();

        fs::write(
            content_dir.join("content.schema.toml"),
            r#"
[[schemas]]
name = "post"

[[schemas.fields]]
name = "title"
type = "string"
required = true
"#,
        )
        .unwrap();

        fs::write(
            content_dir.join("bad.md"),
            "---\nschema: post\n---\n\nMissing title.",
        )
        .unwrap();

        assert!(validate_only(content_dir).is_err());
    }

    #[test]
    fn validate_only_passes() {
        let dir = tempdir().unwrap();
        let content_dir = dir.path();

        fs::write(
            content_dir.join("content.schema.toml"),
            r#"
[[schemas]]
name = "post"

[[schemas.fields]]
name = "title"
type = "string"
required = true
"#,
        )
        .unwrap();

        fs::write(
            content_dir.join("good.md"),
            "---\ntitle: Valid\nschema: post\n---\n\nGood content.",
        )
        .unwrap();

        assert!(validate_only(content_dir).is_ok());
    }

    // -------------------------------------------------------------------
    // Edge cases
    // -------------------------------------------------------------------

    #[test]
    fn validate_multiple_errors_in_one_file() {
        let schema = ContentSchema {
            name: "post".into(),
            fields: vec![
                FieldDef {
                    name: "title".into(),
                    field_type: FieldType::String,
                    required: true,
                    default: None,
                },
                FieldDef {
                    name: "date".into(),
                    field_type: FieldType::Date,
                    required: true,
                    default: None,
                },
                FieldDef {
                    name: "count".into(),
                    field_type: FieldType::Integer,
                    required: true,
                    default: None,
                },
            ],
        };

        // All fields missing
        let fields: HashMap<String, String> = HashMap::new();
        let errors =
            validate_frontmatter(&fields, &schema, Path::new("test.md"), 1);
        assert_eq!(errors.len(), 3);
    }

    #[test]
    fn validate_enum_field_in_frontmatter() {
        let schema = ContentSchema {
            name: "post".into(),
            fields: vec![FieldDef {
                name: "status".into(),
                field_type: FieldType::Enum(vec![
                    "draft".into(),
                    "published".into(),
                ]),
                required: true,
                default: None,
            }],
        };

        let mut ok_fields = HashMap::new();
        let _ = ok_fields.insert("status".into(), "draft".into());
        assert!(validate_frontmatter(
            &ok_fields,
            &schema,
            Path::new("t.md"),
            1
        )
        .is_empty());

        let mut bad_fields = HashMap::new();
        let _ = bad_fields.insert("status".into(), "unknown".into());
        let errors =
            validate_frontmatter(&bad_fields, &schema, Path::new("t.md"), 1);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("allowed values"));
    }

    #[test]
    fn content_schema_clone_and_debug() {
        let schema = ContentSchema {
            name: "post".into(),
            fields: vec![FieldDef {
                name: "title".into(),
                field_type: FieldType::String,
                required: true,
                default: None,
            }],
        };
        let cloned = schema.clone();
        assert_eq!(cloned.name, "post");
        let debug = format!("{schema:?}");
        assert!(debug.contains("post"));
    }

    #[test]
    fn field_def_clone_and_debug() {
        let fd = FieldDef {
            name: "x".into(),
            field_type: FieldType::Bool,
            required: false,
            default: Some("true".into()),
        };
        let cloned = fd.clone();
        assert_eq!(cloned.name, "x");
        let debug = format!("{fd:?}");
        assert!(debug.contains("Bool"));
    }

    #[test]
    fn validation_error_clone_and_debug() {
        let err = ValidationError {
            file: PathBuf::from("x.md"),
            line: 5,
            message: "bad".into(),
        };
        let cloned = err.clone();
        assert_eq!(cloned.line, 5);
        let debug = format!("{err:?}");
        assert!(debug.contains("bad"));
    }

    #[test]
    fn field_type_clone_and_eq() {
        let a = FieldType::Enum(vec!["x".into()]);
        let b = a.clone();
        assert_eq!(a, b);
        assert_ne!(FieldType::String, FieldType::Bool);
    }
}

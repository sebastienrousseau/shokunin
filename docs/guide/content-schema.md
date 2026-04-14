<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# Content Schemas

SSG supports typed content collections with compile-time frontmatter validation via `content.schema.toml`.

## Schema File

Create `content/content.schema.toml` to define schemas for your content types:

```toml
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

[[schemas.fields]]
name = "tags"
type = "list"
required = false

[[schemas.fields]]
name = "category"
type = "enum(tutorial,guide,reference,blog)"
required = false
```

## Linking Pages to Schemas

Add `schema = "post"` to a page's frontmatter to validate it against the `post` schema:

```markdown
---
title: "My Post"
date: 2026-04-13
schema: "post"
tags: ["ssg", "tutorial"]
category: "tutorial"
---
```

## Field Types

| Type | Format | Example |
| :--- | :--- | :--- |
| `string` | Free-form text | `"Hello World"` |
| `date` | ISO 8601 (`YYYY-MM-DD`) | `2026-04-13` |
| `bool` | `true` or `false` | `true` |
| `integer` | Signed integer | `42` |
| `float` | Floating-point number | `3.14` |
| `list` | YAML/TOML array | `["a", "b"]` |
| `enum(v1,v2,...)` | One of a fixed set of values | `"tutorial"` |

## Field Properties

| Property | Type | Required | Description |
| :--- | :--- | :--- | :--- |
| `name` | String | Yes | Field name matching the frontmatter key |
| `type` | String | Yes | One of the field types above |
| `required` | Bool | No | Whether the field must be present (default: `false`) |
| `default` | String | No | Default value if the field is absent |

## Validation

### At Build Time

The `ContentValidationPlugin` runs in `before_compile` and validates every Markdown file that has `schema = "..."` in its frontmatter against the matching schema definition.

Validation errors are reported with the file path, field name, and expected type:

```
error: content/blog/post.md: field "date" expected type "date", got "not-a-date"
```

### Schema-Only Validation

Run validation without building:

```sh
ssg --validate -c content
```

This loads schemas from `content/content.schema.toml`, validates all matching pages, and exits with a non-zero code if any errors are found.

## Multiple Schemas

Define multiple schemas in the same file:

```toml
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

[[schemas]]
name = "page"

[[schemas.fields]]
name = "title"
type = "string"
required = true

[[schemas.fields]]
name = "description"
type = "string"
required = true
```

## Pages Without Schemas

Pages without `schema` in their frontmatter are not validated. This is intentional -- you can adopt schemas incrementally.

## Enum Validation

The `enum` type restricts values to a comma-separated list:

```toml
[[schemas.fields]]
name = "status"
type = "enum(draft,review,published)"
required = true
```

A page with `status: "archived"` would fail validation.

## Next Steps

- [Content](content.md) — frontmatter and Markdown authoring
- [CLI Reference](cli.md) — `--validate` flag
- [Plugins](plugins.md) — ContentValidationPlugin details

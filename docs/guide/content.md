<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# Content

SSG generates sites from Markdown files with YAML/TOML/JSON frontmatter.

## Frontmatter

Every Markdown file starts with a frontmatter block. YAML is the most common format:

```markdown
---
title: "Getting Started"
date: 2026-04-13
description: "An introduction to SSG"
schema: "post"
draft: false
tags: ["tutorial", "ssg"]
---

Your content here.
```

### Standard Frontmatter Fields

| Field | Type | Description |
| :--- | :--- | :--- |
| `title` | String | Page title, used in `<title>` and OG tags |
| `date` | Date | Publication date (ISO 8601: `YYYY-MM-DD`) |
| `description` | String | Meta description for SEO |
| `schema` | String | Content schema name for validation |
| `draft` | Bool | If `true`, excluded unless `--drafts` is passed |
| `tags` | List | Tags for taxonomy generation |
| `categories` | List | Categories for taxonomy generation |
| `template` | String | Override the default template for this page |
| `language` | String | Page language (BCP 47), overrides site default |

### Frontmatter Formats

SSG supports three formats, auto-detected by delimiter:

- **YAML** — delimited by `---`
- **TOML** — delimited by `+++`
- **JSON** — delimited by `{` and `}`

SSG also generates `.meta.json` sidecar files during the build for programmatic access to page metadata.

## Content Schemas

Define typed schemas in `content/content.schema.toml` to validate frontmatter at build time. Pages with `schema = "post"` are checked against the `post` schema.

See [Content Schemas](content-schema.md) for the full schema format.

Validate without building:

```sh
ssg --validate -c content
```

## GitHub Flavored Markdown (GFM)

SSG supports GFM extensions via the `MarkdownExtPlugin`:

- **Tables** — pipe-delimited tables with alignment
- **Strikethrough** — `~~deleted text~~`
- **Task lists** — `- [x] done` / `- [ ] todo`

## Shortcodes

Shortcodes are expanded before compilation. Syntax: `{{< name key="value" >}}`.

### Built-in Shortcodes

**YouTube embed:**
```markdown
{{< youtube id="dQw4w9WgXcQ" >}}
```

**GitHub Gist:**
```markdown
{{< gist user="octocat" id="1234567" >}}
```

**Figure with caption:**
```markdown
{{< figure src="/images/photo.jpg" alt="A photo" caption="Figure 1" >}}
```

**Admonition blocks:**
```markdown
{{< warning >}}
This is a warning.
{{< /warning >}}

{{< info >}}...{{< /info >}}
{{< tip >}}...{{< /tip >}}
{{< danger >}}...{{< /danger >}}
```

## Syntax Highlighting

Code blocks with language identifiers receive syntax highlighting via the `HighlightPlugin`:

````markdown
```rust
fn main() {
    println!("Hello, SSG!");
}
```
````

## Directory Structure

Content is organized in directories. Subdirectories create URL path segments:

```
content/
  index.md          -> /index.html
  about.md          -> /about/index.html
  blog/
    first-post.md   -> /blog/first-post/index.html
    second-post.md  -> /blog/second-post/index.html
```

## Draft Content

Mark pages as drafts in frontmatter:

```yaml
draft: true
```

Drafts are excluded from production builds. Include them with `--drafts`:

```sh
ssg -c content -o public -t templates --drafts
```

## Next Steps

- [Content Schemas](content-schema.md) — typed validation for frontmatter
- [Templates](templates.md) — control how content is rendered
- [SEO](seo.md) — metadata generated from frontmatter

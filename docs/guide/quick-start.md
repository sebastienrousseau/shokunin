<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# Quick Start

Build and serve a static site in under a minute.

## 1. Install

Pick any method from the [Installation guide](installation.md). The fastest:

```sh
cargo install ssg
```

## 2. Scaffold a New Site

```sh
ssg -n mysite -c content -o build -t templates
```

This creates the project directory with starter content, templates, and configuration.

## 3. Explore the Project Structure

```
mysite/
  content/          # Markdown files with YAML frontmatter
  templates/        # Tera HTML templates
  ssg.toml          # Site configuration (optional)
```

## 4. Customize Content

Edit or add Markdown files in `content/`. Every file needs frontmatter:

```markdown
---
title: "My First Post"
date: 2026-04-13
description: "Hello from SSG"
---

Your content here. SSG supports **GFM** (tables, task lists, strikethrough).
```

## 5. Build the Site

```sh
ssg -c content -o public -t templates
```

Generated HTML, RSS, sitemaps, and JSON-LD are written to `public/`.

## 6. Validate Content Schemas

If you use `content.schema.toml`, validate without building:

```sh
ssg --validate -c content
```

## 7. Start the Dev Server

```sh
ssg -c content -o public -t templates -s public
```

Opens a local server at `http://127.0.0.1:3000` with live reload.

## 8. Watch for Changes

Add `--watch` to rebuild automatically when content changes:

```sh
ssg -c content -o public -t templates -s public --watch
```

## 9. Include Drafts

Build with draft pages included:

```sh
ssg -c content -o public -t templates --drafts
```

## 10. Deploy

Generate platform-specific deployment configuration:

```sh
ssg --deploy netlify
ssg --deploy vercel
ssg --deploy cloudflare
ssg --deploy github
```

See the [Deployment guide](deployment.md) for details on each platform.

## Configuration File

Instead of passing flags every time, create `ssg.toml`:

```toml
site_name = "mysite"
content_dir = "content"
output_dir = "public"
template_dir = "templates"
base_url = "https://example.com"
site_title = "My Site"
site_description = "A site built with SSG"
language = "en"
```

Then build with:

```sh
ssg -f ssg.toml
```

## Next Steps

- [Configuration](configuration.md) — all `ssg.toml` fields
- [Content](content.md) — frontmatter and Markdown features
- [Templates](templates.md) — Tera template engine
- [CLI Reference](cli.md) — every flag explained

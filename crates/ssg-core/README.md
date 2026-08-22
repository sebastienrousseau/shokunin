# ssg-core

Core compilation pipeline for SSG — no system dependencies, WASM-compatible.

The parts of the generator that turn text into structure: Markdown to HTML,
frontmatter parsing, taxonomy term splitting, and slugs. It pulls in nothing
that needs a C toolchain or a filesystem, so the same code runs in a build,
in the browser, and at the edge.

## API

```rust
use ssg_core::{compile_markdown, parse_frontmatter, slugify, split_terms};

let html = compile_markdown("# Title\n\nBody.");
let (meta, body) = parse_frontmatter("---\ntitle: X\n---\nBody")?;
```

| function | does |
|---|---|
| `compile_markdown` | Markdown to HTML |
| `parse_frontmatter` | split and parse the YAML header |
| `compile_page` | both, into a rendered page |
| `split_terms` | split a tag/category list into terms |
| `slugify` | URL-safe slug for a term |
| `strip_html_tags` | plain text, for search indexing |

## Two behaviours worth knowing

**`split_terms` splits on non-ASCII separators.** Arabic `،`, fullwidth `，`,
ideographic `、` and `;` all separate terms, not just ASCII `,`. Splitting on
the comma of one script only meant an Arabic tag list collapsed into a single
term whose name was the whole list.

**`slugify` caps output at 200 bytes**, on a char boundary, trailing `-`
trimmed. The cap is in *bytes* because that is what filesystems limit: ext4
allows 255 bytes per component while APFS allows 255 *characters*. A slug
from a long non-Latin term could exceed the ext4 limit while building fine on
macOS — a Linux-only failure no contributor could reproduce locally.

## Licence

MIT or Apache-2.0, at your option.

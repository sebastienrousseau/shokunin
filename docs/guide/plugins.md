<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# Plugins

SSG uses plugins. They hook into the build. They run in order.

## Lifecycle Hooks

Every plugin can use three hooks:

| Hook | When | Use Cases |
| :--- | :--- | :--- |
| `before_compile` | Before build starts | Content prep, shortcodes, schema checks |
| `after_compile` | After build finishes | HTML fixes, SEO tags, sitemaps, minify |
| `on_serve` | Before dev server starts | Live-reload scripts, dev-mode setup |

## Built-in Plugins

SSG ships these plugins. They all run on their own.

### Content & Preprocessing
| Plugin | Hook | Description |
| :--- | :--- | :--- |
| `ShortcodePlugin` | `before_compile` | Shortcode expansion |
| `ContentValidationPlugin` | `before_compile` | Schema checks |
| `DraftPlugin` | `before_compile` | Filters drafts |
| `MarkdownExtPlugin` | `before_compile` | GFM tables, task lists |

### Compilation & Rendering
| Plugin | Hook | Description |
| :--- | :--- | :--- |
| `HighlightPlugin` | `after_compile` | Syntax colours |
| `TemplatePlugin` | `after_compile` | Template rendering |
| `PaginationPlugin` | `after_compile` | Page splits |
| `TaxonomyPlugin` | `after_compile` | Tag indexes |

### SEO & Metadata
| Plugin | Hook | Description |
| :--- | :--- | :--- |
| `SeoPlugin` | `after_compile` | OG and Twitter meta |
| `JsonLdPlugin` | `after_compile` | JSON-LD data |
| `CanonicalPlugin` | `after_compile` | Canonical URLs |
| `RobotsPlugin` | `after_compile` | robots.txt |

### Post-processing
| Plugin | Hook | Description |
| :--- | :--- | :--- |
| `SitemapFixPlugin` | `after_compile` | Sitemap fixes |
| `NewsSitemapFixPlugin` | `after_compile` | News sitemap |
| `RssAggregatePlugin` | `after_compile` | RSS feeds |
| `AtomFeedPlugin` | `after_compile` | Atom feeds |
| `HtmlFixPlugin` | `after_compile` | HTML fixes |
| `ManifestFixPlugin` | `after_compile` | Manifest fixes |
| `MinifyPlugin` | `after_compile` | Minify HTML |

### Features
| Plugin | Hook | Description |
| :--- | :--- | :--- |
| `AccessibilityPlugin` | `after_compile` | WCAG checks |
| `I18nPlugin` | `after_compile` | Hreflang tags |
| `SearchPlugin` | `after_compile` | Search index |
| `ImageOptimizationPlugin` | `after_compile` | WebP images |
| `FingerprintPlugin` | `after_compile` | Asset hashes |
| `AiPlugin` | `after_compile` | AI hooks |
| `CspPlugin` | `after_compile` | CSP + SRI |
| `IslandPlugin` | `after_compile` | Web islands |
| `LlmPlugin` | `after_compile` | LLM content |
| `DeployPlugin` | `after_compile` | Deploy config |

### Dev Server
| Plugin | Hook | Description |
| :--- | :--- | :--- |
| `LiveReloadPlugin` | `on_serve` | Live reload |

## Custom Plugin Example

Create a struct, add the Plugin trait, and register it:

```rust
use ssg::plugin::{Plugin, PluginContext, PluginManager};
use anyhow::Result;
use std::path::Path;

#[derive(Debug)]
struct LogPlugin;

impl Plugin for LogPlugin {
    fn name(&self) -> &str { "logger" }

    fn after_compile(&self, ctx: &PluginContext) -> Result<()> {
        println!("Done: {:?}", ctx.site_dir);
        Ok(())
    }
}

fn main() -> Result<()> {
    let mut pm = PluginManager::new();
    pm.register(LogPlugin);
    pm.register(ssg::plugins::MinifyPlugin);

    let ctx = PluginContext::new(
        Path::new("content"),
        Path::new("build"),
        Path::new("public"),
        Path::new("templates"),
    );
    pm.run_after_compile(&ctx)?;
    Ok(())
}
```

## Plugin Execution

Plugins run in the order you register them. Some plugins use Rayon for parallel file work. This respects `--jobs N`.

## Next Steps

- [Plugin API](plugin-api.md) — Trait details and testing
- [SEO](seo.md) — What the SEO plugins create
- [Accessibility](accessibility.md) — WCAG check details

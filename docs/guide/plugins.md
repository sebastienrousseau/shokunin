<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# Plugins

SSG uses a plugin pipeline to process content and output. Plugins hook into the build lifecycle and run in registration order.

## Lifecycle Hooks

Every plugin can implement three hooks:

| Hook | When | Use Cases |
| :--- | :--- | :--- |
| `before_compile` | Before compilation starts | Content preprocessing, shortcode expansion, schema validation |
| `after_compile` | After compilation completes | HTML post-processing, SEO injection, sitemap generation, minification |
| `on_serve` | Before dev server starts | Live-reload injection, dev-mode scripts |

## Built-in Plugins

SSG ships with these plugins, all running automatically:

### Content & Preprocessing
| Plugin | Hook | Description |
| :--- | :--- | :--- |
| `ShortcodePlugin` | `before_compile` | Expands `{{< shortcode >}}` syntax |
| `ContentValidationPlugin` | `before_compile` | Validates frontmatter against `content.schema.toml` |
| `DraftPlugin` | `before_compile` | Filters draft content unless `--drafts` is set |
| `MarkdownExtPlugin` | `before_compile` | GFM tables, strikethrough, task lists |

### Compilation & Rendering
| Plugin | Hook | Description |
| :--- | :--- | :--- |
| `HighlightPlugin` | `after_compile` | Syntax highlighting for code blocks |
| `TeraPlugin` | `after_compile` | Tera template rendering |
| `PaginationPlugin` | `after_compile` | Pagination for listing pages |
| `TaxonomyPlugin` | `after_compile` | Tag and category index generation |

### SEO & Metadata
| Plugin | Hook | Description |
| :--- | :--- | :--- |
| `SeoPlugin` | `after_compile` | Open Graph and Twitter Card meta tags |
| `JsonLdPlugin` | `after_compile` | JSON-LD structured data (Article, WebPage) |
| `CanonicalPlugin` | `after_compile` | Canonical URL injection |
| `RobotsPlugin` | `after_compile` | `robots.txt` generation |

### Post-processing
| Plugin | Hook | Description |
| :--- | :--- | :--- |
| `SitemapFixPlugin` | `after_compile` | Sitemap XML fixes and validation |
| `NewsSitemapFixPlugin` | `after_compile` | Google News sitemap generation |
| `RssAggregatePlugin` | `after_compile` | RSS 2.0 feed aggregation |
| `AtomFeedPlugin` | `after_compile` | Atom feed generation |
| `HtmlFixPlugin` | `after_compile` | HTML output corrections |
| `ManifestFixPlugin` | `after_compile` | Web manifest fixes |
| `MinifyPlugin` | `after_compile` | HTML minification |

### Features
| Plugin | Hook | Description |
| :--- | :--- | :--- |
| `AccessibilityPlugin` | `after_compile` | WCAG 2.1 AA validation |
| `I18nPlugin` | `after_compile` | Hreflang injection, per-locale sitemaps |
| `SearchPlugin` | `after_compile` | Search index and UI generation |
| `ImageOptimizationPlugin` | `after_compile` | Responsive `<picture>` with AVIF/WebP |
| `FingerprintPlugin` | `after_compile` | Asset fingerprinting and SRI hashes |
| `AiPlugin` | `after_compile` | AI-readiness hooks, `llms.txt` |
| `DeployPlugin` | `after_compile` | Deployment config generation |

### Dev Server
| Plugin | Hook | Description |
| :--- | :--- | :--- |
| `LiveReloadPlugin` | `on_serve` | WebSocket live-reload injection |

## Custom Plugin Example

```rust
use ssg::plugin::{Plugin, PluginContext, PluginManager};
use anyhow::Result;
use std::path::Path;

#[derive(Debug)]
struct LogPlugin;

impl Plugin for LogPlugin {
    fn name(&self) -> &str { "logger" }

    fn after_compile(&self, ctx: &PluginContext) -> Result<()> {
        println!("Site compiled to {:?}", ctx.site_dir);
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

Plugins run in registration order. Some plugins (like `MinifyPlugin`) use Rayon `par_iter` internally for parallel file processing while respecting `--jobs N` thread limits.

## Next Steps

- [Plugin API](plugin-api.md) — trait details, PluginContext, PluginCache, testing
- [SEO](seo.md) — what the SEO plugins generate
- [Accessibility](accessibility.md) — WCAG checking details

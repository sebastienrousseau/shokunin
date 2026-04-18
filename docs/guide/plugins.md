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
| `ShortcodePlugin` | `before_compile` | Expands `{{< shortcode >}}` syntax |
| `ContentValidationPlugin` | `before_compile` | Checks frontmatter against schema |
| `DraftPlugin` | `before_compile` | Filters draft content unless `--drafts` is set |
| `MarkdownExtPlugin` | `before_compile` | GFM tables, strikethrough, task lists |

### Compilation & Rendering
| Plugin | Hook | Description |
| :--- | :--- | :--- |
| `HighlightPlugin` | `after_compile` | Syntax colours for code blocks |
| `TemplatePlugin` | `after_compile` | MiniJinja template rendering |
| `PaginationPlugin` | `after_compile` | Page splits for list pages |
| `TaxonomyPlugin` | `after_compile` | Tag and category indexes |

### SEO & Metadata
| Plugin | Hook | Description |
| :--- | :--- | :--- |
| `SeoPlugin` | `after_compile` | Open Graph and Twitter Card meta tags |
| `JsonLdPlugin` | `after_compile` | JSON-LD data (Article, WebPage) |
| `CanonicalPlugin` | `after_compile` | Adds canonical URL links |
| `RobotsPlugin` | `after_compile` | `robots.txt` creation |

### Post-processing
| Plugin | Hook | Description |
| :--- | :--- | :--- |
| `SitemapFixPlugin` | `after_compile` | Sitemap XML fixes and checks |
| `NewsSitemapFixPlugin` | `after_compile` | Google News sitemap creation |
| `RssAggregatePlugin` | `after_compile` | RSS 2.0 feed building |
| `AtomFeedPlugin` | `after_compile` | Atom feed creation |
| `HtmlFixPlugin` | `after_compile` | HTML output fixes |
| `ManifestFixPlugin` | `after_compile` | Web manifest fixes |
| `MinifyPlugin` | `after_compile` | HTML minify |

### Features
| Plugin | Hook | Description |
| :--- | :--- | :--- |
| `AccessibilityPlugin` | `after_compile` | WCAG 2.1 AA checks |
| `I18nPlugin` | `after_compile` | Hreflang tags, locale sitemaps |
| `SearchPlugin` | `after_compile` | Search index and UI |
| `ImageOptimizationPlugin` | `after_compile` | `<picture>` with AVIF/WebP |
| `FingerprintPlugin` | `after_compile` | Asset hashes and SRI |
| `AiPlugin` | `after_compile` | AI hooks, `llms.txt` |
| `CspPlugin` | `after_compile` | CSP hardening, inline extraction + SRI |
| `IslandPlugin` | `after_compile` | Web Component islands, lazy hydration |
| `LlmPlugin` | `after_compile` | Local LLM content augmentation |
| `DeployPlugin` | `after_compile` | Deploy config files |

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

Plugins run in register order. Some (like `MinifyPlugin`) use Rayon `par_iter` inside. This runs files in parallel. It still respects `--jobs N`.

## Next Steps

- [Plugin API](plugin-api.md) — Trait details and testing
- [SEO](seo.md) — What the SEO plugins create
- [Accessibility](accessibility.md) — WCAG check details

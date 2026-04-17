<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# Plugin API

Technical reference for building custom SSG plugins.

## The Plugin Trait

Every plugin implements the `Plugin` trait from `ssg::plugin`:

```rust
pub trait Plugin: fmt::Debug + Send + Sync {
    /// Returns the unique name of this plugin.
    fn name(&self) -> &str;

    /// Called before site compilation begins.
    fn before_compile(&self, _ctx: &PluginContext) -> Result<()> {
        Ok(())
    }

    /// Called after site compilation completes.
    fn after_compile(&self, _ctx: &PluginContext) -> Result<()> {
        Ok(())
    }

    /// Called before the development server starts.
    fn on_serve(&self, _ctx: &PluginContext) -> Result<()> {
        Ok(())
    }
}
```

All hooks have default no-op implementations. Override only the hooks you need.

**Requirements:** Plugins must be `Debug + Send + Sync` to support parallel execution.

## PluginContext

The context passed to every hook:

```rust
pub struct PluginContext {
    pub content_dir: PathBuf,   // Source content directory
    pub build_dir: PathBuf,     // Build/output directory
    pub site_dir: PathBuf,      // Final site directory
    pub template_dir: PathBuf,  // Template directory
    pub config: Option<SsgConfig>,     // Site configuration
    pub cache: Option<PluginCache>,    // Incremental build cache
}
```

### Constructors

```rust
// Basic context (no config, no cache)
let ctx = PluginContext::new(
    Path::new("content"),
    Path::new("build"),
    Path::new("public"),
    Path::new("templates"),
);

// Context with configuration
let ctx = PluginContext::with_config(
    Path::new("content"),
    Path::new("build"),
    Path::new("public"),
    Path::new("templates"),
    config,
);
```

## PluginCache

Content-addressed cache for incremental builds. Tracks `path -> FNV-1a hash` mappings and persists to `.ssg-plugin-cache.json`.

```rust
// Load from site directory (returns empty cache if missing)
let cache = PluginCache::load(site_dir);

// Check if a file has changed since last recorded
if cache.has_changed(&path) {
    // Process the file
    cache.update(&path);
}

// Save to disk
cache.save(site_dir)?;
```

Use the cache in your plugin to skip unchanged files:

```rust
fn after_compile(&self, ctx: &PluginContext) -> Result<()> {
    let cache = ctx.cache.as_ref();
    for file in html_files {
        if cache.is_none_or(|c| c.has_changed(&file)) {
            // Process file
        }
    }
    Ok(())
}
```

## PluginManager

Manages plugin registration and lifecycle execution:

```rust
let mut pm = PluginManager::new();
pm.register(MyPlugin);
pm.register(ssg::plugins::MinifyPlugin);

assert_eq!(pm.len(), 2);
assert!(!pm.is_empty());

// Execute hooks
pm.run_before_compile(&ctx)?;
pm.run_after_compile(&ctx)?;
pm.run_on_serve(&ctx)?;
```

Plugins run in registration order.

## Writing a Custom Plugin

### Minimal Example

```rust
use ssg::plugin::{Plugin, PluginContext};
use anyhow::Result;

#[derive(Debug)]
struct WordCountPlugin;

impl Plugin for WordCountPlugin {
    fn name(&self) -> &str { "word-count" }

    fn after_compile(&self, ctx: &PluginContext) -> Result<()> {
        let html_files = std::fs::read_dir(&ctx.site_dir)?;
        let mut total = 0usize;

        for entry in html_files.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "html") {
                let content = std::fs::read_to_string(&path)?;
                total += content.split_whitespace().count();
            }
        }

        println!("[word-count] Total words: {total}");
        Ok(())
    }
}
```

## Testing Plugins

Use `tempfile::TempDir` to create an isolated site directory, populate it with test HTML, construct a `PluginContext`, and call your hook:

```rust
#[test]
fn test_my_plugin() {
    let tmp = tempfile::TempDir::new().unwrap();
    let site_dir = tmp.path().join("site");
    std::fs::create_dir_all(&site_dir).unwrap();
    std::fs::write(site_dir.join("index.html"), "<html><body>Hello</body></html>").unwrap();

    let ctx = PluginContext::new(
        Path::new("content"), Path::new("build"), &site_dir, Path::new("templates"),
    );
    MyPlugin.after_compile(&ctx).unwrap();
}
```

## Error Handling

Plugins use `anyhow::Result<()>`. Return errors to halt the pipeline:

```rust
fn after_compile(&self, ctx: &PluginContext) -> Result<()> {
    if !ctx.site_dir.exists() {
        return Ok(()); // Skip gracefully
    }
    // Or fail:
    anyhow::bail!("Something went wrong");
}
```

## Next Steps

- [Plugins](plugins.md) — overview and built-in plugin list
- [Configuration](configuration.md) — SsgConfig fields available in PluginContext

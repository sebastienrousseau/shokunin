# Contributing to SSG

Welcome! We're thrilled that you're interested in contributing to SSG. Whether you're looking to report a bug, suggest a feature, or submit code, this guide will help you get started.

## Development setup

### Prerequisites

- [Rust](https://rustup.rs/) 1.74.0 or later
- Git with commit signing configured (see below)

### Getting started

```bash
git clone https://github.com/sebastienrousseau/shokunin.git
cd shokunin
cargo build
cargo test
```

### Useful commands

```bash
make build       # Build the project
make test        # Run all tests
make lint        # Lint with Clippy
make format      # Format code with rustfmt
make deny        # Check dependencies for security/license issues
```

## Signed commits

All commits must be signed. Configure Git to sign commits with either GPG or SSH:

```bash
# SSH signing (recommended)
git config --global gpg.format ssh
git config --global user.signingkey ~/.ssh/id_ed25519.pub
git config --global commit.gpgsign true

# Or GPG signing
git config --global commit.gpgsign true
git config --global user.signingkey YOUR_GPG_KEY_ID
```

Verify your setup:

```bash
echo "test" | git commit --allow-empty -S -m "test signing"
git log --show-signature -1
```

## Architecture

```
src/
  lib.rs            — Orchestrator: run() → plugin pipeline → compile → serve
  cmd.rs            — CLI parsing and SsgConfig
  plugin.rs         — Plugin trait + PluginManager
  frontmatter.rs    — Frontmatter extraction + .meta.json sidecars
  tera_engine.rs    — Tera template engine wrapper
  tera_plugin.rs    — Tera rendering plugin
  seo.rs            — SeoPlugin, JsonLdPlugin, CanonicalPlugin, RobotsPlugin
  ai.rs             — AI readiness (llms.txt, meta tags, alt validation)
  accessibility.rs  — WCAG checker + ARIA validation
  search.rs         — Client-side search index generation
  highlight.rs      — Syntax highlighting for code blocks
  shortcodes.rs     — Shortcode expansion (youtube, gist, figure, admonitions)
  assets.rs         — Asset fingerprinting + SRI hashes
  deploy.rs         — Deployment adapters (Netlify, Vercel, Cloudflare, GitHub Pages)
  cache.rs          — Incremental build cache
  stream.rs         — High-performance streaming I/O
  watch.rs          — File watcher for live rebuild
  livereload.rs     — WebSocket live-reload injection
```

### Writing a Plugin

```rust
use ssg::plugin::{Plugin, PluginContext};
use anyhow::Result;

#[derive(Debug)]
struct MyPlugin;

impl Plugin for MyPlugin {
    fn name(&self) -> &str { "my-plugin" }
    fn after_compile(&self, ctx: &PluginContext) -> Result<()> {
        if let Some(config) = &ctx.config {
            println!("Site: {}", config.site_name);
        }
        Ok(())
    }
}
```

## How to contribute

### Reporting bugs

- Open an [issue](https://github.com/sebastienrousseau/shokunin/issues/new) with a descriptive title.
- Include steps to reproduce, expected vs actual behavior, and your OS/Rust version.

### Suggesting features

- Open an [issue](https://github.com/sebastienrousseau/shokunin/issues/new) describing the use case and proposed solution.

### Submitting code

1. Fork the repository.
2. Create a feature branch: `git checkout -b feat/my-feature`
3. Make your changes in `src/`. Add tests for new functionality.
4. Ensure all checks pass:
   ```bash
   cargo fmt --all -- --check
   cargo clippy --all-targets
   cargo test
   ```
5. Commit with a signed, [conventional commit](https://www.conventionalcommits.org/) message:
   ```bash
   git commit -S -m "feat: add support for TOML frontmatter"
   ```
6. Push and open a pull request against `main`.

### Pull request guidelines

- Keep PRs focused on a single change.
- Include a clear description of what changed and why.
- Ensure CI passes before requesting review.
- Reference related issues (e.g., `Closes #123`).

## Code of Conduct

Please read our [Code of Conduct](.github/CODE-OF-CONDUCT.md) before participating.

## License

By contributing, you agree that your contributions will be licensed under the same dual license as the project (MIT OR Apache-2.0).

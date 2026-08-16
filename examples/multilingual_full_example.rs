#![allow(clippy::unwrap_used, clippy::expect_used)]
// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! # `multilingual_full` — nested-locale content showcase
//!
//! ## What this example demonstrates
//!
//! - **`content/<lang>/<slug>.md` layout** — 5 locales (en / fr / de /
//!   es / ja), each with an `index.md`, 5 posts and one translated-slug
//!   page. 37 source files total, mirroring the Jekyll
//!   `_posts/<lang>/` pattern.
//! - **Recursive content walk** — every per-locale post produces its
//!   own page under `public/<lang>/post-N/index.html` and a per-locale
//!   `public/<lang>/index.html` landing.
//! - **Translated slugs via `translation_key`** — `about`,
//!   `a-propos`, `ueber-uns`, `acerca-de` and `gaiyou` are the same
//!   document at five different paths. Pages are paired by front-matter
//!   key rather than by path, so all five end up as reciprocal
//!   `hreflang` alternates of one another. Without a key they would
//!   each be a singleton and receive no alternates at all — silently.
//!   The `post-N` pages, which share a slug across locales, still pair
//!   by path with no key needed; both mechanisms run side by side here.
//! - **Per-locale hreflang** — each generated page carries the
//!   `language` / `hreflang` frontmatter so search engines see distinct
//!   language editions.
//!
//! The runner asserts reciprocity, not just existence: a page that
//! renders but pairs with nothing is precisely the failure mode
//! `translation_key` exists to prevent, and it is invisible to a
//! "did the file get written" check.
//!
//! ## Why this is its own example
//!
//! The existing `examples/multilingual_example.rs` shows the runtime
//! language-switcher + per-locale search index pattern using a single
//! flat content tree. **This** example shows the underlying source
//! layout — physically separate per-locale source files under
//! `content/<lang>/`.
//!
//! ## Dependency note
//!
//! The recursive walk in `add()` silently skipped subdirectories
//! before `staticdatagen 0.0.10` ([upstream
//! #70](https://github.com/sebastienrousseau/staticdatagen/issues/70)).
//! The workspace has been on 0.0.11 since v0.0.48, so every per-locale
//! URL lands. The runner still only warns under `CI` rather than
//! failing, so a future upstream regression shows up as missing pages
//! in the log instead of a red build on someone else's change.
//!
//! ## Run it
//!
//! ```sh
//! cargo run --release --example multilingual_full
//! ```

use std::error::Error;
use std::path::Path;

use ssg::pipeline::compile_site;

const LOCALES: &[&str] = &["en", "fr", "de", "es", "ja"];
const POSTS_PER_LOCALE: usize = 5;

const BASE_URL: &str = "https://example.com";
const SITE_NAME: &str = "multilingual_full example";

/// The translated-slug family: one page per locale, all sharing
/// `translation_key: "about"` in front matter, each at a slug of its
/// own. Path matching pairs `post-1` across locales for free because
/// the slug is identical everywhere; it cannot pair these, which is
/// exactly what the key is for.
const ABOUT_SLUGS: &[(&str, &str)] = &[
    ("en", "about"),
    ("fr", "a-propos"),
    ("de", "ueber-uns"),
    ("es", "acerca-de"),
    ("ja", "gaiyou"),
];

/// Returns the `href` of the `<link rel="alternate">` for `locale`,
/// or `None` when the page declares no alternate for it.
fn alternate_href<'a>(html: &'a str, locale: &str) -> Option<&'a str> {
    let needle = format!("hreflang=\"{locale}\" href=\"");
    let start = html.find(&needle)? + needle.len();
    let rest = html.get(start..)?;
    rest.find('"').map(|end| &rest[..end])
}

fn main() -> Result<(), Box<dyn Error>> {
    let content_dir = Path::new("./examples/multilingual_full/content");
    let site_dir = Path::new("./examples/multilingual_full/public");
    let template_dir = Path::new("./examples/templates");

    let build_dir = Path::new("./examples/multilingual_full/build");

    // Wipe previous run output so re-runs are deterministic.
    let _ = std::fs::remove_dir_all(build_dir);
    let _ = std::fs::remove_dir_all(site_dir);
    std::fs::create_dir_all(build_dir)?;

    println!(
        "multilingual_full: compiling 37 source files across 5 locales..."
    );

    // `compile_site` routes through the v0.0.45 content stager
    // (default-layout injection, template stub fill-in, etc.) so this
    // example doesn't need to wire the shim layer by hand.
    //
    // It registers no plugins, though, so the pass below is explicit.
    compile_site(build_dir, content_dir, site_dir, template_dir)?;

    // Front-matter sidecars carry `translation_key` from the source
    // markdown to the i18n plugin, which runs after compilation and so
    // can no longer see the front matter itself. `compile_site` leaves
    // none behind here, so write them where `resolve_sidecar_dir`
    // looks once the build directory is gone: `<site>/.meta`.
    let sidecar_dir = site_dir.join(".meta");
    let sidecars = ssg::frontmatter::emit_sidecars(content_dir, &sidecar_dir)?;
    println!("multilingual_full: wrote {sidecars} front-matter sidecars");

    // SEO + i18n pass.
    //
    // The i18n half is what makes the translated-slug assertions below
    // meaningful — without it the pages are built but unlinked.
    //
    // The SEO half is what makes the pages *conformant*. `compile_site`
    // alone emits no Open Graph tags and no `twitter:card`, so every
    // page this example produced failed the universal HTML invariants
    // in `tests/element_presence.rs` (issue #676). Registering the
    // three SEO plugins is enough to close that without dragging in
    // the postprocess chain, whose manifest / CNAME / humans templates
    // need front matter this example's content does not carry.
    {
        use ssg::i18n::{I18nConfig, I18nPlugin, UrlPrefixStrategy};
        use ssg::plugin::{PluginContext, PluginManager};
        use ssg::seo::{CanonicalPlugin, JsonLdPlugin, SeoPlugin};

        let mut plugins = PluginManager::new();
        plugins.register(SeoPlugin);
        plugins.register(JsonLdPlugin::from_site(BASE_URL, SITE_NAME));
        plugins.register(CanonicalPlugin::new(BASE_URL.to_string()));
        plugins.register(I18nPlugin::new(I18nConfig {
            default_locale: "en".to_string(),
            locales: LOCALES.iter().map(|l| (*l).to_string()).collect(),
            url_prefix: UrlPrefixStrategy::SubPath,
        }));

        let ctx =
            PluginContext::new(content_dir, build_dir, site_dir, template_dir);
        plugins.run_after_compile(&ctx)?;
        plugins.run_fused_transforms(&ctx)?;
        println!("multilingual_full: SEO + i18n pass complete — auditing:");
    }

    let mut found = 0usize;
    let mut missing = 0usize;
    for &lang in LOCALES {
        // Posts land at `<lang>/post-N/index.html`.
        for i in 1..=POSTS_PER_LOCALE {
            let p = site_dir
                .join(lang)
                .join(format!("post-{i}"))
                .join("index.html");
            if p.exists() {
                found += 1;
            } else {
                missing += 1;
                eprintln!("  MISSING: {}", p.display());
            }
        }
        // Per-locale `index.md` lands at `<lang>/index.html`. It used
        // to gain a directory level (`<lang>/index/index.html`)
        // because staticdatagen compares the whole processed name
        // against "index"; the v0.0.50 content stager side-steps that.
        let idx = site_dir.join(lang).join("index.html");
        if idx.exists() {
            found += 1;
        } else {
            missing += 1;
            eprintln!("  MISSING: {}", idx.display());
        }
    }

    // The translated-slug family: every page must exist, and every
    // page must advertise the other four. A page that renders but
    // pairs with nothing is the exact silent failure `translation_key`
    // was added to fix, so "it built" is not the assertion worth
    // making here — "it is reciprocally linked" is.
    let mut unpaired = 0usize;
    for &(lang, slug) in ABOUT_SLUGS {
        let page = site_dir.join(lang).join(slug).join("index.html");
        if !page.exists() {
            missing += 1;
            eprintln!("  MISSING: {}", page.display());
            continue;
        }
        found += 1;

        let html = std::fs::read_to_string(&page)?;
        for &(other, other_slug) in ABOUT_SLUGS {
            if other == lang {
                continue;
            }
            // Match the alternate link itself, not the path anywhere
            // on the page: the generated navbar links every page in
            // the site, so a bare `contains("/fr/a-propos")` passes
            // even when the hreflang block is empty.
            let want = format!("/{other}/{other_slug}");
            let paired = alternate_href(&html, other)
                .is_some_and(|href| href.contains(&want));
            if !paired {
                unpaired += 1;
                eprintln!(
                    "  UNPAIRED: /{lang}/{slug}/ has no hreflang=\"{other}\" \
                     alternate pointing at {want}"
                );
            }
        }
    }

    let expected = LOCALES.len() * (POSTS_PER_LOCALE + 1) + ABOUT_SLUGS.len();
    println!(
        "  found {found}/{expected} per-locale pages  \
         ({missing} missing, {unpaired} unpaired)"
    );

    println!();

    // Unpaired translations fail hard, everywhere, including CI.
    // Page-count shortfalls stay soft because they track an upstream
    // dependency we don't control; hreflang pairing is this crate's
    // own behaviour, and a silent regression in it is what shipped
    // broken in the first place.
    if unpaired > 0 {
        eprintln!(
            "  ✗ {unpaired} missing alternate(s) — translated-slug pairing \
             via `translation_key` has regressed."
        );
        std::process::exit(1);
    }

    if missing > 0 {
        println!(
            "  ⚠ {missing} pages missing — staticdatagen recursive walk regressed?"
        );
        // Exit non-zero only when we have a regression worth flagging
        // — staticdatagen 0.0.11 should produce all 35 pages.
        if std::env::var("CI").is_err() {
            std::process::exit(1);
        }
    } else {
        println!(
            "  ✓ every per-locale page landed — nested-locale walk works."
        );
    }

    Ok(())
}

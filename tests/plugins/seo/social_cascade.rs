// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Spec B8 acceptance tests — social-meta derivation cascade
//! (v0.0.47 plan §5).
//!
//! Every og/twitter field derives from the page's own front matter
//! when not explicitly set: `twitter_title ⇐ seo_title ⇐ title`,
//! `og_image/twitter_image ⇐ banner ⇐ image`,
//! `twitter_description/og_description ⇐ description`, and the
//! twitter card type defaults to `summary_large_image` when an image
//! exists (else `summary`). Explicit per-field front matter always
//! wins, and values never bleed between pages or from global config.

use ssg::plugin::{Plugin, PluginContext};
use ssg::seo::SeoPlugin;
use std::fs;
use std::path::Path;

/// Builds a `PluginContext` rooted under `dir` so front-matter
/// sidecars written to `<dir>/build/.meta` are found for pages under
/// `<dir>/site`.
fn ctx(dir: &Path) -> PluginContext {
    PluginContext::new(
        Path::new("content"),
        &dir.join("build"),
        &dir.join("site"),
        Path::new("templates"),
    )
}

/// Writes the `.meta.json` front-matter sidecar for the
/// site-relative page `rel`.
fn write_sidecar(dir: &Path, rel: &str, json: &str) {
    let sidecar = dir
        .join("build")
        .join(".meta")
        .join(rel)
        .with_extension("meta.json");
    fs::create_dir_all(sidecar.parent().unwrap()).unwrap();
    fs::write(sidecar, json).unwrap();
}

/// Extracts the `content` of a `<meta name=…>`/`<meta property=…>`
/// tag from rendered HTML.
fn meta(html: &str, attr: &str) -> Option<String> {
    for prefix in [
        format!(r#"<meta name="{attr}" content=""#),
        format!(r#"<meta property="{attr}" content=""#),
    ] {
        if let Some(pos) = html.find(prefix.as_str()) {
            let after = &html[pos + prefix.len()..];
            if let Some(end) = after.find('"') {
                return Some(after[..end].to_string());
            }
        }
    }
    None
}

/// Acceptance (spec B8): a post whose front matter carries ONLY
/// `title` + `description` + `banner` gets a complete, mutually
/// consistent og:*/twitter:* set.
#[test]
fn title_description_banner_derive_complete_social_set() {
    let dir = tempfile::tempdir().unwrap();
    write_sidecar(
        dir.path(),
        "posts/hello/index.html",
        r#"{"title":"Hello World","description":"A greeting.","banner":"/img/hello.webp"}"#,
    );
    let c = ctx(dir.path());
    let html = concat!(
        r#"<html lang="en"><head><title>Hello World</title></head>"#,
        "<body><p>Hi.</p></body></html>"
    );
    let page = dir.path().join("site/posts/hello/index.html");
    let out = SeoPlugin.transform_html(html, &page, &c).unwrap();

    assert_eq!(meta(&out, "og:title").as_deref(), Some("Hello World"));
    assert_eq!(meta(&out, "twitter:title").as_deref(), Some("Hello World"));
    assert_eq!(meta(&out, "og:description").as_deref(), Some("A greeting."));
    assert_eq!(
        meta(&out, "twitter:description").as_deref(),
        Some("A greeting.")
    );
    assert_eq!(meta(&out, "description").as_deref(), Some("A greeting."));
    assert_eq!(meta(&out, "og:image").as_deref(), Some("/img/hello.webp"));
    assert_eq!(
        meta(&out, "twitter:image").as_deref(),
        Some("/img/hello.webp")
    );
    // Banner exists ⇒ large-image card.
    assert_eq!(
        meta(&out, "twitter:card").as_deref(),
        Some("summary_large_image")
    );
    // og:locale is emitted by the A5 resolver alongside.
    assert_eq!(meta(&out, "og:locale").as_deref(), Some("en"));

    // Mutual consistency between the og:* and twitter:* families.
    assert_eq!(meta(&out, "og:title"), meta(&out, "twitter:title"));
    assert_eq!(
        meta(&out, "og:description"),
        meta(&out, "twitter:description")
    );
    assert_eq!(meta(&out, "og:image"), meta(&out, "twitter:image"));
}

/// Explicit per-field front matter beats every derived value.
#[test]
fn explicit_fields_always_win_over_cascade() {
    let dir = tempfile::tempdir().unwrap();
    write_sidecar(
        dir.path(),
        "p/index.html",
        concat!(
            r#"{"title":"Base","seo_title":"Seo","twitter_title":"Tweet Me","#,
            r#""banner":"/b.png","twitter_image":"/tw.png","twitter_card":"summary"}"#
        ),
    );
    let c = ctx(dir.path());
    let html = "<html><head><title>Base</title></head><body>x</body></html>";
    let out = SeoPlugin
        .transform_html(html, &dir.path().join("site/p/index.html"), &c)
        .unwrap();

    // twitter_title explicit; og_title falls back to seo_title.
    assert_eq!(meta(&out, "twitter:title").as_deref(), Some("Tweet Me"));
    assert_eq!(meta(&out, "og:title").as_deref(), Some("Seo"));
    // twitter_image explicit; og_image falls back to banner.
    assert_eq!(meta(&out, "twitter:image").as_deref(), Some("/tw.png"));
    assert_eq!(meta(&out, "og:image").as_deref(), Some("/b.png"));
    // Explicit card beats the image-derived summary_large_image.
    assert_eq!(meta(&out, "twitter:card").as_deref(), Some("summary"));
}

/// The stale-field bug class (spec B8): two pages differing only in
/// `title` must resolve their own titles — no cross-page or global
/// config bleed-through.
#[test]
fn no_stale_bleed_between_pages_differing_only_in_title() {
    let dir = tempfile::tempdir().unwrap();
    for (rel, title) in [
        ("alpha/index.html", "Alpha Title"),
        ("beta/index.html", "Beta Title"),
    ] {
        write_sidecar(
            dir.path(),
            rel,
            &format!(
                r#"{{"title":"{title}","description":"shared","banner":"/s.png"}}"#
            ),
        );
    }
    let mut c = ctx(dir.path());
    c.config = Some(ssg::cmd::SsgConfig {
        site_name: "Global Site".to_string(),
        ..ssg::cmd::SsgConfig::default()
    });
    let html = "<html><head><title>t</title></head><body>x</body></html>";

    let out_a = SeoPlugin
        .transform_html(html, &dir.path().join("site/alpha/index.html"), &c)
        .unwrap();
    let out_b = SeoPlugin
        .transform_html(html, &dir.path().join("site/beta/index.html"), &c)
        .unwrap();

    assert_eq!(meta(&out_a, "og:title").as_deref(), Some("Alpha Title"));
    assert_eq!(
        meta(&out_a, "twitter:title").as_deref(),
        Some("Alpha Title")
    );
    assert_eq!(meta(&out_b, "og:title").as_deref(), Some("Beta Title"));
    assert_eq!(meta(&out_b, "twitter:title").as_deref(), Some("Beta Title"));
    assert!(!out_a.contains("Beta Title"));
    assert!(!out_b.contains("Alpha Title"));
    assert!(!out_a.contains("Global Site"));
    assert!(!out_b.contains("Global Site"));
}

/// Without any image signal the card type defaults to `summary`.
#[test]
fn card_defaults_to_summary_without_image() {
    let dir = tempfile::tempdir().unwrap();
    write_sidecar(
        dir.path(),
        "plain/index.html",
        r#"{"title":"Plain","description":"No image here."}"#,
    );
    let c = ctx(dir.path());
    let html = "<html><head><title>Plain</title></head><body>x</body></html>";
    let out = SeoPlugin
        .transform_html(html, &dir.path().join("site/plain/index.html"), &c)
        .unwrap();
    assert_eq!(meta(&out, "twitter:card").as_deref(), Some("summary"));
    assert_eq!(meta(&out, "og:image"), None);
    assert_eq!(meta(&out, "twitter:image"), None);
}

/// `image` (front matter) backs up `banner` in the image cascade.
#[test]
fn image_field_used_when_banner_absent() {
    let dir = tempfile::tempdir().unwrap();
    write_sidecar(
        dir.path(),
        "img/index.html",
        r#"{"title":"T","description":"D","image":"/only-image.png"}"#,
    );
    let c = ctx(dir.path());
    let html = "<html><head><title>T</title></head><body>x</body></html>";
    let out = SeoPlugin
        .transform_html(html, &dir.path().join("site/img/index.html"), &c)
        .unwrap();
    assert_eq!(meta(&out, "og:image").as_deref(), Some("/only-image.png"));
    assert_eq!(
        meta(&out, "twitter:image").as_deref(),
        Some("/only-image.png")
    );
    assert_eq!(
        meta(&out, "twitter:card").as_deref(),
        Some("summary_large_image")
    );
}

// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Image optimisation + alt-text gate.
//!
//! Per page:
//! - Every `<img>` has an `alt` attribute (error).
//! - Every `<img>` has explicit `width` + `height` (warn — required
//!   to avoid CLS).
//! - Each referenced image file has a sibling `.webp` or `.avif` (warn).
//! - Each referenced image file is under the configured size budget
//!   (warn).

use super::super::{AuditGate, AuditOptions, Finding, Severity, Site};
use super::hreflang_attr;

const NAME: &str = "images";

/// Image optimisation + alt-text gate.
#[derive(Debug, Clone, Copy)]
pub struct ImagesGate;

impl AuditGate for ImagesGate {
    fn name(&self) -> &'static str {
        NAME
    }

    fn explain(&self) -> &'static str {
        "Per <img>: asserts alt text (error), explicit width + height \
         (warn), and that the referenced file has a sibling .webp or \
         .avif source (warn — image-optimization plugin emits both). \
         Files larger than `image_budget` raise a warn so authors \
         catch unoptimised originals."
    }

    fn run(&self, site: &Site, opts: &AuditOptions) -> Vec<Finding> {
        let mut findings = Vec::new();
        for path in &site.html_files {
            let Ok(html) = site.read(path) else { continue };
            let rel = site.rel(path);
            for img in extract_imgs(&html) {
                if img.alt.is_none() {
                    findings.push(
                        Finding::new(
                            NAME,
                            Severity::Error,
                            format!("<img src=\"{}\"> missing alt", img.src),
                        )
                        .with_code("IMG-ALT")
                        .with_path(rel.clone()),
                    );
                }
                if img.width.is_none() || img.height.is_none() {
                    findings.push(
                        Finding::new(
                            NAME,
                            Severity::Warn,
                            format!(
                                "<img src=\"{}\"> missing explicit width/height (CLS risk)",
                                img.src
                            ),
                        )
                        .with_code("IMG-DIMS")
                        .with_path(rel.clone()),
                    );
                }
                if img.src.starts_with("http://")
                    || img.src.starts_with("https://")
                    || img.src.starts_with("//")
                    || img.src.starts_with("data:")
                {
                    continue;
                }
                let candidate = if let Some(s) = img.src.strip_prefix('/') {
                    site.root.join(s)
                } else if let Some(parent) = path.parent() {
                    parent.join(&img.src)
                } else {
                    site.root.join(&img.src)
                };
                if let Ok(meta) = std::fs::metadata(&candidate) {
                    if meta.len() as usize > opts.image_budget {
                        findings.push(
                            Finding::new(
                                NAME,
                                Severity::Warn,
                                format!(
                                    "{} weighs {} bytes (budget {})",
                                    img.src,
                                    meta.len(),
                                    opts.image_budget
                                ),
                            )
                            .with_code("IMG-OVER-BUDGET")
                            .with_path(rel.clone()),
                        );
                    }
                    // Check for sibling .webp / .avif
                    let stem = candidate.with_extension("");
                    let has_webp = stem.with_extension("webp").exists();
                    let has_avif = stem.with_extension("avif").exists();
                    if !has_webp && !has_avif {
                        findings.push(
                            Finding::new(
                                NAME,
                                Severity::Warn,
                                format!(
                                    "{} has no sibling .webp or .avif source",
                                    img.src
                                ),
                            )
                            .with_code("IMG-NO-MODERN")
                            .with_path(rel.clone()),
                        );
                    }
                }
            }
        }
        findings
    }
}

struct ImgRef {
    src: String,
    alt: Option<String>,
    width: Option<String>,
    height: Option<String>,
}

fn extract_imgs(html: &str) -> Vec<ImgRef> {
    let mut out = Vec::new();
    let lower = html.to_lowercase();
    let mut cursor = 0;
    while let Some(rel) = lower[cursor..].find("<img") {
        let abs = cursor + rel;
        let end = lower[abs..].find('>').map_or(lower.len(), |e| abs + e + 1);
        let tag = &html[abs..end];
        cursor = end;
        let src = hreflang_attr(tag, "src").unwrap_or_default();
        let alt = hreflang_attr(tag, "alt");
        let width = hreflang_attr(tag, "width");
        let height = hreflang_attr(tag, "height");
        out.push(ImgRef {
            src,
            alt,
            width,
            height,
        });
    }
    out
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    fn site_with(html: &str, image_bytes: usize) -> Site {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().to_path_buf();
        let p = root.join("page.html");
        std::fs::write(&p, html).unwrap();
        std::fs::write(root.join("a.jpg"), vec![0u8; image_bytes]).unwrap();
        std::fs::write(root.join("a.webp"), vec![0u8; 10]).unwrap();
        std::mem::forget(tmp);
        Site {
            root,
            html_files: vec![p],
        }
    }

    #[test]
    fn passing_image_is_clean() {
        let html = r#"<html><body><img src="a.jpg" alt="a" width="10" height="10"></body></html>"#;
        let f = ImagesGate.run(&site_with(html, 10), &AuditOptions::default());
        assert!(f.is_empty(), "got {f:?}");
    }

    #[test]
    fn missing_alt_flags_error() {
        let html = r#"<html><body><img src="a.jpg" width="10" height="10"></body></html>"#;
        let f = ImagesGate.run(&site_with(html, 10), &AuditOptions::default());
        assert!(f.iter().any(|x| x.code.as_deref() == Some("IMG-ALT")));
    }

    #[test]
    fn over_budget_image_flagged() {
        let html = r#"<html><body><img src="a.jpg" alt="a" width="10" height="10"></body></html>"#;
        let f = ImagesGate.run(
            &site_with(html, 5000),
            &AuditOptions {
                image_budget: 100,
                ..AuditOptions::default()
            },
        );
        assert!(f
            .iter()
            .any(|x| x.code.as_deref() == Some("IMG-OVER-BUDGET")));
    }

    #[test]
    fn missing_width_height_warns_with_dims_code() {
        let html = r#"<html><body><img src="a.jpg" alt="a"></body></html>"#;
        let f = ImagesGate.run(&site_with(html, 10), &AuditOptions::default());
        let dims = f
            .iter()
            .find(|x| x.code.as_deref() == Some("IMG-DIMS"))
            .expect("dims finding");
        assert!(matches!(dims.severity, Severity::Warn));
    }

    #[test]
    fn missing_only_height_still_flags_dims() {
        let html =
            r#"<html><body><img src="a.jpg" alt="a" width="10"></body></html>"#;
        let f = ImagesGate.run(&site_with(html, 10), &AuditOptions::default());
        assert!(f.iter().any(|x| x.code.as_deref() == Some("IMG-DIMS")));
    }

    #[test]
    fn no_modern_sibling_warns() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().to_path_buf();
        std::fs::write(root.join("plain.png"), vec![0u8; 10]).unwrap();
        let html_path = root.join("page.html");
        std::fs::write(
            &html_path,
            r#"<html><body><img src="plain.png" alt="p" width="1" height="1"></body></html>"#,
        )
        .unwrap();
        std::mem::forget(tmp);
        let s = Site {
            root,
            html_files: vec![html_path],
        };
        let f = ImagesGate.run(&s, &AuditOptions::default());
        assert!(f.iter().any(|x| x.code.as_deref() == Some("IMG-NO-MODERN")));
    }

    #[test]
    fn avif_sibling_suppresses_no_modern() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().to_path_buf();
        std::fs::write(root.join("hero.png"), vec![0u8; 10]).unwrap();
        std::fs::write(root.join("hero.avif"), vec![0u8; 5]).unwrap();
        let html_path = root.join("page.html");
        std::fs::write(
            &html_path,
            r#"<html><body><img src="hero.png" alt="h" width="1" height="1"></body></html>"#,
        )
        .unwrap();
        std::mem::forget(tmp);
        let s = Site {
            root,
            html_files: vec![html_path],
        };
        let f = ImagesGate.run(&s, &AuditOptions::default());
        assert!(f.iter().all(|x| x.code.as_deref() != Some("IMG-NO-MODERN")));
    }

    #[test]
    fn external_image_src_is_skipped() {
        let html = r#"<html><body><img src="https://cdn.example/a.jpg" alt="x" width="1" height="1"></body></html>"#;
        let f = ImagesGate.run(&site_with(html, 10), &AuditOptions::default());
        assert!(
            f.iter()
                .all(|x| x.code.as_deref() != Some("IMG-OVER-BUDGET")
                    && x.code.as_deref() != Some("IMG-NO-MODERN")),
            "external imgs should not be probed; got {f:?}"
        );
    }

    #[test]
    fn data_uri_image_src_is_skipped() {
        let html = r#"<html><body><img src="data:image/png;base64,iVBOR" alt="x" width="1" height="1"></body></html>"#;
        let f = ImagesGate.run(&site_with(html, 10), &AuditOptions::default());
        assert!(f
            .iter()
            .all(|x| x.code.as_deref() != Some("IMG-OVER-BUDGET")));
    }

    #[test]
    fn protocol_relative_image_src_is_skipped() {
        let html = r#"<html><body><img src="//cdn.example/a.jpg" alt="x" width="1" height="1"></body></html>"#;
        let f = ImagesGate.run(&site_with(html, 10), &AuditOptions::default());
        assert!(f.iter().all(|x| x.code.as_deref() != Some("IMG-NO-MODERN")));
    }

    #[test]
    fn unreadable_html_skipped_no_panic() {
        let tmp = tempfile::tempdir().unwrap();
        let bogus = tmp.path().join("ghost.html");
        let s = Site {
            root: tmp.path().to_path_buf(),
            html_files: vec![bogus],
        };
        std::mem::forget(tmp);
        let f = ImagesGate.run(&s, &AuditOptions::default());
        assert!(f.is_empty());
    }

    #[test]
    fn metadata_methods_exposed() {
        let g = ImagesGate;
        assert_eq!(g.name(), "images");
        assert!(g.explain().contains("alt"));
        let _copy: ImagesGate = g;
        let _clone = g;
        assert!(format!("{g:?}").contains("ImagesGate"));
    }
}

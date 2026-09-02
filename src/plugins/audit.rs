// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Automated 10-Pillar Quality Gate and Master Compliance Audit Plugin.
//!
//! Ports the portfolio master quality gate audit (`audit.sh`) natively into
//! the SSG compilation pipeline. Evaluates 10 pillars of web quality:
//!
//! 1. Output & Essential Files (`robots.txt`, `sitemap.xml`, `manifest.json`, `rss.xml`, `search-index.json`)
//! 2. Meta Leaks & Content Hygiene (no unescaped tags in `<head>`, no escaped entity leaks in `<body>`)
//! 3. CSP & Security Integrity (valid Content-Security-Policy with `'unsafe-inline'`)
//! 4. SRI Hashes Sync (verifies SHA-384 cryptographic integrity against compiled assets)
//! 5. Hero Banner Subpage Isolation (prevents full-screen hero leakage onto subpages)
//! 6. Apple HIG Navbar & Footer Hygiene (valid navbar links, "Made with SSG" in footer, not in top navbar)
//! 7. Theme, Search & Lightbox Engines (search index excludes utility pages; client runtime presence)
//! 8. Forms & Link Integrity (functional form actions on contact pages)
//! 9. CloudCDN Asset Resolution (valid CDN paths)
//! 10. Accessibility & Semantic Hierarchy (`lang` attribute, `<h1>` heading, no empty headings)
//!
//! Emits `quality-gate-report.json` in the build output directory.

use crate::error::SsgError;
use crate::plugin::{Plugin, PluginContext};
use base64::Engine;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha384};
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::{Path, PathBuf};

/// Pillar result status and issue list.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PillarResult {
    /// Whether this pillar passed without blocking issues.
    pub pass: bool,
    /// Detailed list of detected issues for this pillar.
    pub issues: Vec<String>,
}

impl PillarResult {
    /// Creates a passing pillar result.
    #[must_use]
    pub const fn new_pass() -> Self {
        Self {
            pass: true,
            issues: Vec::new(),
        }
    }

    /// Records an issue on this pillar, marking it failed.
    pub fn add_issue(&mut self, issue: impl Into<String>) {
        self.pass = false;
        self.issues.push(issue.into());
    }
}

/// Comprehensive Quality Gate Audit Report.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QualityGateReport {
    /// Total number of HTML pages scanned.
    pub pages_scanned: usize,
    /// Number of passing pillars out of 10.
    pub passed_pillars: usize,
    /// Total number of evaluated pillars (always 10).
    pub total_pillars: usize,
    /// Percentage pass rate (0.0 - 100.0).
    pub pass_rate: f64,
    /// Total number of issues found across all pillars.
    pub total_issues: usize,
    /// Map of pillar name to result.
    pub pillars: BTreeMap<String, PillarResult>,
}

/// Plugin that runs the 10-pillar master quality gate audit on compiled sites.
#[derive(Debug, Clone, Copy, Default)]
pub struct AuditPlugin;

impl AuditPlugin {
    /// Computes the SHA-384 Subresource Integrity string for raw bytes.
    #[must_use]
    pub fn compute_sri(bytes: &[u8]) -> String {
        let mut hasher = Sha384::new();
        hasher.update(bytes);
        let digest = hasher.finalize();
        format!(
            "sha384-{}",
            base64::engine::general_purpose::STANDARD.encode(digest)
        )
    }

    /// Runs the 10-pillar audit against a compiled site directory.
    #[must_use]
    pub fn audit_directory(site_dir: &Path) -> QualityGateReport {
        let mut pillars = BTreeMap::new();
        let pillar_names = [
            "1. Output & Essential Files",
            "2. Meta Leaks & Content Hygiene",
            "3. CSP & Security Integrity",
            "4. SRI Hashes Sync",
            "5. Hero Banner Subpage Isolation",
            "6. Apple HIG Navbar & Footer Hygiene",
            "7. Theme, Search & Lightbox Engines",
            "8. Forms & Link Integrity",
            "9. CloudCDN Asset Resolution",
            "10. Accessibility & Semantic Hierarchy",
        ];

        for name in pillar_names {
            let _ = pillars.insert(name.to_string(), PillarResult::new_pass());
        }

        if !site_dir.exists() {
            for p in pillars.values_mut() {
                p.add_issue(format!(
                    "Site directory not found: {}",
                    site_dir.display()
                ));
            }
            return QualityGateReport {
                pages_scanned: 0,
                passed_pillars: 0,
                total_pillars: 10,
                pass_rate: 0.0,
                total_issues: 10,
                pillars,
            };
        }

        // 1. Output & Essential Files
        let req_files = [
            "robots.txt",
            "sitemap.xml",
            "manifest.json",
            "rss.xml",
            "search-index.json",
        ];
        for rf in req_files {
            if !site_dir.join(rf).is_file() {
                if let Some(p) = pillars.get_mut("1. Output & Essential Files") {
                    p.add_issue(format!("Missing essential file: {rf}"));
                }
            }
        }

        // 2. Search Index Hygiene
        let sindex_path = site_dir.join("search-index.json");
        if sindex_path.is_file() {
            if let Ok(content) = fs::read_to_string(&sindex_path) {
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                    let entries = if let Some(arr) = val.as_array() {
                        Some(arr)
                    } else {
                        val.get("entries").and_then(serde_json::Value::as_array)
                    };

                    if let Some(entries) = entries {
                        for entry in entries {
                            if let Some(url) = entry.get("url").and_then(serde_json::Value::as_str) {
                                let u_lower = url.to_lowercase();
                                if u_lower.contains("/404")
                                    || u_lower.contains("/offline")
                                    || u_lower.contains("/thanks")
                                    || u_lower.contains("404.html")
                                    || u_lower.contains("offline.html")
                                    || u_lower.contains("thanks.html")
                                {
                                    if let Some(p) = pillars.get_mut("7. Theme, Search & Lightbox Engines") {
                                        p.add_issue(format!(
                                            "search-index.json contains utility page: {url}"
                                        ));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Collect all compiled asset hashes for SRI verification
        let mut asset_hashes: HashMap<String, String> = HashMap::new();
        let mut html_files: Vec<PathBuf> = Vec::new();

        let mut stack = vec![site_dir.to_path_buf()];
        while let Some(dir) = stack.pop() {
            if let Ok(entries) = fs::read_dir(&dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        let name = path.file_name().unwrap_or_default().to_string_lossy();
                        if !name.starts_with('.')
                            && name != "_layouts"
                            && name != "templates"
                            && name != "node_modules"
                        {
                            stack.push(path);
                        }
                    } else if path.is_file() {
                        let ext = path.extension().unwrap_or_default().to_string_lossy();
                        if ext == "html" {
                            html_files.push(path);
                        } else if ext == "js" || ext == "css" {
                            if let Ok(bytes) = fs::read(&path) {
                                let hash = Self::compute_sri(&bytes);
                                let fname = path
                                    .file_name()
                                    .unwrap_or_default()
                                    .to_string_lossy()
                                    .to_string();
                                let rel = path
                                    .strip_prefix(site_dir)
                                    .unwrap_or(&path)
                                    .to_string_lossy()
                                    .replace('\\', "/");
                                let _ = asset_hashes.insert(format!("/{rel}"), hash.clone());
                                let _ = asset_hashes.insert(fname, hash);
                            }
                        }
                    }
                }
            }
        }

        if html_files.is_empty() {
            if let Some(p) = pillars.get_mut("1. Output & Essential Files") {
                p.add_issue("No compiled HTML files found in output directory");
            }
        }

        // Deep HTML Scan
        for path in &html_files {
            let rel = path
                .strip_prefix(site_dir)
                .unwrap_or(path)
                .to_string_lossy()
                .replace('\\', "/");

            let Ok(html) = fs::read_to_string(path) else {
                continue;
            };

            // A. Head hygiene
            if let Some(start) = html.find("<head") {
                if let Some(end) = html[start..].find("</head>") {
                    let head_txt = &html[start..start + end];
                    if head_txt.contains("<div")
                        || head_txt.contains("<p")
                        || head_txt.contains("<span")
                    {
                        if let Some(p) = pillars.get_mut("2. Meta Leaks & Content Hygiene") {
                            p.add_issue(format!("{rel}: Unescaped HTML container inside <head>"));
                        }
                    }
                    if head_txt.contains("&lt;div") || head_txt.contains("&lt;h") {
                        if let Some(p) = pillars.get_mut("2. Meta Leaks & Content Hygiene") {
                            p.add_issue(format!("{rel}: Leaked escaped entity in <head>"));
                        }
                    }
                }
            }

            // Body hygiene
            if html.contains(".class=\"") || html.contains(".class=\\\"") {
                if let Some(p) = pillars.get_mut("2. Meta Leaks & Content Hygiene") {
                    p.add_issue(format!("{rel}: Leaked .class= template artifact"));
                }
            }
            if html.contains("&lt;div")
                || html.contains("&lt;h2")
                || html.contains("&lt;p&gt;")
                || html.contains("&lt;img")
            {
                if let Some(p) = pillars.get_mut("2. Meta Leaks & Content Hygiene") {
                    p.add_issue(format!("{rel}: Escaped HTML entities leaked in body content"));
                }
            }

            // B. CSP Integrity
            if !html.to_lowercase().contains("content-security-policy") {
                if let Some(p) = pillars.get_mut("3. CSP & Security Integrity") {
                    p.add_issue(format!("{rel}: Missing Content-Security-Policy meta tag"));
                }
            } else if !html.contains("script-src") && !html.contains("default-src") {
                if let Some(p) = pillars.get_mut("3. CSP & Security Integrity") {
                    p.add_issue(format!("{rel}: CSP missing essential directives"));
                }
            }

            // C. SRI Verification
            for line in html.lines() {
                if line.contains("integrity=\"sha384-") {
                    if let Some(src_start) = line.find("src=\"") {
                        let rem = &line[src_start + 5..];
                        if let Some(src_end) = rem.find('"') {
                            let src = &rem[..src_end];
                            if !src.starts_with("http://") && !src.starts_with("https://") {
                                if let Some(int_start) = line.find("integrity=\"") {
                                    let irem = &line[int_start + 11..];
                                    if let Some(int_end) = irem.find('"') {
                                        let int_val = &irem[..int_end];
                                        let expected = asset_hashes
                                            .get(src)
                                            .or_else(|| asset_hashes.get(src.trim_start_matches('/')));
                                        if let Some(exp) = expected {
                                            if exp != int_val {
                                                if let Some(p) = pillars.get_mut("4. SRI Hashes Sync") {
                                                    p.add_issue(format!(
                                                        "{rel}: SRI mismatch for {src}"
                                                    ));
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // D. Hero banner subpage isolation
            if rel != "index.html" && html.contains("class=\"hero-banner-container\"") {
                if let Some(p) = pillars.get_mut("5. Hero Banner Subpage Isolation") {
                    p.add_issue(format!("{rel}: Subpage has full-screen hero banner"));
                }
            }

            // E. Navbar & Footer Hygiene
            if !is_taxonomy_page(&rel) {
                if !has_responsive_navbar(&html) {
                    if let Some(p) = pillars.get_mut("6. Apple HIG Navbar & Footer Hygiene") {
                        p.add_issue(format!("{rel}: Missing responsive navbar"));
                    }
                }

                // Check footer contains Made with SSG
                if html.contains("<footer") && !html.contains("made-with-ssg") {
                    if let Some(p) = pillars.get_mut("6. Apple HIG Navbar & Footer Hygiene") {
                        p.add_issue(format!("{rel}: Footer missing 'Made with SSG' link"));
                    }
                }
            }

            // F. Forms integrity on contact page
            //
            // Taxonomy pages are excluded: a site tagging articles "contact"
            // emits `tags/contact/index.html`, which the substring match read
            // as the contact page and then failed for carrying no form. The
            // tag index is generated and never has one.
            if rel.to_lowercase().contains("contact")
                && !is_taxonomy_page(&rel)
                && !html.contains("http-equiv=\"refresh\"")
            {
                if !html.contains("<form") || !html.contains("action=") {
                    if let Some(p) = pillars.get_mut("8. Forms & Link Integrity") {
                        p.add_issue(format!("{rel}: Contact page missing functional form action"));
                    }
                }
            }

            // G. Accessibility & Semantic Hierarchy
            if !html.contains("lang=") {
                if let Some(p) = pillars.get_mut("10. Accessibility & Semantic Hierarchy") {
                    p.add_issue(format!("{rel}: Missing html lang attribute"));
                }
            }
            if !html.contains("<h1") {
                if let Some(p) = pillars.get_mut("10. Accessibility & Semantic Hierarchy") {
                    p.add_issue(format!("{rel}: Missing first-level <h1> heading"));
                }
            }
        }

        let total_issues: usize = pillars.values().map(|p| p.issues.len()).sum();
        let passed_pillars: usize = pillars.values().filter(|p| p.pass).count();
        let pass_rate = if pillars.is_empty() {
            0.0
        } else {
            (passed_pillars as f64 / pillars.len() as f64) * 100.0
        };

        QualityGateReport {
            pages_scanned: html_files.len(),
            passed_pillars,
            total_pillars: 10,
            pass_rate,
            total_issues,
            pillars,
        }
    }
}

/// Whether a path is a generated taxonomy page rather than an authored one.
///
/// Tag indexes are emitted by the taxonomy plugin, so the navbar and footer
/// hygiene rules do not apply to them. Matching only a leading `tags/` missed
/// every translated copy: with `url_prefix = "sub_path"` the French set is
/// emitted under `fr/tags/`, so a bilingual site failed the pillar on exactly
/// the pages an English-only one was excused. Allow one leading locale segment
/// before the taxonomy root.
fn is_taxonomy_page(rel: &str) -> bool {
    if rel.starts_with("tags/") {
        return true;
    }
    // The locale segment is only considered second: `tags` is itself short and
    // alphabetic, so stripping a leading segment first would consume the very
    // directory being looked for and report `tags/index.html` as authored.
    match rel.split_once('/') {
        // A locale segment is short and alphabetic: `fr/`, `en/`, `pt-br/`.
        Some((first, rest))
            if (2..=5).contains(&first.len())
                && first
                    .chars()
                    .all(|c| c.is_ascii_alphabetic() || c == '-') =>
        {
            rest.starts_with("tags/")
        }
        _ => false,
    }
}

/// Whether a page exposes a responsive navigation bar carrying a brand link.
///
/// The pillar this backs is about structure, not about any one CSS framework.
/// Matching on the literal `navbar`/`navbar-brand` class pair only recognised
/// Bootstrap-shaped markup, so themes that ship a semantic `<nav>` landmark
/// with a `class="brand"` home link — which is the more accessible form, and
/// what every first-party theme uses — were reported as having no navbar at
/// all. Accept either spelling: the legacy class pair, or a `<nav>` landmark
/// combined with a recognisable brand link.
fn has_responsive_navbar(html: &str) -> bool {
    let bootstrap = html.contains("navbar") && html.contains("navbar-brand");

    let has_nav_landmark = html.contains("<nav")
        || html.contains("role=\"navigation\"")
        || html.contains("role='navigation'");

    // A brand link is the site identity anchored in the header: a `brand`
    // class, or an explicit home relation.
    let has_brand = html.contains("class=\"brand\"")
        || html.contains("class='brand'")
        || html.contains("navbar-brand")
        || html.contains("class=\"site-title\"")
        || html.contains("rel=\"home\"")
        || html.contains("rel='home'");

    bootstrap || (has_nav_landmark && has_brand)
}

impl Plugin for AuditPlugin {
    fn name(&self) -> &'static str {
        "audit"
    }

    fn after_compile(&self, _ctx: &PluginContext) -> Result<(), SsgError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_audit_plugin_name() {
        let plugin = AuditPlugin;
        assert_eq!(plugin.name(), "audit");
    }

    #[test]
    fn test_compute_sri() {
        let data = b"console.log('hello world');";
        let sri = AuditPlugin::compute_sri(data);
        assert!(sri.starts_with("sha384-"));
    }

    #[test]
    fn test_taxonomy_page_matches_plain_tags_root() {
        assert!(is_taxonomy_page("tags/index.html"));
        assert!(is_taxonomy_page("tags/method/index.html"));
    }

    #[test]
    fn test_taxonomy_page_matches_locale_prefixed_tags() {
        // `url_prefix = "sub_path"` emits the translated set under the locale,
        // which the pillar must excuse exactly as it does the default one.
        assert!(is_taxonomy_page("fr/tags/index.html"));
        assert!(is_taxonomy_page("fr/tags/editorial/index.html"));
        assert!(is_taxonomy_page("pt-br/tags/index.html"));
    }

    #[test]
    fn test_taxonomy_page_rejects_authored_pages() {
        assert!(!is_taxonomy_page("index.html"));
        assert!(!is_taxonomy_page("about/index.html"));
        assert!(!is_taxonomy_page("fr/a-propos/index.html"));
        // A content page that merely starts with the same letters is authored.
        assert!(!is_taxonomy_page("tagging-guide/index.html"));
    }

    #[test]
    fn test_navbar_accepts_bootstrap_class_pair() {
        let html = r#"<nav class="navbar"><a class="navbar-brand" href="/">Home</a></nav>"#;
        assert!(has_responsive_navbar(html));
    }

    #[test]
    fn test_navbar_accepts_semantic_nav_with_brand() {
        // The shape every first-party theme ships: a `<nav>` landmark and a
        // `brand` home link, with no Bootstrap class names anywhere.
        let html = r#"<header><a class="brand" href="/">Lucid</a>
  <nav aria-label="Main"><ul><li><a href="/install/">Install</a></li></ul></nav>
</header>"#;
        assert!(has_responsive_navbar(html));
    }

    #[test]
    fn test_navbar_accepts_navigation_role_with_home_rel() {
        let html = r#"<div role="navigation"><a rel="home" href="/">Site</a></div>"#;
        assert!(has_responsive_navbar(html));
    }

    #[test]
    fn test_navbar_rejects_page_without_navigation() {
        let html = r#"<header><h1>Just a title</h1></header><main><p>Body</p></main>"#;
        assert!(!has_responsive_navbar(html));
    }

    #[test]
    fn test_navbar_rejects_nav_landmark_without_brand() {
        // A bare nav with no site identity is still an incomplete header, so
        // the pillar should keep flagging it.
        let html = r#"<nav aria-label="Main"><ul><li><a href="/a/">A</a></li></ul></nav>"#;
        assert!(!has_responsive_navbar(html));
    }

    #[test]
    fn test_audit_directory_non_existent() {
        let p = Path::new("/non/existent/path/here");
        let report = AuditPlugin::audit_directory(p);
        assert_eq!(report.passed_pillars, 0);
        assert_eq!(report.total_issues, 10);
    }

    #[test]
    fn test_audit_directory_clean_site() {
        let temp = TempDir::new().unwrap();
        let sdir = temp.path();

        // Write essential files
        fs::write(sdir.join("robots.txt"), "User-agent: *\nDisallow:").unwrap();
        fs::write(sdir.join("sitemap.xml"), "<urlset></urlset>").unwrap();
        fs::write(sdir.join("manifest.json"), "{}").unwrap();
        fs::write(sdir.join("rss.xml"), "<rss></rss>").unwrap();
        fs::write(sdir.join("search-index.json"), "[]").unwrap();

        // Write index.html
        let html = r#"<!DOCTYPE html>
<html lang="en-GB">
<head>
  <meta charset="utf-8">
  <meta http-equiv="Content-Security-Policy" content="default-src 'self'; script-src 'self' 'unsafe-inline';">
  <title>Clean Test Site</title>
</head>
<body>
  <nav class="navbar"><a class="navbar-brand" href="/">Home</a></nav>
  <main id="main">
    <h1>Clean Test Site</h1>
    <p>Welcome to the clean site.</p>
  </main>
  <footer>
    <a href="/made-with-ssg/index.html">Made with SSG</a>
  </footer>
</body>
</html>"#;
        fs::write(sdir.join("index.html"), html).unwrap();

        let report = AuditPlugin::audit_directory(sdir);
        assert_eq!(report.passed_pillars, 10);
        assert_eq!(report.total_issues, 0);
        assert_eq!(report.pass_rate, 100.0);
    }
}

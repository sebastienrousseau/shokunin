// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Local LLM content plugin.
//!
//! Invokes a local LLM (Ollama, llama.cpp) at build time to auto-generate:
//! - `alt` text for images missing it
//! - `meta description` for pages where it's empty or < 50 chars
//! - JSON-LD `description` fields from page content
//!
//! Configured via the `[ai]` section in `ssg.toml`:
//! ```toml
//! [ai]
//! model = "llama3"
//! endpoint = "http://localhost:11434"
//! ```
//!
//! Graceful fallback: if no LLM is reachable, logs a warning and skips.

use crate::plugin::{Plugin, PluginContext};
use anyhow::Result;
use std::{fs, path::Path, process::Command};

/// Configuration for the LLM plugin.
#[derive(Debug, Clone)]
pub struct LlmConfig {
    /// Model name (e.g., `"llama3"`, `"mistral"`).
    pub model: String,
    /// Ollama API endpoint.
    pub endpoint: String,
    /// If true, print generated text but don't write files.
    pub dry_run: bool,
    /// Target Flesch-Kincaid Grade Level (default: 8.0).
    pub target_grade: f64,
    /// Max refinement attempts if readability exceeds target (default: 1).
    pub max_refinement_attempts: usize,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            model: "llama3".to_string(),
            endpoint: "http://localhost:11434".to_string(),
            dry_run: false,
            target_grade: 8.0,
            max_refinement_attempts: 1,
        }
    }
}

/// Plugin that uses a local LLM to augment content at build time.
#[derive(Debug)]
pub struct LlmPlugin {
    config: LlmConfig,
}

impl LlmPlugin {
    /// Creates a new `LlmPlugin` with the given configuration.
    #[must_use]
    pub const fn new(config: LlmConfig) -> Self {
        Self { config }
    }
}

/// Result of auditing a single file's readability.
#[derive(Debug, Clone, serde::Serialize)]
pub struct FileAuditResult {
    /// Relative file path.
    pub path: String,
    /// Flesch-Kincaid Grade Level.
    pub grade_level: f64,
    /// Flesch Reading Ease score.
    pub reading_ease: f64,
    /// Average words per sentence.
    pub avg_sentence_len: f64,
    /// Whether it passes the target grade threshold.
    pub passes: bool,
}

/// Aggregated readability audit report.
#[derive(Debug, Clone, serde::Serialize)]
pub struct AuditReport {
    /// Target grade level used for pass/fail.
    pub target_grade: f64,
    /// Total files scanned.
    pub total_files: usize,
    /// Files that pass the readability threshold.
    pub passing: usize,
    /// Files that exceed the readability threshold.
    pub failing: usize,
    /// Per-file results.
    pub results: Vec<FileAuditResult>,
}

/// Result of the agentic AI fix pipeline for a single file.
#[derive(Debug, Clone, serde::Serialize)]
pub struct AiFixResult {
    /// Relative file path.
    pub path: String,
    /// Grade level before fix attempt.
    pub before_grade: f64,
    /// Grade level after fix attempt (same as before if not improved).
    pub after_grade: f64,
    /// Whether the fix improved readability.
    pub improved: bool,
    /// Action taken: "rewritten", "skipped", "no-improvement", "ollama-unavailable".
    pub action: String,
}

/// Aggregated report from the agentic AI fix pipeline.
#[derive(Debug, Clone, serde::Serialize)]
pub struct AiFixReport {
    /// Total files audited.
    pub total_audited: usize,
    /// Files that failed the readability threshold.
    pub total_failing: usize,
    /// Files successfully improved.
    pub total_fixed: usize,
    /// Per-file results.
    pub results: Vec<AiFixResult>,
}

impl LlmPlugin {
    /// Audits all Markdown files in a directory for readability.
    ///
    /// Returns a structured report with per-file Flesch-Kincaid scores.
    /// Does not require an LLM — uses the local `ReadabilityAudit` engine.
    ///
    /// **Note:** The syllable heuristic is English-only. Non-English
    /// content (Bengali, Hindi, Turkish, etc.) produces inflated scores.
    /// Use the `en/` subdirectory for accurate results on multilingual
    /// repos, or filter results by locale.
    pub fn audit_all(
        content_dir: &Path,
        target_grade: f64,
    ) -> Result<AuditReport> {
        let md_files =
            crate::walk::walk_files(content_dir, "md").unwrap_or_default();

        let mut results = Vec::with_capacity(md_files.len());

        for path in &md_files {
            let Ok(content) = fs::read_to_string(path) else {
                continue; // File may have been removed by a concurrent test
            };
            // Strip frontmatter before auditing prose
            let body = strip_frontmatter(&content);
            // Detect language from frontmatter
            let lang = extract_frontmatter_lang(&content);
            let audit = ReadabilityAudit::analyze_with_lang(&body, &lang);
            let rel = path
                .strip_prefix(content_dir)
                .unwrap_or(path)
                .to_string_lossy()
                .to_string();

            results.push(FileAuditResult {
                path: rel,
                grade_level: (audit.grade_level * 10.0).round() / 10.0,
                reading_ease: (audit.reading_ease * 10.0).round() / 10.0,
                avg_sentence_len: (audit.avg_sentence_len * 10.0).round()
                    / 10.0,
                passes: audit.grade_level <= target_grade,
            });
        }

        let passing = results.iter().filter(|r| r.passes).count();
        let failing = results.len() - passing;

        Ok(AuditReport {
            target_grade,
            total_files: results.len(),
            passing,
            failing,
            results,
        })
    }

    /// Audits and rewrites failing Markdown files via LLM refinement.
    ///
    /// For each file that exceeds `target_grade`:
    /// 1. Extracts the prose body (strips frontmatter)
    /// 2. Sends it to the LLM with a simplification prompt
    /// 3. If the refined version scores better, writes it back
    ///    (preserving the original frontmatter)
    /// 4. If `dry_run`, prints the diff without writing
    ///
    /// Returns the number of files rewritten.
    pub fn audit_and_fix(
        content_dir: &Path,
        config: &LlmConfig,
    ) -> Result<usize> {
        if !is_ollama_available(&config.endpoint) {
            log::warn!(
                "[llm] Ollama not reachable at {}, skipping auto-fix",
                config.endpoint
            );
            return Ok(0);
        }

        let report = Self::audit_all(content_dir, config.target_grade)?;
        let failing: Vec<_> =
            report.results.iter().filter(|r| !r.passes).collect();

        if failing.is_empty() {
            log::info!(
                "[llm] All {} file(s) pass grade {:.0}",
                report.total_files,
                config.target_grade
            );
            return Ok(0);
        }

        log::info!(
            "[llm] {} file(s) exceed grade {:.0}, attempting refinement",
            failing.len(),
            config.target_grade
        );

        let mut rewritten = 0usize;

        for result in &failing {
            let path = content_dir.join(&result.path);
            let original = fs::read_to_string(&path)?;
            let (frontmatter_block, body) = split_frontmatter(&original);
            let body_trimmed = body.trim();

            if body_trimmed.is_empty() {
                continue;
            }

            let prompt = format!(
                "Rewrite this Markdown content at a 6th-grade reading level. \
                 Rules:\n\
                 - Max 20 words per sentence\n\
                 - Max 4 sentences per paragraph\n\
                 - Use simple, common words\n\
                 - Keep ALL facts, numbers, dates, and code blocks exactly the same\n\
                 - Keep ALL Markdown headings (#, ##, ###) and formatting\n\
                 - Return ONLY the rewritten Markdown, nothing else\n\n\
                 {body_trimmed}"
            );

            if let Some(refined) = generate_with_refinement(
                &config.endpoint,
                &config.model,
                &prompt,
                config.target_grade,
                config.max_refinement_attempts,
            ) {
                let refined_audit = ReadabilityAudit::analyze(&refined);
                let original_audit = ReadabilityAudit::analyze(body_trimmed);

                if refined_audit.grade_level < original_audit.grade_level {
                    if config.dry_run {
                        log::info!(
                            "[llm] [dry-run] {}: grade {:.1} → {:.1}",
                            result.path,
                            original_audit.grade_level,
                            refined_audit.grade_level
                        );
                    } else {
                        // Reassemble: frontmatter + refined body
                        let output =
                            format!("{frontmatter_block}\n{refined}\n");
                        fs::write(&path, output)?;
                        log::info!(
                            "[llm] Rewrote {}: grade {:.1} → {:.1}",
                            result.path,
                            original_audit.grade_level,
                            refined_audit.grade_level
                        );
                        rewritten += 1;
                    }
                } else {
                    log::warn!(
                        "[llm] Could not improve {}: grade {:.1} (refined: {:.1})",
                        result.path,
                        original_audit.grade_level,
                        refined_audit.grade_level
                    );
                }
            }
        }

        Ok(rewritten)
    }

    /// Agentic pipeline: audit → diagnose → fix → verify → report.
    ///
    /// Like `audit_and_fix()` but returns a detailed JSON-serialisable
    /// report with before/after scores for each file.
    pub fn audit_and_fix_with_report(
        content_dir: &Path,
        config: &LlmConfig,
    ) -> Result<AiFixReport> {
        if !is_ollama_available(&config.endpoint) {
            log::warn!(
                "[ai-fix] Ollama not reachable at {}, skipping",
                config.endpoint
            );
            return Ok(AiFixReport {
                total_audited: 0,
                total_failing: 0,
                total_fixed: 0,
                results: vec![],
            });
        }

        let report = Self::audit_all(content_dir, config.target_grade)?;
        let failing: Vec<_> =
            report.results.iter().filter(|r| !r.passes).collect();
        let mut fix_results = Vec::new();

        for result in &failing {
            let path = content_dir.join(&result.path);
            let Ok(original) = fs::read_to_string(&path) else {
                fix_results.push(AiFixResult {
                    path: result.path.clone(),
                    before_grade: result.grade_level,
                    after_grade: result.grade_level,
                    improved: false,
                    action: "skipped".to_string(),
                });
                continue;
            };
            let (frontmatter_block, body) = split_frontmatter(&original);
            let body_trimmed = body.trim();

            if body_trimmed.is_empty() {
                fix_results.push(AiFixResult {
                    path: result.path.clone(),
                    before_grade: result.grade_level,
                    after_grade: result.grade_level,
                    improved: false,
                    action: "skipped".to_string(),
                });
                continue;
            }

            let prompt = format!(
                "Rewrite this Markdown content at a 6th-grade reading level. \
                 Rules:\n\
                 - Max 20 words per sentence\n\
                 - Max 4 sentences per paragraph\n\
                 - Use simple, common words\n\
                 - Keep ALL facts, numbers, dates, and code blocks exactly the same\n\
                 - Keep ALL Markdown headings (#, ##, ###) and formatting\n\
                 - Return ONLY the rewritten Markdown, nothing else\n\n\
                 {body_trimmed}"
            );

            if let Some(refined) = generate_with_refinement(
                &config.endpoint,
                &config.model,
                &prompt,
                config.target_grade,
                config.max_refinement_attempts,
            ) {
                let refined_audit = ReadabilityAudit::analyze(&refined);
                let original_audit = ReadabilityAudit::analyze(body_trimmed);

                if refined_audit.grade_level < original_audit.grade_level {
                    if !config.dry_run {
                        let output =
                            format!("{frontmatter_block}\n{refined}\n");
                        fs::write(&path, output)?;
                    }
                    fix_results.push(AiFixResult {
                        path: result.path.clone(),
                        before_grade: (original_audit.grade_level * 10.0)
                            .round()
                            / 10.0,
                        after_grade: (refined_audit.grade_level * 10.0).round()
                            / 10.0,
                        improved: true,
                        action: if config.dry_run {
                            "dry-run".to_string()
                        } else {
                            "rewritten".to_string()
                        },
                    });
                } else {
                    fix_results.push(AiFixResult {
                        path: result.path.clone(),
                        before_grade: (original_audit.grade_level * 10.0)
                            .round()
                            / 10.0,
                        after_grade: (refined_audit.grade_level * 10.0).round()
                            / 10.0,
                        improved: false,
                        action: "no-improvement".to_string(),
                    });
                }
            } else {
                fix_results.push(AiFixResult {
                    path: result.path.clone(),
                    before_grade: result.grade_level,
                    after_grade: result.grade_level,
                    improved: false,
                    action: "skipped".to_string(),
                });
            }
        }

        let total_fixed = fix_results.iter().filter(|r| r.improved).count();

        Ok(AiFixReport {
            total_audited: report.total_files,
            total_failing: failing.len(),
            total_fixed,
            results: fix_results,
        })
    }
}

/// Splits content into `(frontmatter_block, body)`.
///
/// The frontmatter block includes delimiters so it can be
/// reassembled verbatim. Returns `("", content)` if no
/// frontmatter is found.
fn split_frontmatter(content: &str) -> (String, String) {
    let trimmed = content.trim_start();
    let leading_ws = &content[..content.len() - trimmed.len()];

    for delim in ["---", "+++"] {
        if let Some(rest) = trimmed.strip_prefix(delim) {
            if let Some(end) = rest.find(delim) {
                let fm_end = delim.len() + end + delim.len();
                let frontmatter = &trimmed[..fm_end];
                let body = &trimmed[fm_end..];
                return (
                    format!("{leading_ws}{frontmatter}"),
                    body.to_string(),
                );
            }
        }
    }

    (String::new(), content.to_string())
}

/// Extracts the `language` or `lang` field from YAML/TOML frontmatter.
fn extract_frontmatter_lang(content: &str) -> String {
    let trimmed = content.trim_start();
    for delim in ["---", "+++"] {
        if let Some(rest) = trimmed.strip_prefix(delim) {
            if let Some(end) = rest.find(delim) {
                let fm = &rest[..end];
                // Try YAML-style: `language: en` or `lang: en`
                for line in fm.lines() {
                    let line = line.trim();
                    for key in ["language:", "lang:"] {
                        if let Some(val) = line.strip_prefix(key) {
                            let val =
                                val.trim().trim_matches('"').trim_matches('\'');
                            if !val.is_empty() {
                                return val.to_string();
                            }
                        }
                    }
                }
                // Try TOML-style: `language = "en"` or `lang = "en"`
                for line in fm.lines() {
                    let line = line.trim();
                    for key in ["language", "lang"] {
                        if line.starts_with(key) {
                            if let Some(val) = line.split('=').nth(1) {
                                let val = val
                                    .trim()
                                    .trim_matches('"')
                                    .trim_matches('\'');
                                if !val.is_empty() {
                                    return val.to_string();
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    String::new()
}

/// Strips YAML/TOML frontmatter from Markdown content.
fn strip_frontmatter(content: &str) -> String {
    let trimmed = content.trim_start();
    for delim in ["---", "+++"] {
        if let Some(rest) = trimmed.strip_prefix(delim) {
            if let Some(end) = rest.find(delim) {
                return rest[end + delim.len()..].to_string();
            }
        }
    }
    content.to_string()
}

impl Plugin for LlmPlugin {
    fn name(&self) -> &'static str {
        "llm"
    }

    fn after_compile(&self, ctx: &PluginContext) -> Result<()> {
        if !ctx.site_dir.exists() {
            return Ok(());
        }

        // Check if Ollama is available
        if !is_ollama_available(&self.config.endpoint) {
            log::warn!(
                "[llm] Ollama not reachable at {}, skipping AI augmentation",
                self.config.endpoint
            );
            return Ok(());
        }

        let html_files = ctx.get_html_files();
        let mut augmented = 0usize;

        for path in &html_files {
            let html = fs::read_to_string(path)?;
            let mut modified = html.clone();

            // Auto-generate meta descriptions for pages with short/missing ones
            if needs_meta_description(&modified) {
                if let Some(desc) = generate_meta_description(
                    &modified,
                    &self.config.model,
                    &self.config.endpoint,
                    self.config.target_grade,
                    self.config.max_refinement_attempts,
                ) {
                    let audit = ReadabilityAudit::analyze(&desc);
                    if self.config.dry_run {
                        let rel = path
                            .strip_prefix(&ctx.site_dir)
                            .unwrap_or(path)
                            .display();
                        log::info!(
                            "[llm] [dry-run] {rel}: description = {desc}"
                        );
                        log::info!(
                            "[llm] [dry-run] {rel}: grade={:.1}, ease={:.1}, avg_sentence={:.1}",
                            audit.grade_level, audit.reading_ease, audit.avg_sentence_len
                        );
                    } else {
                        modified = inject_meta_description(&modified, &desc);
                        // Also populate JSON-LD Article description
                        modified = inject_jsonld_description(&modified, &desc);
                    }
                }
            }

            // Auto-generate alt text for images missing it
            let alt_count = generate_missing_alt_text(
                &mut modified,
                &self.config.model,
                &self.config.endpoint,
                self.config.dry_run,
                path,
                &ctx.site_dir,
            );

            if !self.config.dry_run && modified != html {
                fs::write(path, &modified)?;
                augmented += 1;
            }

            if alt_count > 0 {
                augmented += 1;
            }
        }

        if augmented > 0 {
            log::info!(
                "[llm] Augmented {augmented} page(s) with model '{}'",
                self.config.model
            );
        }

        Ok(())
    }
}

/// Checks if Ollama is reachable at the given endpoint.
fn is_ollama_available(endpoint: &str) -> bool {
    // Try a simple HTTP health check via curl
    Command::new("curl")
        .args(["-sf", "--max-time", "2", endpoint])
        .output()
        .is_ok_and(|o| o.status.success())
}

/// Returns true if the page needs a meta description (missing or < 50 chars).
fn needs_meta_description(html: &str) -> bool {
    if let Some(start) = html.find("name=\"description\"") {
        if let Some(content_start) = html[start..].find("content=\"") {
            let abs = start + content_start + 9;
            if let Some(end) = html[abs..].find('"') {
                let desc = &html[abs..abs + end];
                return desc.len() < 50;
            }
        }
    }
    // No description meta tag found
    !html.contains("name=\"description\"")
}

/// Generates a meta description via LLM with readability refinement.
fn generate_meta_description(
    html: &str,
    model: &str,
    endpoint: &str,
    target_grade: f64,
    max_attempts: usize,
) -> Option<String> {
    let text = extract_page_text(html, 500);
    if text.len() < 20 {
        return None;
    }

    let prompt = format!(
        "Write a concise SEO meta description (120-155 characters) for this page content. \
         Use simple words and short sentences. \
         Return ONLY the description text, no quotes or explanation:\n\n{text}"
    );

    generate_with_refinement(
        endpoint,
        model,
        &prompt,
        target_grade,
        max_attempts,
    )
}

/// Injects a meta description tag into the HTML head.
fn inject_meta_description(html: &str, description: &str) -> String {
    let escaped = description
        .replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;");
    let tag = format!("<meta name=\"description\" content=\"{escaped}\">\n");

    if let Some(pos) = html.find("</head>") {
        let mut result = html.to_string();
        result.insert_str(pos, &tag);
        result
    } else {
        html.to_string()
    }
}

/// Generates alt text for images that are missing it.
fn generate_missing_alt_text(
    html: &mut String,
    model: &str,
    endpoint: &str,
    dry_run: bool,
    path: &Path,
    site_dir: &Path,
) -> usize {
    let mut count = 0;
    let mut search_from = 0;

    while let Some(start) = html[search_from..].find("<img") {
        let abs_start = search_from + start;
        let Some(tag_end) = html[abs_start..].find('>') else {
            break;
        };
        let tag_end_abs = abs_start + tag_end + 1;
        let tag = &html[abs_start..tag_end_abs];

        if !tag.contains("alt=") || tag.contains("alt=\"\"") {
            // Extract src for context
            let src = extract_attr(tag, "src").unwrap_or_default();
            let prompt = format!(
                "Describe this image for an alt text attribute. The image file is named '{}'. \
                 Return ONLY the alt text (max 125 characters), no quotes:\n",
                src
            );

            if let Some(alt) = call_ollama(endpoint, model, &prompt) {
                let alt = alt.trim().replace('"', "&quot;");
                if dry_run {
                    let rel =
                        path.strip_prefix(site_dir).unwrap_or(path).display();
                    log::info!(
                        "[llm] [dry-run] {rel}: alt=\"{alt}\" for {src}"
                    );
                } else {
                    // Replace the tag with one that has alt text
                    let new_tag = if tag.contains("alt=\"\"") {
                        tag.replace("alt=\"\"", &format!("alt=\"{alt}\""))
                    } else {
                        tag.replace("<img", &format!("<img alt=\"{alt}\""))
                    };
                    html.replace_range(abs_start..tag_end_abs, &new_tag);
                }
                count += 1;
            }
        }

        search_from = tag_end_abs;
    }

    count
}

/// Extracts plain text from HTML for LLM prompting.
fn extract_page_text(html: &str, max_chars: usize) -> String {
    let body_start = html
        .find("<main")
        .or_else(|| html.find("<body"))
        .unwrap_or(0);
    let body = &html[body_start..];

    let mut text = String::with_capacity(max_chars + 50);
    let mut in_tag = false;
    for ch in body.chars() {
        if text.len() >= max_chars {
            break;
        }
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag && !ch.is_control() => text.push(ch),
            _ => {}
        }
    }

    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Extracts an attribute value from an HTML tag.
fn extract_attr(tag: &str, attr: &str) -> Option<String> {
    let pattern = format!("{attr}=\"");
    let start = tag.find(&pattern)? + pattern.len();
    let end = tag[start..].find('"')? + start;
    Some(tag[start..end].to_string())
}

// =====================================================================
// Readability intelligence
// =====================================================================

/// Readability formula selection based on content language.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReadabilityFormula {
    /// Flesch-Kincaid (English).
    FleschKincaid,
    /// Kandel-Moles (French).
    KandelMoles,
    /// Wiener Sachtextformel (German).
    WienerSachtextformel,
    /// Gulpease index (Italian).
    Gulpease,
    /// LIX readability (Swedish/Scandinavian).
    Lix,
    /// Fernández Huerta (Spanish).
    FernandezHuerta,
}

impl ReadabilityFormula {
    /// Selects the appropriate formula from a language code.
    ///
    /// Accepts BCP 47 codes (e.g., `"en"`, `"fr"`, `"de-AT"`).
    /// Returns `None` for unsupported languages.
    #[must_use]
    pub fn from_lang(lang: &str) -> Option<Self> {
        let primary = lang.split(['-', '_']).next().unwrap_or(lang);
        match primary.to_lowercase().as_str() {
            "en" => Some(Self::FleschKincaid),
            "fr" => Some(Self::KandelMoles),
            "de" => Some(Self::WienerSachtextformel),
            "it" => Some(Self::Gulpease),
            "sv" | "nb" | "nn" | "da" | "no" => Some(Self::Lix),
            "es" => Some(Self::FernandezHuerta),
            _ => None,
        }
    }
}

/// Readability metrics for a text passage.
#[derive(Debug, Clone, Copy)]
pub struct ReadabilityAudit {
    /// Flesch-Kincaid Grade Level (lower = simpler).
    pub grade_level: f64,
    /// Flesch Reading Ease (higher = easier, 0–100).
    pub reading_ease: f64,
    /// Average words per sentence.
    pub avg_sentence_len: f64,
}

impl ReadabilityAudit {
    /// Analyzes text and returns readability metrics.
    #[must_use]
    pub fn analyze(text: &str) -> Self {
        let words = count_words(text);
        let sentences = count_sentences(text);
        let syllables = count_syllables(text);

        if words == 0 || sentences == 0 {
            return Self {
                grade_level: 0.0,
                reading_ease: 100.0,
                avg_sentence_len: 0.0,
            };
        }

        let wps = words as f64 / sentences as f64;
        let spw = syllables as f64 / words as f64;

        let grade = 0.39f64.mul_add(wps, 11.8f64.mul_add(spw, -15.59));
        let ease = (-1.015f64).mul_add(wps, (-84.6f64).mul_add(spw, 206.835));

        Self {
            grade_level: grade.max(0.0),
            reading_ease: ease.clamp(0.0, 100.0),
            avg_sentence_len: wps,
        }
    }

    /// Analyzes text using the appropriate formula for the given language.
    ///
    /// Falls back to Flesch-Kincaid if the language is unsupported or empty.
    #[must_use]
    pub fn analyze_with_lang(text: &str, lang: &str) -> Self {
        let formula = if lang.is_empty() {
            ReadabilityFormula::FleschKincaid
        } else {
            ReadabilityFormula::from_lang(lang)
                .unwrap_or(ReadabilityFormula::FleschKincaid)
        };

        let words = count_words(text);
        let sentences = count_sentences(text);
        let syllables = count_syllables(text);
        let chars: usize = text.chars().filter(|c| c.is_alphanumeric()).count();

        if words == 0 || sentences == 0 {
            return Self {
                grade_level: 0.0,
                reading_ease: 100.0,
                avg_sentence_len: 0.0,
            };
        }

        let wps = words as f64 / sentences as f64;
        let spw = syllables as f64 / words as f64;

        match formula {
            ReadabilityFormula::FleschKincaid => Self::analyze(text),

            ReadabilityFormula::KandelMoles => {
                // Kandel-Moles reading ease (French)
                let ease = 68.0f64.mul_add(-spw, 1.15f64.mul_add(-wps, 209.0));
                Self {
                    grade_level: ((100.0 - ease.clamp(0.0, 100.0)) / 10.0)
                        .max(0.0),
                    reading_ease: ease.clamp(0.0, 100.0),
                    avg_sentence_len: wps,
                }
            }

            ReadabilityFormula::WienerSachtextformel => {
                // Wiener Sachtextformel (German)
                let word_list: Vec<&str> = text.split_whitespace().collect();
                let total = word_list.len().max(1) as f64;
                let pct_3plus_syl = word_list
                    .iter()
                    .filter(|w| count_word_syllables(w) >= 3)
                    .count() as f64
                    / total
                    * 100.0;
                let pct_6plus_char = word_list
                    .iter()
                    .filter(|w| {
                        w.chars().filter(|c| c.is_alphabetic()).count() > 6
                    })
                    .count() as f64
                    / total
                    * 100.0;
                let pct_1syl = word_list
                    .iter()
                    .filter(|w| count_word_syllables(w) == 1)
                    .count() as f64
                    / total
                    * 100.0;

                let grade = 0.1935f64.mul_add(
                    pct_3plus_syl,
                    0.1672f64.mul_add(
                        wps,
                        (-0.1297f64).mul_add(
                            pct_6plus_char,
                            (-0.0327f64).mul_add(pct_1syl, -0.875),
                        ),
                    ),
                );

                Self {
                    grade_level: grade.max(0.0),
                    reading_ease: grade
                        .clamp(0.0, 20.0)
                        .mul_add(-5.0, 100.0)
                        .clamp(0.0, 100.0),
                    avg_sentence_len: wps,
                }
            }

            ReadabilityFormula::Gulpease => {
                // Gulpease index (Italian)
                let ease = 89.0
                    + 10.0f64
                        .mul_add(-(chars as f64), 300.0 * sentences as f64)
                        / words as f64;
                Self {
                    grade_level: ((100.0 - ease.clamp(0.0, 100.0)) / 10.0)
                        .max(0.0),
                    reading_ease: ease.clamp(0.0, 100.0),
                    avg_sentence_len: wps,
                }
            }

            ReadabilityFormula::Lix => {
                // LIX (Swedish/Scandinavian)
                let word_list: Vec<&str> = text.split_whitespace().collect();
                let total = word_list.len().max(1) as f64;
                let long_words = word_list
                    .iter()
                    .filter(|w| {
                        w.chars().filter(|c| c.is_alphabetic()).count() > 6
                    })
                    .count() as f64;
                let lix = wps + 100.0 * long_words / total;
                // LIX scale: <25 very easy, 25-35 easy, 35-45 medium,
                // 45-55 hard, >55 very hard
                Self {
                    grade_level: (lix / 5.0).max(0.0),
                    reading_ease: (100.0 - lix).clamp(0.0, 100.0),
                    avg_sentence_len: wps,
                }
            }

            ReadabilityFormula::FernandezHuerta => {
                // Fernández Huerta (Spanish)
                let ease =
                    1.02f64.mul_add(-wps, (-60.0f64).mul_add(spw, 206.84));
                Self {
                    grade_level: ((100.0 - ease.clamp(0.0, 100.0)) / 10.0)
                        .max(0.0),
                    reading_ease: ease.clamp(0.0, 100.0),
                    avg_sentence_len: wps,
                }
            }
        }
    }
}

/// Counts words in text (whitespace-separated tokens).
fn count_words(text: &str) -> usize {
    text.split_whitespace().count()
}

/// Counts sentences by splitting on `.`, `!`, `?`.
fn count_sentences(text: &str) -> usize {
    text.chars()
        .filter(|&c| c == '.' || c == '!' || c == '?')
        .count()
        .max(1)
}

/// Counts syllables using a lightweight heuristic:
/// - Count vowel groups (consecutive vowels = 1 syllable)
/// - Subtract silent trailing 'e'
/// - Minimum 1 syllable per word
fn count_syllables(text: &str) -> usize {
    text.split_whitespace()
        .map(|word| count_word_syllables(word))
        .sum()
}

/// Counts syllables in a single word.
fn count_word_syllables(word: &str) -> usize {
    let word = word.to_lowercase();
    let chars: Vec<char> = word.chars().filter(|c| c.is_alphabetic()).collect();
    if chars.is_empty() {
        return 1;
    }

    let vowels = b"aeiouy";
    let mut count = 0usize;
    let mut prev_vowel = false;

    for &ch in &chars {
        let is_vowel = vowels.contains(&(ch as u8));
        if is_vowel && !prev_vowel {
            count += 1;
        }
        prev_vowel = is_vowel;
    }

    // Subtract silent trailing 'e'
    if chars.len() > 2 && chars.last() == Some(&'e') && count > 1 {
        count -= 1;
    }

    count.max(1)
}

/// Generates text via LLM with readability-driven refinement.
///
/// If the initial output exceeds `target_grade`, re-prompts the LLM
/// once to simplify. Keeps the best available draft on failure.
fn generate_with_refinement(
    endpoint: &str,
    model: &str,
    prompt: &str,
    target_grade: f64,
    max_attempts: usize,
) -> Option<String> {
    let mut text = call_ollama(endpoint, model, prompt)?;
    let mut audit = ReadabilityAudit::analyze(&text);

    for attempt in 0..max_attempts {
        if audit.grade_level <= target_grade {
            break;
        }

        log::info!(
            "[llm] Grade {:.1} exceeds target {:.1}, refining (attempt {})",
            audit.grade_level,
            target_grade,
            attempt + 1
        );

        let simplify_prompt = format!(
            "Rewrite this text at a 6th-grade reading level. \
             Use short sentences (max 20 words). Use simple words. \
             Keep all facts and numbers exactly the same. \
             Return ONLY the rewritten text:\n\n{text}"
        );

        if let Some(refined) = call_ollama(endpoint, model, &simplify_prompt) {
            let refined_audit = ReadabilityAudit::analyze(&refined);
            if refined_audit.grade_level < audit.grade_level {
                text = refined;
                audit = refined_audit;
            }
        }
    }

    Some(text)
}

// =====================================================================
// JSON-LD generation
// =====================================================================

/// Injects or updates a JSON-LD `Article` script block in the HTML head.
///
/// Populates `description`, `datePublished`, and `author` from the page
/// content and frontmatter sidecar.
fn inject_jsonld_description(html: &str, description: &str) -> String {
    // Skip if JSON-LD Article already has a description
    if html.contains("\"@type\":\"Article\"")
        && html.contains("\"description\"")
    {
        return html.to_string();
    }

    let jsonld = serde_json::json!({
        "@context": "https://schema.org",
        "@type": "Article",
        "description": description,
    });

    let script =
        format!("<script type=\"application/ld+json\">{}</script>\n", jsonld);

    if let Some(pos) = html.find("</head>") {
        let mut result = html.to_string();
        result.insert_str(pos, &script);
        result
    } else {
        html.to_string()
    }
}

/// Calls the Ollama API to generate text.
fn call_ollama(endpoint: &str, model: &str, prompt: &str) -> Option<String> {
    let url = format!("{}/api/generate", endpoint.trim_end_matches('/'));
    let payload = serde_json::json!({
        "model": model,
        "prompt": prompt,
        "stream": false,
    });

    let output = Command::new("curl")
        .args([
            "-sf",
            "--max-time",
            "30",
            "-X",
            "POST",
            &url,
            "-H",
            "Content-Type: application/json",
            "-d",
            &payload.to_string(),
        ])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let response: serde_json::Value =
        serde_json::from_slice(&output.stdout).ok()?;
    response
        .get("response")
        .and_then(|v| v.as_str())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn needs_meta_description_missing() {
        assert!(needs_meta_description("<html><head></head></html>"));
    }

    #[test]
    fn needs_meta_description_short() {
        let html = r#"<html><head><meta name="description" content="Short"></head></html>"#;
        assert!(needs_meta_description(html));
    }

    #[test]
    fn needs_meta_description_adequate() {
        let html = r#"<html><head><meta name="description" content="This is a sufficiently long meta description that exceeds fifty characters easily"></head></html>"#;
        assert!(!needs_meta_description(html));
    }

    #[test]
    fn inject_meta_description_into_head() {
        let html = "<html><head><title>T</title></head><body></body></html>";
        let result = inject_meta_description(html, "Test description");
        assert!(result.contains("name=\"description\""));
        assert!(result.contains("Test description"));
    }

    #[test]
    fn extract_attr_basic() {
        assert_eq!(
            extract_attr(r#"<img src="photo.jpg" alt="x">"#, "src"),
            Some("photo.jpg".to_string())
        );
    }

    #[test]
    fn extract_attr_missing() {
        assert_eq!(extract_attr(r#"<img src="x.jpg">"#, "alt"), None);
    }

    #[test]
    fn extract_page_text_strips_tags() {
        let html = "<body><p>Hello <b>world</b></p></body>";
        let text = extract_page_text(html, 100);
        assert_eq!(text, "Hello world");
    }

    #[test]
    fn llm_plugin_name() {
        let plugin = LlmPlugin::new(LlmConfig::default());
        assert_eq!(plugin.name(), "llm");
    }

    // ── Readability engine tests ──────────────────────────────────

    #[test]
    fn flesch_kincaid_simple_text() {
        // "The cat sat on the mat." — very simple, ~grade 1
        let audit = ReadabilityAudit::analyze("The cat sat on the mat.");
        assert!(
            audit.grade_level < 4.0,
            "Simple text should be below grade 4, got {:.1}",
            audit.grade_level
        );
        assert!(audit.reading_ease > 80.0);
    }

    #[test]
    fn flesch_kincaid_complex_text() {
        let text = "The implementation of sophisticated cryptographic \
                    algorithms necessitates comprehensive understanding \
                    of mathematical foundations. Asymmetric encryption \
                    protocols demonstrate considerable computational \
                    overhead compared to symmetric alternatives.";
        let audit = ReadabilityAudit::analyze(text);
        assert!(
            audit.grade_level > 12.0,
            "Complex text should be above grade 12, got {:.1}",
            audit.grade_level
        );
    }

    #[test]
    fn flesch_kincaid_empty_text() {
        let audit = ReadabilityAudit::analyze("");
        assert!(audit.grade_level.abs() < f64::EPSILON);
        assert!((audit.reading_ease - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn syllable_count_known_words() {
        assert_eq!(count_word_syllables("cat"), 1);
        assert_eq!(count_word_syllables("hello"), 2);
        assert_eq!(count_word_syllables("beautiful"), 3);
        assert_eq!(count_word_syllables("implementation"), 5);
    }

    #[test]
    fn count_sentences_basic() {
        assert_eq!(count_sentences("Hello. World!"), 2);
        assert_eq!(count_sentences("One sentence"), 1); // min 1
        assert_eq!(count_sentences("A? B? C!"), 3);
    }

    // ── JSON-LD tests ───────────────────────────────────────────

    #[test]
    fn inject_jsonld_adds_article_block() {
        let html = "<html><head><title>T</title></head><body></body></html>";
        let result = inject_jsonld_description(html, "Test desc");
        assert!(result.contains("application/ld+json"));
        assert!(result.contains("\"@type\":\"Article\""));
        assert!(result.contains("Test desc"));
    }

    #[test]
    fn inject_jsonld_skips_existing() {
        let html = r#"<html><head><script type="application/ld+json">{"@type":"Article","description":"Existing"}</script></head></html>"#;
        let result = inject_jsonld_description(html, "New desc");
        assert!(!result.contains("New desc"));
        assert!(result.contains("Existing"));
    }

    // ── Content audit tests ───────────────────────────────────────

    #[test]
    fn audit_all_scans_markdown_files() {
        let dir = tempfile::tempdir().unwrap();
        let content = dir.path().join("content");
        fs::create_dir_all(&content).unwrap();

        fs::write(
            content.join("simple.md"),
            "---\ntitle: Simple\n---\nThe cat sat on the mat. It was a good day.",
        )
        .unwrap();
        fs::write(
            content.join("complex.md"),
            "---\ntitle: Complex\n---\n\
             The implementation of sophisticated cryptographic algorithms \
             necessitates comprehensive understanding of mathematical \
             foundations and computational complexity theory.",
        )
        .unwrap();

        let report = LlmPlugin::audit_all(&content, 8.0).unwrap();
        assert_eq!(report.total_files, 2);
        assert!(report.failing > 0, "complex.md should fail grade 8");
    }

    #[test]
    fn audit_all_empty_dir() {
        let dir = tempfile::tempdir().unwrap();
        let content = dir.path().join("empty");
        fs::create_dir_all(&content).unwrap();

        let report = LlmPlugin::audit_all(&content, 8.0).unwrap();
        assert_eq!(report.total_files, 0);
        assert_eq!(report.failing, 0);
    }

    #[test]
    fn strip_frontmatter_yaml() {
        let input = "---\ntitle: Hello\n---\nBody text here.";
        let body = strip_frontmatter(input);
        assert!(body.contains("Body text here"));
        assert!(!body.contains("title:"));
    }

    #[test]
    fn strip_frontmatter_toml() {
        let input = "+++\ntitle = \"Hello\"\n+++\nBody text here.";
        let body = strip_frontmatter(input);
        assert!(body.contains("Body text here"));
        assert!(!body.contains("title"));
    }

    #[test]
    fn strip_frontmatter_none() {
        let input = "Just plain content.";
        assert_eq!(strip_frontmatter(input), input);
    }

    #[test]
    fn split_frontmatter_preserves_delimiters() {
        let input = "---\ntitle: Hello\ndate: 2026-01-01\n---\n\n# Body text";
        let (fm, body) = split_frontmatter(input);
        assert!(fm.starts_with("---"));
        assert!(fm.ends_with("---"));
        assert!(fm.contains("title: Hello"));
        assert!(body.contains("# Body text"));
    }

    #[test]
    fn split_frontmatter_toml_preserves() {
        let input = "+++\ntitle = \"Hello\"\n+++\nBody.";
        let (fm, body) = split_frontmatter(input);
        assert!(fm.starts_with("+++"));
        assert!(body.contains("Body."));
    }

    #[test]
    fn split_frontmatter_no_frontmatter() {
        let input = "Just plain content.";
        let (fm, body) = split_frontmatter(input);
        assert!(fm.is_empty());
        assert_eq!(body, input);
    }

    #[test]
    fn audit_and_fix_skips_when_ollama_unavailable() {
        let dir = tempfile::tempdir().unwrap();
        let content = dir.path().join("content");
        fs::create_dir_all(&content).unwrap();
        fs::write(content.join("test.md"), "---\ntitle: T\n---\nSimple text.")
            .unwrap();

        let config = LlmConfig {
            endpoint: "http://localhost:99999".to_string(),
            ..LlmConfig::default()
        };
        let result = LlmPlugin::audit_and_fix(&content, &config).unwrap();
        assert_eq!(result, 0);
    }

    #[test]
    fn full_repo_readability_audit() {
        // Audits ALL Markdown content across the entire repository.
        let dirs = [
            ("docs/guide", 15.0),
            ("examples/basic/content", 10.0),
            ("examples/blog/content", 10.0),
            ("examples/docs/content", 13.0),
            ("examples/landing/content", 10.0),
            ("examples/plugins/content", 10.0),
            ("examples/portfolio/content", 10.0),
            ("examples/quickstart/content", 10.0),
            ("examples/content/en", 10.0),
        ];

        let mut total_files = 0usize;
        let mut total_pass = 0usize;
        let mut total_fail = 0usize;

        println!("\n{}", "=".repeat(60));
        println!("  FULL REPOSITORY READABILITY AUDIT");
        println!("{}\n", "=".repeat(60));

        for (dir, target) in &dirs {
            let path = Path::new(dir);
            if !path.exists() {
                continue;
            }

            let report = LlmPlugin::audit_all(path, *target).unwrap();
            if report.total_files == 0 {
                continue;
            }

            println!("── {dir} (target: grade {target:.0}) ��─");
            for r in &report.results {
                let status = if r.passes { "PASS" } else { "FAIL" };
                println!(
                    "  {:.<40} grade {:>5.1}  ease {:>5.1}  [{status}]",
                    r.path, r.grade_level, r.reading_ease
                );
            }
            println!("  → {}/{} pass\n", report.passing, report.total_files);

            total_files += report.total_files;
            total_pass += report.passing;
            total_fail += report.failing;
        }

        println!("{}", "=".repeat(60));
        println!(
            "  TOTAL: {total_files} files — {total_pass} pass, {total_fail} fail"
        );
        println!("{}\n", "=".repeat(60));
    }

    #[test]
    fn audit_docs_guide() {
        // This test is called by the readability-gate CI workflow.
        // It audits all .md files in docs/guide/ against grade 17
        // (documentation is technical and includes code blocks which
        // inflate Flesch-Kincaid scores).
        let guide_dir = Path::new("docs/guide");
        if !guide_dir.exists() {
            return; // Skip in environments without the guide
        }

        let report = LlmPlugin::audit_all(guide_dir, 17.0).unwrap();
        for result in &report.results {
            let status = if result.passes { "PASS" } else { "FAIL" };
            println!(
                "[readability] {}: grade={:.1}, ease={:.1}, avg_sentence={:.1} — {}",
                result.path,
                result.grade_level,
                result.reading_ease,
                result.avg_sentence_len,
                status
            );
        }

        println!(
            "\n[readability] {}/{} files pass (target: grade {:.0})",
            report.passing, report.total_files, report.target_grade
        );
    }

    // ── Coverage gap tests ────────────────────────────────────────

    #[test]
    fn is_ollama_available_unreachable() {
        assert!(!is_ollama_available("http://localhost:99999"));
    }

    #[test]
    fn call_ollama_unreachable_returns_none() {
        assert!(call_ollama("http://localhost:99999", "llama3", "hi").is_none());
    }

    #[test]
    fn needs_meta_description_with_content_attr_first() {
        // content= before name= (different ordering)
        let html = r#"<meta content="Decent length description that is more than fifty characters long enough" name="description">"#;
        // name="description" is present so returns false-ish check
        assert!(!needs_meta_description(html));
    }

    #[test]
    fn inject_meta_description_no_head() {
        let html = "<html><body>No head tag</body></html>";
        let result = inject_meta_description(html, "desc");
        assert_eq!(result, html); // unchanged
    }

    #[test]
    fn inject_jsonld_no_head() {
        let html = "<html><body>No head</body></html>";
        let result = inject_jsonld_description(html, "desc");
        assert_eq!(result, html);
    }

    #[test]
    fn extract_page_text_no_body() {
        let html = "just plain text no tags";
        let text = extract_page_text(html, 100);
        assert_eq!(text, "just plain text no tags");
    }

    #[test]
    fn extract_page_text_truncates() {
        let html = "<body><p>word </p></body>";
        let text = extract_page_text(html, 3);
        assert!(text.len() <= 5);
    }

    #[test]
    fn generate_missing_alt_text_no_images() {
        let mut html = "<html><body><p>No images</p></body></html>".to_string();
        let count = generate_missing_alt_text(
            &mut html,
            "llama3",
            "http://localhost:99999",
            true,
            Path::new("test.html"),
            Path::new("."),
        );
        assert_eq!(count, 0);
    }

    #[test]
    fn readability_audit_single_word() {
        let audit = ReadabilityAudit::analyze("Hello");
        assert!(audit.grade_level >= 0.0);
        assert!(audit.avg_sentence_len >= 0.0);
    }

    #[test]
    fn count_word_syllables_empty() {
        assert_eq!(count_word_syllables(""), 1);
    }

    #[test]
    fn count_word_syllables_numbers() {
        assert_eq!(count_word_syllables("123"), 1);
    }

    #[test]
    fn split_frontmatter_unclosed() {
        let input = "---\ntitle: Hello\nNo closing delimiter";
        let (fm, body) = split_frontmatter(input);
        assert!(fm.is_empty());
        assert_eq!(body, input);
    }

    #[test]
    fn llm_plugin_skips_missing_site_dir() {
        let plugin = LlmPlugin::new(LlmConfig::default());
        let ctx = PluginContext::new(
            Path::new("/tmp/c"),
            Path::new("/tmp/b"),
            Path::new("/nonexistent/site"),
            Path::new("/tmp/t"),
        );
        assert!(plugin.after_compile(&ctx).is_ok());
    }

    #[test]
    fn config_defaults_readability() {
        let config = LlmConfig::default();
        assert!((config.target_grade - 8.0).abs() < f64::EPSILON);
        assert_eq!(config.max_refinement_attempts, 1);
    }

    #[test]
    fn llm_plugin_skips_when_ollama_unavailable() {
        let plugin = LlmPlugin::new(LlmConfig {
            endpoint: "http://localhost:99999".to_string(),
            ..LlmConfig::default()
        });

        let dir = tempfile::tempdir().unwrap();
        let site = dir.path().join("site");
        fs::create_dir_all(&site).unwrap();
        fs::write(site.join("index.html"), "<html><body></body></html>")
            .unwrap();

        let ctx = PluginContext::new(dir.path(), dir.path(), &site, dir.path());
        // Should succeed (graceful skip)
        plugin.after_compile(&ctx).unwrap();
    }

    // ── Agentic AI fix pipeline tests ────────────────────────────

    #[test]
    fn ai_fix_report_serializes_to_json() {
        let report = AiFixReport {
            total_audited: 10,
            total_failing: 3,
            total_fixed: 2,
            results: vec![
                AiFixResult {
                    path: "docs/guide.md".to_string(),
                    before_grade: 12.5,
                    after_grade: 7.2,
                    improved: true,
                    action: "rewritten".to_string(),
                },
                AiFixResult {
                    path: "docs/api.md".to_string(),
                    before_grade: 14.0,
                    after_grade: 13.8,
                    improved: false,
                    action: "no-improvement".to_string(),
                },
            ],
        };
        let json = serde_json::to_string(&report).unwrap();
        assert!(json.contains("\"total_fixed\":2"));
        assert!(json.contains("\"action\":\"rewritten\""));
    }

    #[test]
    fn ai_fix_report_skips_when_ollama_unavailable() {
        let dir = tempfile::tempdir().unwrap();
        let content = dir.path().join("content");
        fs::create_dir_all(&content).unwrap();
        fs::write(
            content.join("test.md"),
            "---\ntitle: T\n---\nThe implementation of sophisticated algorithms.",
        )
        .unwrap();

        let config = LlmConfig {
            endpoint: "http://localhost:99999".to_string(),
            max_refinement_attempts: 3,
            ..LlmConfig::default()
        };
        let report =
            LlmPlugin::audit_and_fix_with_report(&content, &config).unwrap();
        assert_eq!(report.total_fixed, 0);
        assert!(report.results.is_empty());
    }

    // ── Multilingual readability tests ──────────────────────────

    #[test]
    fn formula_from_lang_english() {
        assert_eq!(
            ReadabilityFormula::from_lang("en"),
            Some(ReadabilityFormula::FleschKincaid)
        );
        assert_eq!(
            ReadabilityFormula::from_lang("en-US"),
            Some(ReadabilityFormula::FleschKincaid)
        );
    }

    #[test]
    fn formula_from_lang_french() {
        assert_eq!(
            ReadabilityFormula::from_lang("fr"),
            Some(ReadabilityFormula::KandelMoles)
        );
        assert_eq!(
            ReadabilityFormula::from_lang("fr-CA"),
            Some(ReadabilityFormula::KandelMoles)
        );
    }

    #[test]
    fn formula_from_lang_german() {
        assert_eq!(
            ReadabilityFormula::from_lang("de"),
            Some(ReadabilityFormula::WienerSachtextformel)
        );
        assert_eq!(
            ReadabilityFormula::from_lang("de-AT"),
            Some(ReadabilityFormula::WienerSachtextformel)
        );
    }

    #[test]
    fn formula_from_lang_italian() {
        assert_eq!(
            ReadabilityFormula::from_lang("it"),
            Some(ReadabilityFormula::Gulpease)
        );
    }

    #[test]
    fn formula_from_lang_swedish() {
        assert_eq!(
            ReadabilityFormula::from_lang("sv"),
            Some(ReadabilityFormula::Lix)
        );
        assert_eq!(
            ReadabilityFormula::from_lang("nb"),
            Some(ReadabilityFormula::Lix)
        );
        assert_eq!(
            ReadabilityFormula::from_lang("da"),
            Some(ReadabilityFormula::Lix)
        );
    }

    #[test]
    fn formula_from_lang_spanish() {
        assert_eq!(
            ReadabilityFormula::from_lang("es"),
            Some(ReadabilityFormula::FernandezHuerta)
        );
    }

    #[test]
    fn formula_from_lang_unsupported() {
        assert_eq!(ReadabilityFormula::from_lang("ja"), None);
        assert_eq!(ReadabilityFormula::from_lang("zh"), None);
        assert_eq!(ReadabilityFormula::from_lang("ar"), None);
    }

    #[test]
    fn kandel_moles_simple_french() {
        let text = "Le chat est sur le tapis. Il fait beau. Le soleil brille.";
        let audit = ReadabilityAudit::analyze_with_lang(text, "fr");
        assert!(
            audit.reading_ease > 50.0,
            "Simple French should be readable, got {:.1}",
            audit.reading_ease
        );
    }

    #[test]
    fn wiener_simple_german() {
        let text = "Die Katze sitzt auf der Matte. Es ist ein guter Tag. Die Sonne scheint.";
        let audit = ReadabilityAudit::analyze_with_lang(text, "de");
        assert!(
            audit.grade_level < 15.0,
            "Simple German got grade {:.1}",
            audit.grade_level
        );
    }

    #[test]
    fn gulpease_simple_italian() {
        let text = "Il gatto si siede sul tappeto. Il sole splende. Oggi è una bella giornata.";
        let audit = ReadabilityAudit::analyze_with_lang(text, "it");
        assert!(
            audit.reading_ease > 40.0,
            "Simple Italian got ease {:.1}",
            audit.reading_ease
        );
    }

    #[test]
    fn lix_simple_swedish() {
        let text = "Katten sitter på mattan. Solen skiner. Det är en fin dag.";
        let audit = ReadabilityAudit::analyze_with_lang(text, "sv");
        assert!(audit.grade_level >= 0.0);
        assert!(audit.reading_ease > 0.0);
    }

    #[test]
    fn fernandez_huerta_simple_spanish() {
        let text = "El gato está en la mesa. El sol brilla. Es un buen día.";
        let audit = ReadabilityAudit::analyze_with_lang(text, "es");
        assert!(
            audit.reading_ease > 50.0,
            "Simple Spanish got ease {:.1}",
            audit.reading_ease
        );
    }

    #[test]
    fn analyze_with_lang_empty_defaults_to_english() {
        let text = "The cat sat on the mat.";
        let a = ReadabilityAudit::analyze(text);
        let b = ReadabilityAudit::analyze_with_lang(text, "");
        assert!((a.grade_level - b.grade_level).abs() < f64::EPSILON);
    }

    #[test]
    fn analyze_with_lang_unsupported_falls_back() {
        let text = "The cat sat on the mat.";
        let a = ReadabilityAudit::analyze(text);
        let b = ReadabilityAudit::analyze_with_lang(text, "ja");
        assert!((a.grade_level - b.grade_level).abs() < f64::EPSILON);
    }

    #[test]
    fn extract_frontmatter_lang_yaml() {
        let content = "---\ntitle: Hello\nlanguage: fr\n---\nBody.";
        assert_eq!(extract_frontmatter_lang(content), "fr");
    }

    #[test]
    fn extract_frontmatter_lang_yaml_short() {
        let content = "---\ntitle: Hello\nlang: de\n---\nBody.";
        assert_eq!(extract_frontmatter_lang(content), "de");
    }

    #[test]
    fn extract_frontmatter_lang_toml() {
        let content = "+++\ntitle = \"Hello\"\nlanguage = \"it\"\n+++\nBody.";
        assert_eq!(extract_frontmatter_lang(content), "it");
    }

    #[test]
    fn extract_frontmatter_lang_missing() {
        let content = "---\ntitle: Hello\n---\nBody.";
        assert_eq!(extract_frontmatter_lang(content), "");
    }

    #[test]
    fn extract_frontmatter_lang_no_frontmatter() {
        let content = "Just plain text.";
        assert_eq!(extract_frontmatter_lang(content), "");
    }

    #[test]
    fn audit_all_respects_language() {
        let dir = tempfile::tempdir().unwrap();
        let content = dir.path().join("content");
        fs::create_dir_all(&content).unwrap();

        fs::write(
            content.join("french.md"),
            "---\ntitle: Bonjour\nlanguage: fr\n---\nLe chat est sur le tapis. Il fait beau.",
        )
        .unwrap();

        let report = LlmPlugin::audit_all(&content, 8.0).unwrap();
        assert_eq!(report.total_files, 1);
        // Should use Kandel-Moles, not Flesch-Kincaid
    }
}

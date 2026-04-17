// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Client-side search index generator.
//!
//! Generates a JSON search index and injects a search UI into HTML pages,
//! providing instant full-text search without any server or external service.
//!
//! # How it works
//!
//! 1. At build time, `SearchIndex` scans all HTML files in the site directory.
//! 2. It extracts the page title, URL, headings, and body text.
//! 3. It writes a `search-index.json` file to the site root.
//! 4. The `SearchPlugin` injects a `<script>` tag and search UI into every
//!    HTML page that loads the index and performs client-side fuzzy matching.
//!
//! The search UI is a modal overlay activated by `Ctrl+K` / `Cmd+K`.

use crate::plugin::{Plugin, PluginContext};
use anyhow::{Context, Result};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// A single entry in the search index.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SearchEntry {
    /// Page title extracted from `<title>` or first `<h1>`.
    pub title: String,
    /// Relative URL path (e.g., `/about/index.html`).
    pub url: String,
    /// Plain-text body content, truncated to `MAX_CONTENT_LENGTH`.
    pub content: String,
    /// Section headings found on the page.
    pub headings: Vec<String>,
}

/// Maximum content length per page in the search index (characters).
/// Keeps the index compact for fast client-side loading.
pub const MAX_CONTENT_LENGTH: usize = 5_000;

/// Maximum number of pages to index.
pub const MAX_INDEX_ENTRIES: usize = 50_000;

/// The complete search index written to `search-index.json`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SearchIndex {
    /// All indexed pages.
    pub entries: Vec<SearchEntry>,
}

impl SearchIndex {
    /// Build a search index from all HTML files in `site_dir`.
    ///
    /// Walks the directory recursively, extracts content from each
    /// `.html` file, and returns the populated index.
    pub fn build(site_dir: &Path) -> Result<Self> {
        let html_files = collect_html_files(site_dir)?;
        let capped: Vec<_> =
            html_files.into_iter().take(MAX_INDEX_ENTRIES).collect();

        let entries: Vec<SearchEntry> = capped
            .par_iter()
            .map(|path| -> Result<SearchEntry> {
                let html = fs::read_to_string(path).with_context(|| {
                    format!("cannot read {}", path.display())
                })?;

                let rel_url = path
                    .strip_prefix(site_dir)
                    .unwrap_or(path)
                    .to_string_lossy()
                    .replace('\\', "/");

                let title = extract_title(&html);
                let headings = extract_headings(&html);
                let content = extract_text(&html);

                Ok(SearchEntry {
                    title,
                    url: format!("/{rel_url}"),
                    content: truncate(&content, MAX_CONTENT_LENGTH),
                    headings,
                })
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(Self { entries })
    }

    /// Write the index to `search-index.json` in the given directory.
    pub fn write(&self, site_dir: &Path) -> Result<()> {
        let json = serde_json::to_string(self)
            .context("failed to serialize search index")?;
        let path = site_dir.join("search-index.json");
        fs::write(&path, json)
            .with_context(|| format!("cannot write {}", path.display()))?;
        Ok(())
    }

    /// Number of indexed pages.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns true if the index has no entries.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// Localizable strings shown in the search widget UI.
///
/// All fields are plain text. They are HTML-escaped when substituted into
/// attributes/text and JS-escaped when substituted into the inline script
/// (for the "no results" message). Build a value with one of the bundled
/// constructors ([`SearchLabels::english`], [`SearchLabels::french`],
/// [`SearchLabels::for_locale`]) or construct your own for any locale.
#[derive(Debug, Clone)]
pub struct SearchLabels {
    /// Visible text on the trigger button (e.g. "Search").
    pub button_text: String,
    /// `aria-label` of the trigger button.
    pub button_aria: String,
    /// `aria-label` of the modal dialog.
    pub modal_aria: String,
    /// Placeholder text inside the input field.
    pub input_placeholder: String,
    /// `aria-label` of the input field.
    pub input_aria: String,
    /// Footer hint text shown next to the `Esc` key.
    pub footer_close: String,
    /// Footer hint text shown next to the up/down arrow keys.
    pub footer_navigate: String,
    /// Footer hint text shown next to the `Enter` key.
    pub footer_open: String,
    /// Message shown when a query has no matches. The literal `{query}`
    /// is replaced with the typed query at runtime.
    pub no_results: String,
}

/// Compact per-locale strings used by [`SearchLabels::for_locale`].
struct LocaleEntry {
    button: &'static str,
    placeholder: &'static str,
    close: &'static str,
    navigate: &'static str,
    open: &'static str,
    no_results: &'static str,
}

/// Translations for the locales bundled with the search widget.
const LOCALE_TABLE: &[(&str, LocaleEntry)] = &[
    ("en", LocaleEntry { button: "Search",     placeholder: "Search documentation...",                close: "close",     navigate: "navigate",   open: "open",     no_results: "No results for \u{201c}{query}\u{201d}" }),
    ("fr", LocaleEntry { button: "Rechercher", placeholder: "Rechercher dans la documentation...",    close: "fermer",    navigate: "naviguer",   open: "ouvrir",   no_results: "Aucun r\u{e9}sultat pour \u{ab}\u{a0}{query}\u{a0}\u{bb}" }),
    ("ar", LocaleEntry { button: "بحث",        placeholder: "ابحث في الوثائق...",                      close: "إغلاق",     navigate: "تنقل",        open: "فتح",      no_results: "لا توجد نتائج لـ «{query}»" }),
    ("bn", LocaleEntry { button: "অনুসন্ধান",  placeholder: "ডকুমেন্টেশন অনুসন্ধান করুন...",          close: "বন্ধ",      navigate: "নেভিগেট",     open: "খুলুন",    no_results: "{query} এর জন্য কোনো ফলাফল নেই" }),
    ("cs", LocaleEntry { button: "Hledat",     placeholder: "Prohledat dokumentaci...",               close: "zav\u{159}\u{ed}t", navigate: "proch\u{e1}zet", open: "otev\u{159}\u{ed}t", no_results: "\u{17d}\u{e1}dn\u{e9} v\u{fd}sledky pro \u{201e}{query}\u{201c}" }),
    ("de", LocaleEntry { button: "Suchen",     placeholder: "Dokumentation durchsuchen...",           close: "schlie\u{df}en", navigate: "navigieren", open: "\u{f6}ffnen", no_results: "Keine Ergebnisse f\u{fc}r \u{201e}{query}\u{201c}" }),
    ("es", LocaleEntry { button: "Buscar",     placeholder: "Buscar en la documentaci\u{f3}n...",    close: "cerrar",    navigate: "navegar",    open: "abrir",    no_results: "Sin resultados para \u{ab}{query}\u{bb}" }),
    ("ha", LocaleEntry { button: "Bincike",    placeholder: "Bincika takardun...",                    close: "rufe",      navigate: "kewaya",     open: "bu\u{6b}e", no_results: "Babu sakamako don \u{201c}{query}\u{201d}" }),
    ("he", LocaleEntry { button: "חיפוש",      placeholder: "חפש בתיעוד...",                          close: "סגור",       navigate: "נווט",        open: "פתח",      no_results: "אין תוצאות עבור «{query}»" }),
    ("hi", LocaleEntry { button: "खोजें",       placeholder: "दस्तावेज़ खोजें...",                      close: "बंद करें",   navigate: "नेविगेट",     open: "खोलें",    no_results: "{query} के लिए कोई परिणाम नहीं" }),
    ("id", LocaleEntry { button: "Cari",       placeholder: "Cari dokumentasi...",                    close: "tutup",     navigate: "navigasi",   open: "buka",     no_results: "Tidak ada hasil untuk \u{201c}{query}\u{201d}" }),
    ("it", LocaleEntry { button: "Cerca",      placeholder: "Cerca nella documentazione...",          close: "chiudi",    navigate: "naviga",     open: "apri",     no_results: "Nessun risultato per \u{ab}{query}\u{bb}" }),
    ("ja", LocaleEntry { button: "検索",        placeholder: "ドキュメントを検索...",                     close: "閉じる",    navigate: "移動",        open: "開く",     no_results: "「{query}」の結果はありません" }),
    ("ko", LocaleEntry { button: "검색",        placeholder: "문서 검색...",                              close: "닫기",       navigate: "탐색",        open: "열기",     no_results: "«{query}»에 대한 결과가 없습니다" }),
    ("nl", LocaleEntry { button: "Zoeken",     placeholder: "Documentatie doorzoeken...",             close: "sluiten",   navigate: "navigeren",  open: "openen",   no_results: "Geen resultaten voor \u{201c}{query}\u{201d}" }),
    ("pl", LocaleEntry { button: "Szukaj",     placeholder: "Przeszukaj dokumentacj\u{119}...",      close: "zamknij",   navigate: "nawiguj",    open: "otw\u{f3}rz", no_results: "Brak wynik\u{f3}w dla \u{201e}{query}\u{201d}" }),
    ("pt", LocaleEntry { button: "Pesquisar",  placeholder: "Pesquisar na documenta\u{e7}\u{e3}o...", close: "fechar",  navigate: "navegar",    open: "abrir",    no_results: "Sem resultados para \u{ab}{query}\u{bb}" }),
    ("ro", LocaleEntry { button: "Caut\u{103}", placeholder: "Caut\u{103} \u{ee}n documenta\u{21b}ie...", close: "\u{ee}nchide", navigate: "navigheaz\u{103}", open: "deschide", no_results: "Niciun rezultat pentru \u{201e}{query}\u{201d}" }),
    ("ru", LocaleEntry { button: "Поиск",      placeholder: "Поиск по документации...",               close: "закрыть",   navigate: "навигация",  open: "открыть",  no_results: "Нет результатов для «{query}»" }),
    ("sv", LocaleEntry { button: "S\u{f6}k",  placeholder: "S\u{f6}k i dokumentationen...",         close: "st\u{e4}ng", navigate: "navigera", open: "\u{f6}ppna", no_results: "Inga resultat f\u{f6}r \u{201d}{query}\u{201d}" }),
    ("th", LocaleEntry { button: "ค้นหา",       placeholder: "ค้นหาเอกสาร...",                          close: "ปิด",        navigate: "นำทาง",       open: "เปิด",      no_results: "ไม่พบผลลัพธ์สำหรับ \u{201c}{query}\u{201d}" }),
    ("tl", LocaleEntry { button: "Maghanap",   placeholder: "Maghanap sa dokumentasyon...",           close: "isara",     navigate: "mag-navigate", open: "buksan", no_results: "Walang resulta para sa \u{201c}{query}\u{201d}" }),
    ("tr", LocaleEntry { button: "Ara",        placeholder: "Belgelerde ara...",                      close: "kapat",     navigate: "gezin",      open: "a\u{e7}", no_results: "\u{201c}{query}\u{201d} i\u{e7}in sonu\u{e7} yok" }),
    ("uk", LocaleEntry { button: "Пошук",      placeholder: "Пошук у документації...",                close: "закрити",   navigate: "навігація",  open: "відкрити", no_results: "Немає результатів для «{query}»" }),
    ("vi", LocaleEntry { button: "T\u{ec}m ki\u{1ebf}m", placeholder: "T\u{ec}m trong t\u{e0}i li\u{1ec7}u...", close: "\u{111}\u{f3}ng", navigate: "\u{111}i\u{1ec1}u h\u{1b0}\u{1edb}ng", open: "m\u{1edf}", no_results: "Kh\u{f4}ng c\u{f3} k\u{1ebf}t qu\u{1ea3} cho \u{201c}{query}\u{201d}" }),
    ("yo", LocaleEntry { button: "Wáàwáà",     placeholder: "Ṣàwárí ìwé...",                           close: "pa",        navigate: "lọ kiri",    open: "ṣí",       no_results: "Kò sí àbájáde fún \u{201c}{query}\u{201d}" }),
    ("zh", LocaleEntry { button: "搜索",        placeholder: "搜索文档...",                              close: "关闭",       navigate: "导航",        open: "打开",     no_results: "「{query}」没有匹配结果" }),
    ("zh-tw", LocaleEntry { button: "搜尋",     placeholder: "搜尋文件...",                              close: "關閉",       navigate: "瀏覽",        open: "開啟",     no_results: "「{query}」找不到結果" }),
];

impl SearchLabels {
    /// English (default) labels.
    #[must_use]
    pub fn english() -> Self {
        Self::for_locale("en")
    }

    /// French labels.
    #[must_use]
    pub fn french() -> Self {
        Self::for_locale("fr")
    }

    /// Build labels for a known locale code (ISO 639-1, plus `zh-tw`).
    ///
    /// Lookup is case-insensitive. Falls back to English if the code is not
    /// in the bundled table.
    #[must_use]
    pub fn for_locale(code: &str) -> Self {
        let key = code.to_ascii_lowercase();
        let entry = LOCALE_TABLE.iter().find(|(c, _)| *c == key).map_or_else(
            || {
                // `LOCALE_TABLE` is a hand-authored constant array that
                // always contains the `en` entry; the `expect` is a
                // type-system formality, not a runtime risk.
                #[allow(clippy::expect_used)]
                let en = LOCALE_TABLE
                    .iter()
                    .find(|(c, _)| *c == "en")
                    .expect("en entry must exist in LOCALE_TABLE");
                &en.1
            },
            |(_, e)| e,
        );
        Self {
            button_text: entry.button.into(),
            button_aria: entry.button.into(),
            modal_aria: entry.button.into(),
            input_placeholder: entry.placeholder.into(),
            input_aria: entry.button.into(),
            footer_close: entry.close.into(),
            footer_navigate: entry.navigate.into(),
            footer_open: entry.open.into(),
            no_results: entry.no_results.into(),
        }
    }
}

impl Default for SearchLabels {
    fn default() -> Self {
        Self::english()
    }
}

/// Plugin that generates a search index and injects client-side search UI.
///
/// The unit form uses [`SearchLabels::english`] for the modal copy. To render
/// the widget in another language, construct a [`LocalizedSearchPlugin`].
///
/// # Example
///
/// ```rust
/// use ssg::plugin::PluginManager;
/// use ssg::search::SearchPlugin;
///
/// let mut pm = PluginManager::new();
/// pm.register(SearchPlugin);
/// ```
#[derive(Debug, Copy, Clone)]
pub struct SearchPlugin;

impl Plugin for SearchPlugin {
    fn name(&self) -> &'static str {
        "search"
    }

    fn after_compile(&self, ctx: &PluginContext) -> Result<()> {
        run_search(ctx, &SearchLabels::english())
    }
}

/// Variant of [`SearchPlugin`] that injects the widget with caller-supplied
/// localized [`SearchLabels`].
///
/// # Example
///
/// ```rust
/// use ssg::plugin::PluginManager;
/// use ssg::search::{LocalizedSearchPlugin, SearchLabels};
///
/// let mut pm = PluginManager::new();
/// pm.register(LocalizedSearchPlugin::new(SearchLabels::french()));
/// ```
#[derive(Debug, Clone)]
pub struct LocalizedSearchPlugin {
    labels: SearchLabels,
}

impl LocalizedSearchPlugin {
    /// Create a new localized search plugin with the given labels.
    #[must_use]
    pub const fn new(labels: SearchLabels) -> Self {
        Self { labels }
    }
}

impl Plugin for LocalizedSearchPlugin {
    fn name(&self) -> &'static str {
        "search"
    }

    fn after_compile(&self, ctx: &PluginContext) -> Result<()> {
        run_search(ctx, &self.labels)
    }
}

/// Shared body for [`SearchPlugin`] and [`LocalizedSearchPlugin`].
fn run_search(ctx: &PluginContext, labels: &SearchLabels) -> Result<()> {
    if !ctx.site_dir.exists() {
        return Ok(());
    }

    let index = SearchIndex::build(&ctx.site_dir)?;
    if index.is_empty() {
        return Ok(());
    }

    index.write(&ctx.site_dir)?;

    let html_files = ctx.get_html_files();
    let script = build_widget_script(labels);
    html_files
        .par_iter()
        .try_for_each(|path| inject_search_ui(path, &script))?;

    println!(
        "[search] Indexed {} pages, search-index.json written",
        index.len()
    );
    Ok(())
}

// =====================================================================
// HTML content extraction (lightweight, no external parser)
// =====================================================================

/// Extract the page title from `<title>` tag or first `<h1>`.
fn extract_title(html: &str) -> String {
    // Try <title>
    if let Some(start) = html.find("<title>") {
        let after = &html[start + 7..];
        if let Some(end) = after.find("</title>") {
            let title = &after[..end];
            if !title.trim().is_empty() {
                return strip_tags(title).trim().to_string();
            }
        }
    }
    // Fallback to first <h1>
    if let Some(start) = html.find("<h1") {
        let after = &html[start..];
        if let Some(gt) = after.find('>') {
            let content = &after[gt + 1..];
            if let Some(end) = content.find("</h1>") {
                return strip_tags(&content[..end]).trim().to_string();
            }
        }
    }
    String::new()
}

/// Extract all heading text (`<h1>` through `<h6>`).
fn extract_headings(html: &str) -> Vec<String> {
    let mut headings = Vec::new();
    for tag in &["h1", "h2", "h3", "h4", "h5", "h6"] {
        let open = format!("<{tag}");
        let close = format!("</{tag}>");
        let mut search_from = 0;

        while let Some(start) = html[search_from..].find(&open) {
            let abs_start = search_from + start;
            let after = &html[abs_start..];
            if let Some(gt) = after.find('>') {
                let content = &after[gt + 1..];
                if let Some(end) = content.find(&close) {
                    let text = strip_tags(&content[..end]).trim().to_string();
                    if !text.is_empty() {
                        headings.push(text);
                    }
                    search_from = abs_start + gt + 1 + end + close.len();
                } else {
                    break;
                }
            } else {
                break;
            }
        }
    }
    headings
}

/// Extract visible text from HTML, stripping all tags.
fn extract_text(html: &str) -> String {
    // Remove non-content blocks. Note: <header> is intentionally kept
    // so hero taglines / subtitles are searchable.
    let mut clean = html.to_string();
    for tag in &["script", "style", "nav", "footer", "head"] {
        let open = format!("<{tag}");
        let close = format!("</{tag}>");
        while let Some(start) = clean.find(&open) {
            if let Some(end) = clean[start..].find(&close) {
                clean.replace_range(start..start + end + close.len(), " ");
            } else {
                break;
            }
        }
    }
    strip_tags(&clean)
}

/// Remove all HTML tags, collapse whitespace.
fn strip_tags(html: &str) -> String {
    let mut result = String::with_capacity(html.len());
    let mut in_tag = false;
    for ch in html.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => {
                in_tag = false;
                result.push(' ');
            }
            _ if !in_tag => result.push(ch),
            _ => {}
        }
    }
    // Collapse whitespace
    let mut collapsed = String::with_capacity(result.len());
    let mut prev_space = false;
    for ch in result.chars() {
        if ch.is_whitespace() {
            if !prev_space {
                collapsed.push(' ');
                prev_space = true;
            }
        } else {
            collapsed.push(ch);
            prev_space = false;
        }
    }
    collapsed.trim().to_string()
}

/// Truncate a string to approximately `max` characters at a word boundary.
fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        return s.to_string();
    }
    let byte_pos: usize = s
        .char_indices()
        .take(max)
        .last()
        .map_or(0, |(i, c)| i + c.len_utf8());
    let truncated = &s[..byte_pos];
    if let Some(last_space) = truncated.rfind(' ') {
        truncated[..last_space].to_string()
    } else {
        truncated.to_string()
    }
}

/// Collect all `.html` files under `dir` (delegates to `crate::walk`).
fn collect_html_files(dir: &Path) -> Result<Vec<PathBuf>> {
    crate::walk::walk_files_bounded_count(dir, "html", MAX_INDEX_ENTRIES)
}

/// Inject the search UI script into an HTML file.
///
/// Inserts a `<script>` block before `</body>` that:
/// 1. Loads `search-index.json`
/// 2. Creates a modal overlay with an input field
/// 3. Performs case-insensitive substring matching on title + content
/// 4. Displays results with highlighted snippets
/// 5. Activates on `Ctrl+K` / `Cmd+K`
fn inject_search_ui(path: &Path, script: &str) -> Result<()> {
    let html = fs::read_to_string(path)
        .with_context(|| format!("cannot read {}", path.display()))?;

    if html.contains("ssg-search-widget") {
        return Ok(()); // Already injected
    }

    let injected = if let Some(pos) = html.rfind("</body>") {
        format!("{}{}{}", &html[..pos], script, &html[pos..])
    } else {
        format!("{html}{script}")
    };

    fs::write(path, injected)
        .with_context(|| format!("cannot write {}", path.display()))?;
    Ok(())
}

/// Render [`SEARCH_WIDGET_SCRIPT`] (a template) with the given labels.
///
/// HTML attribute / text values are HTML-escaped; the `no_results` string is
/// also JS-escaped because it ends up inside a single-quoted JS string literal.
fn build_widget_script(labels: &SearchLabels) -> String {
    let no_results_with_expr = html_escape(&labels.no_results)
        .replace("{query}", "&ldquo;\'+esc(q)+\'&rdquo;");

    SEARCH_WIDGET_SCRIPT
        .replace("{{SSG_BTN_ARIA}}", &html_escape(&labels.button_aria))
        .replace("{{SSG_BTN_TEXT}}", &html_escape(&labels.button_text))
        .replace("{{SSG_MODAL_ARIA}}", &html_escape(&labels.modal_aria))
        .replace(
            "{{SSG_INPUT_PLACEHOLDER}}",
            &html_escape(&labels.input_placeholder),
        )
        .replace("{{SSG_INPUT_ARIA}}", &html_escape(&labels.input_aria))
        .replace("{{SSG_FOOTER_CLOSE}}", &html_escape(&labels.footer_close))
        .replace(
            "{{SSG_FOOTER_NAVIGATE}}",
            &html_escape(&labels.footer_navigate),
        )
        .replace("{{SSG_FOOTER_OPEN}}", &html_escape(&labels.footer_open))
        .replace("{{SSG_NO_RESULTS}}", &js_escape(&no_results_with_expr))
}

/// Minimal HTML escaper covering the characters that matter inside attribute
/// values and text nodes.
fn html_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(ch),
        }
    }
    out
}

/// Escape a string so it is safe to embed inside a single-quoted JS literal.
fn js_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '\'' => out.push_str("\\\'"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            _ => out.push(ch),
        }
    }
    out
}

/// The self-contained search widget (HTML + CSS + JS).
///
/// Includes a fixed search button in the top-right corner (like pacs008.com's
/// `DocSearch` bar) that opens a full-screen search modal. Also responds to
/// `Ctrl+K` / `Cmd+K`.
const SEARCH_WIDGET_SCRIPT: &str = r#"
<!-- SSG Search Widget -->
<div id="ssg-search-widget">
<style>
/* ── Trigger button (always visible) ── */
#ssg-search-btn{position:fixed;top:16px;right:16px;z-index:9998;display:flex;align-items:center;gap:8px;padding:8px 16px;background:#fff;border:1px solid #d1d5db;border-radius:8px;cursor:pointer;font-family:-apple-system,system-ui,sans-serif;font-size:14px;color:#595960;box-shadow:0 1px 3px rgba(0,0,0,.08);transition:border-color .15s,box-shadow .15s}
#ssg-search-btn:hover{border-color:#595960;box-shadow:0 2px 6px rgba(0,0,0,.12)}
#ssg-search-btn svg{width:16px;height:16px;stroke:currentColor;fill:none;stroke-width:2;stroke-linecap:round;stroke-linejoin:round}
#ssg-search-btn kbd{font-family:inherit;font-size:11px;padding:2px 6px;background:#f3f4f6;border:1px solid #e5e7eb;border-radius:4px;color:#595960;margin-left:4px}
/* ── Modal overlay ── */
#ssg-search-overlay{display:none;position:fixed;inset:0;z-index:9999;background:rgba(0,0,0,.5);align-items:flex-start;justify-content:center;padding-top:12vh}
#ssg-search-overlay.active{display:flex}
#ssg-search-box{background:#fff;border-radius:12px;width:92%;max-width:640px;box-shadow:0 25px 60px rgba(0,0,0,.3);overflow:hidden;font-family:-apple-system,system-ui,sans-serif}
#ssg-search-header{display:flex;align-items:center;padding:0 16px;border-bottom:1px solid #e5e7eb}
#ssg-search-header svg{width:20px;height:20px;stroke:#9ca3af;fill:none;stroke-width:2;flex-shrink:0}
#ssg-search-input{flex:1;padding:16px 12px;font-size:16px;border:none;outline:none;background:transparent}
#ssg-search-results{max-height:50vh;overflow-y:auto}
.ssg-result{display:block;padding:12px 20px;text-decoration:none;color:#111;border-bottom:1px solid #f3f4f6;transition:background .1s}
.ssg-result:hover,.ssg-result.active{background:#ecfdf5}
.ssg-result-title{font-weight:600;font-size:15px;margin-bottom:3px}
.ssg-result-snippet{font-size:13px;color:#595960;line-height:1.5}
.ssg-result-snippet mark{background:#fef08a;color:inherit;border-radius:2px;padding:0 2px}
.ssg-no-results{padding:32px 20px;text-align:center;color:#595960;font-size:14px}
.ssg-search-footer{display:flex;gap:16px;padding:10px 20px;font-size:12px;color:#595960;border-top:1px solid #e5e7eb;justify-content:flex-end}
.ssg-search-footer kbd{font-family:inherit;font-size:11px;padding:1px 5px;background:#f3f4f6;border:1px solid #e5e7eb;border-radius:3px}
/* ── Dark mode (media query + data-theme attribute) ── */
@media(prefers-color-scheme:dark){
:root:not([data-theme="light"]) #ssg-search-btn{background:#1f2937;border-color:#374151;color:#cccccf}
:root:not([data-theme="light"]) #ssg-search-btn:hover{border-color:#4b5563}
:root:not([data-theme="light"]) #ssg-search-btn kbd{background:#374151;border-color:#4b5563;color:#cccccf}
:root:not([data-theme="light"]) #ssg-search-box{background:#1f2937;color:#f9fafb}
:root:not([data-theme="light"]) #ssg-search-header{border-color:#374151}
:root:not([data-theme="light"]) #ssg-search-input{color:#f9fafb}
:root:not([data-theme="light"]) .ssg-result{color:#f9fafb;border-color:#374151}
:root:not([data-theme="light"]) .ssg-result:hover,:root:not([data-theme="light"]) .ssg-result.active{background:#374151}
:root:not([data-theme="light"]) .ssg-result-snippet{color:#cccccf}
:root:not([data-theme="light"]) .ssg-result-snippet mark{background:#854d0e;color:#fef08a}
:root:not([data-theme="light"]) .ssg-no-results{color:#cccccf}
:root:not([data-theme="light"]) .ssg-search-footer{border-color:#374151;color:#cccccf}
:root:not([data-theme="light"]) .ssg-search-footer kbd{background:#374151;border-color:#4b5563}
}
[data-theme="dark"] #ssg-search-btn{background:#1f2937;border-color:#374151;color:#cccccf}
[data-theme="dark"] #ssg-search-btn:hover{border-color:#4b5563}
[data-theme="dark"] #ssg-search-btn kbd{background:#374151;border-color:#4b5563;color:#cccccf}
[data-theme="dark"] #ssg-search-box{background:#1f2937;color:#f9fafb}
[data-theme="dark"] #ssg-search-header{border-color:#374151}
[data-theme="dark"] #ssg-search-input{color:#f9fafb}
[data-theme="dark"] .ssg-result{color:#f9fafb;border-color:#374151}
[data-theme="dark"] .ssg-result:hover,[data-theme="dark"] .ssg-result.active{background:#374151}
[data-theme="dark"] .ssg-result-snippet{color:#cccccf}
[data-theme="dark"] .ssg-result-snippet mark{background:#854d0e;color:#fef08a}
[data-theme="dark"] .ssg-no-results{color:#cccccf}
[data-theme="dark"] .ssg-search-footer{border-color:#374151;color:#cccccf}
[data-theme="dark"] .ssg-search-footer kbd{background:#374151;border-color:#4b5563}
</style>
<!-- Search trigger button -->
<button id="ssg-search-btn" type="button" aria-label="{{SSG_BTN_ARIA}}">
<svg viewBox="0 0 24 24"><circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/></svg>
<span>{{SSG_BTN_TEXT}}</span>
<kbd>K</kbd>
</button>
<!-- Search modal -->
<div id="ssg-search-overlay" role="dialog" aria-label="{{SSG_MODAL_ARIA}}">
<div id="ssg-search-box">
<div id="ssg-search-header">
<svg viewBox="0 0 24 24"><circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/></svg>
<input id="ssg-search-input" type="search" placeholder="{{SSG_INPUT_PLACEHOLDER}}" autocomplete="off" aria-label="{{SSG_INPUT_ARIA}}"/>
</div>
<div id="ssg-search-results"></div>
<div class="ssg-search-footer"><span><kbd>Esc</kbd> {{SSG_FOOTER_CLOSE}}</span><span><kbd>&uarr;</kbd><kbd>&darr;</kbd> {{SSG_FOOTER_NAVIGATE}}</span><span><kbd>Enter</kbd> {{SSG_FOOTER_OPEN}}</span></div>
</div>
</div>
<script>
(function(){
var idx=null,overlay=document.getElementById('ssg-search-overlay'),
input=document.getElementById('ssg-search-input'),
results=document.getElementById('ssg-search-results'),
btn=document.getElementById('ssg-search-btn'),active=-1,
lm=location.pathname.match(/^\/(en|fr|ar|bn|cs|de|es|ha|he|hi|id|it|ja|ko|nl|pl|pt|ro|ru|sv|th|tl|tr|uk|vi|yo|zh-tw|zh)\//),
lp=lm?'/'+lm[1]:'';
function load(){if(idx)return Promise.resolve();var sp=lm?'/'+lm[1]+'/search-index.json':'/search-index.json';return fetch(sp).then(function(r){return r.json()}).then(function(d){idx=d.entries||[]}).catch(function(){idx=[]})}
function open(){load().then(function(){overlay.classList.add('active');input.value='';results.innerHTML='';input.focus();active=-1})}
function close(){overlay.classList.remove('active');active=-1}
function highlight(text,q){if(!q)return esc(text);var re=new RegExp('('+q.replace(/[.*+?^${}()|[\]\\]/g,'\\$&')+')','gi');return esc(text).replace(re,'<mark>$1</mark>')}
function esc(s){var d=document.createElement('div');d.textContent=s;return d.innerHTML}
function snippet(content,q,len){len=len||150;if(!q)return esc(content.substring(0,len));var i=content.toLowerCase().indexOf(q.toLowerCase());if(i<0)return esc(content.substring(0,len));var s=Math.max(0,i-50),e=Math.min(content.length,i+len);var t=(s>0?'...':'')+content.substring(s,e)+(e<content.length?'...':'');return highlight(t,q)}
function search(q){if(!idx||!q){results.innerHTML='';return}q=q.trim();if(!q){results.innerHTML='';return}var ql=q.toLowerCase(),hits=[];
for(var i=0;i<idx.length&&hits.length<20;i++){var e=idx[i],s=0;if(e.title.toLowerCase().indexOf(ql)>=0)s+=10;if(e.content.toLowerCase().indexOf(ql)>=0)s+=5;for(var h=0;h<e.headings.length;h++){if(e.headings[h].toLowerCase().indexOf(ql)>=0){s+=3;break}}if(s>0)hits.push({entry:e,score:s})}
hits.sort(function(a,b){return b.score-a.score});
if(!hits.length){results.innerHTML='<div class="ssg-no-results">{{SSG_NO_RESULTS}}</div>';return}
var html='';for(var j=0;j<hits.length;j++){var e=hits[j].entry;html+='<a class="ssg-result" href="'+esc(lp+e.url)+'">'+'<div class="ssg-result-title">'+highlight(e.title,q)+'</div>'+'<div class="ssg-result-snippet">'+snippet(e.content,q)+'</div></a>'}
results.innerHTML=html;active=-1}
function nav(dir){var items=results.querySelectorAll('.ssg-result');if(!items.length)return;if(active>=0&&items[active])items[active].classList.remove('active');active+=dir;if(active<0)active=items.length-1;if(active>=items.length)active=0;items[active].classList.add('active');items[active].scrollIntoView({block:'nearest'})}
btn.addEventListener('click',function(){open()});
input.addEventListener('input',function(){search(this.value)});
overlay.addEventListener('click',function(e){if(e.target===overlay)close()});
document.addEventListener('keydown',function(e){if((e.ctrlKey||e.metaKey)&&e.key==='k'){e.preventDefault();if(overlay.classList.contains('active'))close();else open()}
if(!overlay.classList.contains('active'))return;if(e.key==='Escape')close();if(e.key==='ArrowDown'){e.preventDefault();nav(1)}if(e.key==='ArrowUp'){e.preventDefault();nav(-1)}
if(e.key==='Enter'){e.preventDefault();var items=results.querySelectorAll('.ssg-result');if(active>=0&&items[active])window.location=items[active].href;else if(items[0])window.location=items[0].href}})
})();
</script>
</div>
"#;

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn make_html(title: &str, body: &str) -> String {
        format!(
            "<html><head><title>{title}</title></head>\
             <body><h1>{title}</h1>{body}</body></html>"
        )
    }

    #[test]
    fn extract_title_from_title_tag() {
        let html =
            "<html><head><title>My Page</title></head><body></body></html>";
        assert_eq!(extract_title(html), "My Page");
    }

    #[test]
    fn extract_title_from_h1() {
        let html = "<html><body><h1>Heading</h1></body></html>";
        assert_eq!(extract_title(html), "Heading");
    }

    #[test]
    fn extract_title_empty() {
        assert_eq!(extract_title("<html><body></body></html>"), "");
    }

    #[test]
    fn extract_headings_multiple() {
        let html = "<h1>Title</h1><h2>Intro</h2><h3>Detail</h3>";
        let h = extract_headings(html);
        assert_eq!(h, vec!["Title", "Intro", "Detail"]);
    }

    #[test]
    fn extract_headings_with_attributes() {
        let html = r#"<h2 class="section" id="s1">Section One</h2>"#;
        let h = extract_headings(html);
        assert_eq!(h, vec!["Section One"]);
    }

    #[test]
    fn extract_text_strips_tags() {
        let html = "<p>Hello <strong>world</strong></p>";
        let text = extract_text(html);
        assert_eq!(text, "Hello world");
    }

    #[test]
    fn extract_text_removes_scripts() {
        let html = "<body><script>alert(1)</script><p>Visible</p></body>";
        let text = extract_text(html);
        assert!(text.contains("Visible"));
        assert!(!text.contains("alert"));
    }

    #[test]
    fn strip_tags_collapses_whitespace() {
        let result = strip_tags("<p>  hello   <br>  world  </p>");
        assert_eq!(result, "hello world");
    }

    #[test]
    fn truncate_short_string() {
        assert_eq!(truncate("short", 100), "short");
    }

    #[test]
    fn truncate_at_word_boundary() {
        let result = truncate("hello beautiful world", 18);
        assert_eq!(result, "hello beautiful");
    }

    #[test]
    fn search_index_build_from_directory() -> Result<()> {
        let tmp = tempdir()?;
        fs::write(
            tmp.path().join("index.html"),
            make_html("Home", "<p>Welcome to SSG</p>"),
        )?;
        fs::write(
            tmp.path().join("about.html"),
            make_html("About", "<p>About this site</p>"),
        )?;

        let index = SearchIndex::build(tmp.path())?;
        assert_eq!(index.len(), 2);
        assert!(!index.is_empty());

        let titles: Vec<&str> =
            index.entries.iter().map(|e| e.title.as_str()).collect();
        assert!(titles.contains(&"Home"));
        assert!(titles.contains(&"About"));
        Ok(())
    }

    #[test]
    fn search_index_write_creates_json() -> Result<()> {
        let tmp = tempdir()?;
        let index = SearchIndex {
            entries: vec![SearchEntry {
                title: "Test".into(),
                url: "/test.html".into(),
                content: "Test content".into(),
                headings: vec!["Heading".into()],
            }],
        };
        index.write(tmp.path())?;

        let path = tmp.path().join("search-index.json");
        assert!(path.exists());
        let json: SearchIndex =
            serde_json::from_str(&fs::read_to_string(&path)?)?;
        assert_eq!(json.entries.len(), 1);
        assert_eq!(json.entries[0].title, "Test");
        Ok(())
    }

    #[test]
    fn search_index_empty_directory() -> Result<()> {
        let tmp = tempdir()?;
        let index = SearchIndex::build(tmp.path())?;
        assert!(index.is_empty());
        Ok(())
    }

    #[test]
    fn search_index_ignores_non_html() -> Result<()> {
        let tmp = tempdir()?;
        fs::write(tmp.path().join("style.css"), "body{}")?;
        fs::write(tmp.path().join("data.json"), "{}")?;
        let index = SearchIndex::build(tmp.path())?;
        assert!(index.is_empty());
        Ok(())
    }

    #[test]
    fn search_index_nested_directories() -> Result<()> {
        let tmp = tempdir()?;
        fs::create_dir_all(tmp.path().join("blog"))?;
        fs::write(tmp.path().join("index.html"), make_html("Home", ""))?;
        fs::write(
            tmp.path().join("blog/post.html"),
            make_html("Post", "<p>Blog content</p>"),
        )?;

        let index = SearchIndex::build(tmp.path())?;
        assert_eq!(index.len(), 2);
        let urls: Vec<&str> =
            index.entries.iter().map(|e| e.url.as_str()).collect();
        assert!(urls.iter().any(|u| u.contains("blog")));
        Ok(())
    }

    #[test]
    fn search_entry_content_truncated() -> Result<()> {
        let tmp = tempdir()?;
        let long_text = "word ".repeat(2000); // 10,000 chars
        fs::write(
            tmp.path().join("long.html"),
            make_html("Long", &format!("<p>{long_text}</p>")),
        )?;

        let index = SearchIndex::build(tmp.path())?;
        assert!(index.entries[0].content.len() <= MAX_CONTENT_LENGTH);
        Ok(())
    }

    #[test]
    fn inject_search_ui_adds_widget() -> Result<()> {
        let tmp = tempdir()?;
        let path = tmp.path().join("page.html");
        fs::write(&path, "<html><body><p>Hello</p></body></html>")?;

        let script = build_widget_script(&SearchLabels::english());
        inject_search_ui(&path, &script)?;

        let result = fs::read_to_string(&path)?;
        assert!(result.contains("ssg-search-widget"));
        assert!(result.contains("search-index.json"));
        assert!(result.contains("ctrlKey"));
        Ok(())
    }

    #[test]
    fn inject_search_ui_idempotent() -> Result<()> {
        let tmp = tempdir()?;
        let path = tmp.path().join("page.html");
        fs::write(&path, "<html><body><p>Hi</p></body></html>")?;

        let script = build_widget_script(&SearchLabels::english());
        inject_search_ui(&path, &script)?;
        let first = fs::read_to_string(&path)?;

        inject_search_ui(&path, &script)?;
        let second = fs::read_to_string(&path)?;

        assert_eq!(first, second); // No double injection
        Ok(())
    }

    #[test]
    fn search_plugin_name() {
        assert_eq!(SearchPlugin.name(), "search");
    }

    #[test]
    fn search_plugin_full_pipeline() -> Result<()> {
        let tmp = tempdir()?;
        fs::write(
            tmp.path().join("index.html"),
            make_html("Home", "<p>Welcome</p>"),
        )?;
        fs::write(
            tmp.path().join("about.html"),
            make_html("About", "<p>About us</p>"),
        )?;

        let ctx = PluginContext::new(
            Path::new("content"),
            Path::new("build"),
            tmp.path(),
            Path::new("templates"),
        );
        SearchPlugin.after_compile(&ctx)?;

        // Index was written
        assert!(tmp.path().join("search-index.json").exists());

        // Widget was injected
        let html = fs::read_to_string(tmp.path().join("index.html"))?;
        assert!(html.contains("ssg-search-widget"));
        Ok(())
    }

    #[test]
    fn search_plugin_nonexistent_dir() -> Result<()> {
        let ctx = PluginContext::new(
            Path::new("c"),
            Path::new("b"),
            Path::new("/nonexistent"),
            Path::new("t"),
        );
        SearchPlugin.after_compile(&ctx)?; // Should not error
        Ok(())
    }

    #[test]
    fn search_plugin_registers() {
        use crate::plugin::PluginManager;
        let mut pm = PluginManager::new();
        pm.register(SearchPlugin);
        assert_eq!(pm.names(), vec!["search"]);
    }

    #[test]
    fn search_entry_serialize_deserialize() -> Result<()> {
        let entry = SearchEntry {
            title: "Test".into(),
            url: "/test.html".into(),
            content: "Content".into(),
            headings: vec!["H1".into()],
        };
        let json = serde_json::to_string(&entry)?;
        let parsed: SearchEntry = serde_json::from_str(&json)?;
        assert_eq!(entry, parsed);
        Ok(())
    }

    // -------------------------------------------------------------------
    // Targeted edge-case coverage
    // -------------------------------------------------------------------

    #[test]
    fn search_plugin_after_compile_empty_index_short_circuits() -> Result<()> {
        // Line 136: `if index.is_empty() { return Ok(()) }`. Need a
        // site with HTML files that produce zero entries — easiest:
        // a site with only a stylesheet (collect_html_files returns
        // empty, build returns empty index).
        let tmp = tempdir()?;
        fs::write(tmp.path().join("style.css"), "body{}")?;
        let ctx = PluginContext::new(
            Path::new("content"),
            Path::new("build"),
            tmp.path(),
            Path::new("templates"),
        );
        SearchPlugin.after_compile(&ctx)?;
        // No search-index.json should have been written.
        assert!(!tmp.path().join("search-index.json").exists());
        Ok(())
    }

    #[test]
    fn extract_title_empty_title_falls_back_to_h1() {
        // Line 167 false branch: title trimmed is empty, so we fall
        // through to the h1 fallback at lines 172-180.
        let html = "<html><head><title>   </title></head><body><h1>Heading One</h1></body></html>";
        assert_eq!(extract_title(html), "Heading One");
    }

    #[test]
    fn extract_title_no_title_tag_falls_back_to_h1() {
        // Lines 178-179: the h1 fallback Some-Some success path.
        let html = "<html><body><h1>From H1</h1></body></html>";
        assert_eq!(extract_title(html), "From H1");
    }

    #[test]
    fn extract_title_h1_with_attributes_works() {
        // Verifies the `find('>')` step at line 174 handles attrs.
        let html = r#"<html><body><h1 class="title">Attrs</h1></body></html>"#;
        assert_eq!(extract_title(html), "Attrs");
    }

    #[test]
    fn extract_title_no_title_no_h1_returns_empty() {
        let html = "<html><body><p>just a paragraph</p></body></html>";
        assert_eq!(extract_title(html), "");
    }

    #[test]
    fn extract_title_unterminated_title_falls_back_to_h1() {
        // <title> open without close — `find("</title>")` returns
        // None, the outer `if let` body exits, and the function
        // proceeds to the <h1> fallback.
        let html =
            "<html><head><title>Open<body><h1>Fallback</h1></body></html>";
        let result = extract_title(html);
        assert_eq!(result, "Fallback");
    }

    #[test]
    fn extract_title_unterminated_h1_returns_empty() {
        // <h1> open without `>` and without `</h1>` — both inner
        // `if let`s return None, function returns "".
        let html = "<html><body><h1 attr=\"open";
        assert_eq!(extract_title(html), "");
    }

    #[test]
    fn extract_headings_unterminated_h_tag_breaks_inner_loop() {
        // Line 204: the `break` when no `</hN>` close tag is found.
        let html = "<html><body><h1>Has close</h1><h2>no close tag";
        let headings = extract_headings(html);
        // The first heading is captured; the unterminated one
        // breaks out of the inner loop without panicking.
        assert!(headings.contains(&"Has close".to_string()));
    }

    #[test]
    fn extract_headings_unterminated_open_tag_breaks_outer() {
        // Line 207: the `break` when `<h1` has no `>`. Build a
        // pathological string that contains `<h1` but never `>`
        // afterwards.
        let html = "<h1 attr=\"unterminated";
        let headings = extract_headings(html);
        assert!(headings.is_empty());
    }

    #[test]
    fn extract_text_unterminated_strip_tag_breaks() {
        // Line 225: the `break` in the strip loop when a tag opener
        // exists but no matching close. extract_text strips
        // <script>/<style>/etc. blocks; an unterminated <script>
        // hits the inner break.
        let html = "<html><body><script>unterminated<p>visible</p>";
        let _ = extract_text(html);
    }

    #[test]
    fn truncate_no_space_falls_back_to_byte_cut() {
        // Line 278: `else { truncated.to_string() }` when there is
        // no space within the first `max` characters.
        let result = truncate("oneverylongwordwithnospacesatall", 10);
        // Returns the byte-truncated string (no space to break on).
        assert_eq!(result, "oneverylon");
    }

    #[test]
    fn truncate_short_string_returned_unchanged() {
        // Line 266 true branch: input shorter than max returns as-is.
        assert_eq!(truncate("short", 100), "short");
    }

    #[test]
    fn collect_html_files_respects_bound() -> Result<()> {
        let tmp = tempdir()?;
        for i in 0..50 {
            fs::write(tmp.path().join(format!("p{i}.html")), "<html></html>")?;
        }
        let files = collect_html_files(tmp.path())?;
        assert_eq!(files.len(), 50);
        Ok(())
    }

    #[test]
    fn search_index_empty_site_dir() -> Result<()> {
        // Arrange
        let tmp = tempdir()?;

        // Act
        let index = SearchIndex::build(tmp.path())?;

        // Assert
        assert!(index.is_empty());
        assert_eq!(index.len(), 0);
        Ok(())
    }

    #[test]
    fn search_index_max_content_length_truncation() -> Result<()> {
        // Arrange
        let tmp = tempdir()?;
        let long_content = "a ".repeat(MAX_CONTENT_LENGTH + 1000);
        fs::write(
            tmp.path().join("long.html"),
            make_html("Long Page", &format!("<p>{long_content}</p>")),
        )?;

        // Act
        let index = SearchIndex::build(tmp.path())?;

        // Assert
        assert_eq!(index.len(), 1);
        assert!(
            index.entries[0].content.chars().count() <= MAX_CONTENT_LENGTH,
            "content should be truncated to at most MAX_CONTENT_LENGTH characters"
        );
        Ok(())
    }

    #[test]
    fn search_index_unicode_content() -> Result<()> {
        // Arrange
        let tmp = tempdir()?;
        let unicode_body = "<p>Héllo wörld! 日本語テスト 🦀🔍 Ñoño café</p>";
        fs::write(
            tmp.path().join("unicode.html"),
            make_html("Ünïcödé Pagé 🎉", unicode_body),
        )?;

        // Act
        let index = SearchIndex::build(tmp.path())?;

        // Assert
        assert_eq!(index.len(), 1);
        let entry = &index.entries[0];
        assert_eq!(entry.title, "Ünïcödé Pagé 🎉");
        assert!(entry.content.contains("日本語テスト"));
        assert!(entry.content.contains("🦀🔍"));
        assert!(entry.content.contains("café"));
        Ok(())
    }

    #[test]
    fn search_plugin_nonexistent_dir_returns_ok() -> Result<()> {
        // Arrange
        let ctx = PluginContext::new(
            Path::new("content"),
            Path::new("build"),
            Path::new("/tmp/nonexistent_search_test_dir_xyz"),
            Path::new("templates"),
        );

        // Act
        let result = SearchPlugin.after_compile(&ctx);

        // Assert
        assert!(result.is_ok());
        Ok(())
    }

    #[test]
    fn inject_search_ui_no_body_tag() -> Result<()> {
        // Arrange
        let tmp = tempdir()?;
        let path = tmp.path().join("fragment.html");
        fs::write(&path, "<html><p>No body tag here</p></html>")?;

        // Act
        let script = build_widget_script(&SearchLabels::english());
        inject_search_ui(&path, &script)?;

        // Assert
        let result = fs::read_to_string(&path)?;
        assert!(
            result.contains("ssg-search-widget"),
            "widget should be appended even without </body>"
        );
        assert!(result.contains("<html><p>No body tag here</p></html>"));
        Ok(())
    }

    #[test]
    fn search_entry_serialization_roundtrip() -> Result<()> {
        // Arrange
        let entry = SearchEntry {
            title: "Roundtrip Test".into(),
            url: "/roundtrip/index.html".into(),
            content: "Some searchable content here".into(),
            headings: vec!["Introduction".into(), "Details".into()],
        };

        // Act
        let json = serde_json::to_string(&entry)?;
        let deserialized: SearchEntry = serde_json::from_str(&json)?;

        // Assert
        assert_eq!(entry, deserialized);
        assert_eq!(deserialized.title, "Roundtrip Test");
        assert_eq!(deserialized.headings.len(), 2);
        Ok(())
    }

    #[test]
    fn search_index_multiple_headings() -> Result<()> {
        // Arrange
        let tmp = tempdir()?;
        let html = "\
            <html><head><title>Multi Heading</title></head><body>\
            <h1>Main Title</h1>\
            <h2>Section A</h2>\
            <p>Content A</p>\
            <h3>Subsection A1</h3>\
            <p>Content A1</p>\
            </body></html>";
        fs::write(tmp.path().join("headings.html"), html)?;

        // Act
        let index = SearchIndex::build(tmp.path())?;

        // Assert
        assert_eq!(index.len(), 1);
        let entry = &index.entries[0];
        assert!(entry.headings.contains(&"Main Title".to_string()));
        assert!(entry.headings.contains(&"Section A".to_string()));
        assert!(entry.headings.contains(&"Subsection A1".to_string()));
        assert_eq!(entry.headings.len(), 3);
        Ok(())
    }

    #[test]
    fn search_index_nested_directories_deep() -> Result<()> {
        // Arrange
        let tmp = tempdir()?;
        fs::create_dir_all(tmp.path().join("docs/guide/advanced"))?;
        fs::write(
            tmp.path().join("index.html"),
            make_html("Root", "<p>Root page</p>"),
        )?;
        fs::write(
            tmp.path().join("docs/overview.html"),
            make_html("Docs", "<p>Docs overview</p>"),
        )?;
        fs::write(
            tmp.path().join("docs/guide/advanced/tips.html"),
            make_html("Tips", "<p>Advanced tips</p>"),
        )?;

        // Act
        let index = SearchIndex::build(tmp.path())?;

        // Assert
        assert_eq!(index.len(), 3);
        let urls: Vec<&str> =
            index.entries.iter().map(|e| e.url.as_str()).collect();
        assert!(urls.iter().any(|u| u.contains("docs/guide/advanced")));
        assert!(urls.iter().any(|u| u.contains("index.html")));
        Ok(())
    }

    // -----------------------------------------------------------------
    // SearchIndex::build — parallel path with multiple HTML files
    // -----------------------------------------------------------------

    #[test]
    fn search_index_build_parallel_with_many_files() -> Result<()> {
        let tmp = tempdir()?;
        for i in 0..10 {
            fs::write(
                tmp.path().join(format!("page{i}.html")),
                make_html(
                    &format!("Page {i}"),
                    &format!("<p>Content for page {i}</p>"),
                ),
            )?;
        }

        let index = SearchIndex::build(tmp.path())?;
        assert_eq!(index.len(), 10);

        // Verify all pages are indexed
        for i in 0..10 {
            let title = format!("Page {i}");
            assert!(
                index.entries.iter().any(|e| e.title == title),
                "missing entry for {title}"
            );
        }
        Ok(())
    }

    // -----------------------------------------------------------------
    // extract_headings — h1 through h6
    // -----------------------------------------------------------------

    #[test]
    fn extract_headings_all_levels() {
        let html = "\
            <h1>One</h1>\
            <h2>Two</h2>\
            <h3>Three</h3>\
            <h4>Four</h4>\
            <h5>Five</h5>\
            <h6>Six</h6>";
        let h = extract_headings(html);
        assert_eq!(h, vec!["One", "Two", "Three", "Four", "Five", "Six"]);
    }

    #[test]
    fn extract_headings_empty_heading_skipped() {
        let html = "<h1></h1><h2>Real Heading</h2>";
        let h = extract_headings(html);
        assert_eq!(h, vec!["Real Heading"]);
    }

    // -----------------------------------------------------------------
    // truncate — word boundary and short content
    // -----------------------------------------------------------------

    #[test]
    fn truncate_at_word_boundary_exact() {
        // truncate(s, 13) takes first 13 chars "one two three"
        // then finds last space at position 7, truncating to "one two"
        let result = truncate("one two three four five", 13);
        assert_eq!(result, "one two");
    }

    #[test]
    fn truncate_content_shorter_than_limit() {
        let input = "short text";
        assert_eq!(truncate(input, 1000), "short text");
    }

    #[test]
    fn truncate_exact_length_returns_unchanged() {
        let input = "exact";
        assert_eq!(truncate(input, 5), "exact");
    }

    // -----------------------------------------------------------------
    // SearchLabels::for_locale
    // -----------------------------------------------------------------

    #[test]
    fn search_labels_for_locale_french() {
        let labels = SearchLabels::for_locale("fr");
        assert_eq!(labels.button_text, "Rechercher");
        assert!(labels.input_placeholder.contains("Rechercher"));
        assert_eq!(labels.footer_close, "fermer");
    }

    #[test]
    fn search_labels_for_locale_german() {
        let labels = SearchLabels::for_locale("de");
        assert_eq!(labels.button_text, "Suchen");
        assert_eq!(labels.footer_open, "\u{f6}ffnen"); // öffnen
    }

    #[test]
    fn search_labels_for_locale_unknown_falls_back_to_english() {
        let labels = SearchLabels::for_locale("xx");
        assert_eq!(labels.button_text, "Search");
        assert!(labels.input_placeholder.contains("Search"));
        assert_eq!(labels.footer_close, "close");
    }

    #[test]
    fn search_labels_for_locale_case_insensitive() {
        let labels = SearchLabels::for_locale("FR");
        assert_eq!(labels.button_text, "Rechercher");
    }

    #[test]
    fn search_labels_for_locale_zh_tw() {
        let labels = SearchLabels::for_locale("zh-tw");
        assert_eq!(labels.button_text, "搜尋");
    }

    #[test]
    fn search_labels_default_is_english() {
        let labels = SearchLabels::default();
        assert_eq!(labels.button_text, "Search");
    }

    #[test]
    fn search_labels_english_constructor() {
        let labels = SearchLabels::english();
        assert_eq!(labels.button_text, "Search");
        assert_eq!(
            SearchLabels::english().input_placeholder,
            labels.input_placeholder
        );
    }

    #[test]
    fn search_labels_french_constructor() {
        let labels = SearchLabels::french();
        assert_eq!(labels.button_text, "Rechercher");
    }

    #[test]
    fn localized_search_plugin_new_keeps_supplied_labels() {
        let labels = SearchLabels::french();
        let p = LocalizedSearchPlugin::new(labels.clone());
        assert_eq!(p.labels.button_text, "Rechercher");
    }

    #[test]
    fn localized_search_plugin_name_is_search() {
        let p = LocalizedSearchPlugin::new(SearchLabels::default());
        assert_eq!(p.name(), "search");
    }

    #[test]
    fn localized_search_plugin_no_op_when_site_missing() -> Result<()> {
        let dir = tempdir().unwrap();
        let nope = dir.path().join("nope");
        let ctx = PluginContext::new(
            Path::new("c"),
            Path::new("b"),
            &nope,
            Path::new("t"),
        );
        LocalizedSearchPlugin::new(SearchLabels::default())
            .after_compile(&ctx)?;
        Ok(())
    }

    #[test]
    fn localized_search_plugin_writes_index_with_localized_labels() -> Result<()>
    {
        let dir = tempdir().unwrap();
        fs::write(
            dir.path().join("page.html"),
            "<html><head><title>P</title></head><body>x</body></html>",
        )?;
        let ctx = PluginContext::new(
            Path::new("c"),
            Path::new("b"),
            dir.path(),
            Path::new("t"),
        );
        LocalizedSearchPlugin::new(SearchLabels::french())
            .after_compile(&ctx)?;
        let html = fs::read_to_string(dir.path().join("page.html"))?;
        // Localized button text should appear in the injected widget.
        assert!(
            html.contains("Rechercher"),
            "French label 'Rechercher' should appear in injected UI"
        );
        Ok(())
    }
}

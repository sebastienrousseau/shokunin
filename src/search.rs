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
        let mut entries =
            Vec::with_capacity(html_files.len().min(MAX_INDEX_ENTRIES));

        for path in html_files.iter().take(MAX_INDEX_ENTRIES) {
            let html = fs::read_to_string(path)
                .with_context(|| format!("cannot read {}", path.display()))?;

            let rel_url = path
                .strip_prefix(site_dir)
                .unwrap_or(path)
                .to_string_lossy()
                .replace('\\', "/");

            let title = extract_title(&html);
            let headings = extract_headings(&html);
            let content = extract_text(&html);

            entries.push(SearchEntry {
                title,
                url: format!("/{rel_url}"),
                content: truncate(&content, MAX_CONTENT_LENGTH),
                headings,
            });
        }

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
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns true if the index has no entries.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// Plugin that generates a search index and injects client-side search UI.
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
        if !ctx.site_dir.exists() {
            return Ok(());
        }

        let index = SearchIndex::build(&ctx.site_dir)?;
        if index.is_empty() {
            return Ok(());
        }

        index.write(&ctx.site_dir)?;

        // Inject search UI into all HTML files
        let html_files = collect_html_files(&ctx.site_dir)?;
        for path in &html_files {
            inject_search_ui(path)?;
        }

        println!(
            "[search] Indexed {} pages, search-index.json written",
            index.len()
        );
        Ok(())
    }
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
    // Remove <script>, <style>, <nav>, <header>, <footer> blocks
    let mut clean = html.to_string();
    for tag in &["script", "style", "nav", "header", "footer", "head"] {
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

/// Collect all `.html` files under `dir` (iterative, bounded).
fn collect_html_files(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    let mut stack = vec![dir.to_path_buf()];

    while let Some(current) = stack.pop() {
        if files.len() >= MAX_INDEX_ENTRIES {
            break;
        }
        let entries = fs::read_dir(&current)
            .with_context(|| format!("cannot read {}", current.display()))?;
        for entry in entries {
            let path = entry?.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|e| e == "html") {
                files.push(path);
            }
        }
    }

    Ok(files)
}

/// Inject the search UI script into an HTML file.
///
/// Inserts a `<script>` block before `</body>` that:
/// 1. Loads `search-index.json`
/// 2. Creates a modal overlay with an input field
/// 3. Performs case-insensitive substring matching on title + content
/// 4. Displays results with highlighted snippets
/// 5. Activates on `Ctrl+K` / `Cmd+K`
fn inject_search_ui(path: &Path) -> Result<()> {
    let html = fs::read_to_string(path)
        .with_context(|| format!("cannot read {}", path.display()))?;

    if html.contains("ssg-search-widget") {
        return Ok(()); // Already injected
    }

    let script = SEARCH_WIDGET_SCRIPT;

    let injected = if let Some(pos) = html.rfind("</body>") {
        format!("{}{}{}", &html[..pos], script, &html[pos..])
    } else {
        format!("{html}{script}")
    };

    fs::write(path, injected)
        .with_context(|| format!("cannot write {}", path.display()))?;
    Ok(())
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
#ssg-search-btn{position:fixed;top:16px;right:16px;z-index:9998;display:flex;align-items:center;gap:8px;padding:8px 16px;background:#fff;border:1px solid #d1d5db;border-radius:8px;cursor:pointer;font-family:-apple-system,system-ui,sans-serif;font-size:14px;color:#6b7280;box-shadow:0 1px 3px rgba(0,0,0,.08);transition:border-color .15s,box-shadow .15s}
#ssg-search-btn:hover{border-color:#9ca3af;box-shadow:0 2px 6px rgba(0,0,0,.12)}
#ssg-search-btn svg{width:16px;height:16px;stroke:currentColor;fill:none;stroke-width:2;stroke-linecap:round;stroke-linejoin:round}
#ssg-search-btn kbd{font-family:inherit;font-size:11px;padding:2px 6px;background:#f3f4f6;border:1px solid #e5e7eb;border-radius:4px;color:#9ca3af;margin-left:4px}
/* ── Modal overlay ── */
#ssg-search-overlay{display:none;position:fixed;inset:0;z-index:9999;background:rgba(0,0,0,.5);align-items:flex-start;justify-content:center;padding-top:12vh}
#ssg-search-overlay.active{display:flex}
#ssg-search-box{background:#fff;border-radius:12px;width:92%;max-width:640px;box-shadow:0 25px 60px rgba(0,0,0,.3);overflow:hidden;font-family:-apple-system,system-ui,sans-serif}
#ssg-search-header{display:flex;align-items:center;padding:0 16px;border-bottom:1px solid #e5e7eb}
#ssg-search-header svg{width:20px;height:20px;stroke:#9ca3af;fill:none;stroke-width:2;flex-shrink:0}
#ssg-search-input{flex:1;padding:16px 12px;font-size:16px;border:none;outline:none;background:transparent}
#ssg-search-results{max-height:50vh;overflow-y:auto}
.ssg-result{display:block;padding:12px 20px;text-decoration:none;color:#111;border-bottom:1px solid #f3f4f6;transition:background .1s}
.ssg-result:hover,.ssg-result.active{background:#eff6ff}
.ssg-result-title{font-weight:600;font-size:15px;margin-bottom:3px}
.ssg-result-snippet{font-size:13px;color:#6b7280;line-height:1.5}
.ssg-result-snippet mark{background:#fef08a;color:inherit;border-radius:2px;padding:0 2px}
.ssg-no-results{padding:32px 20px;text-align:center;color:#9ca3af;font-size:14px}
.ssg-search-footer{display:flex;gap:16px;padding:10px 20px;font-size:12px;color:#9ca3af;border-top:1px solid #e5e7eb;justify-content:flex-end}
.ssg-search-footer kbd{font-family:inherit;font-size:11px;padding:1px 5px;background:#f3f4f6;border:1px solid #e5e7eb;border-radius:3px}
/* ── Dark mode (media query + data-theme attribute) ── */
@media(prefers-color-scheme:dark){
:root:not([data-theme="light"]) #ssg-search-btn{background:#1f2937;border-color:#374151;color:#9ca3af}
:root:not([data-theme="light"]) #ssg-search-btn:hover{border-color:#4b5563}
:root:not([data-theme="light"]) #ssg-search-btn kbd{background:#374151;border-color:#4b5563;color:#6b7280}
:root:not([data-theme="light"]) #ssg-search-box{background:#1f2937;color:#f9fafb}
:root:not([data-theme="light"]) #ssg-search-header{border-color:#374151}
:root:not([data-theme="light"]) #ssg-search-input{color:#f9fafb}
:root:not([data-theme="light"]) .ssg-result{color:#f9fafb;border-color:#374151}
:root:not([data-theme="light"]) .ssg-result:hover,:root:not([data-theme="light"]) .ssg-result.active{background:#374151}
:root:not([data-theme="light"]) .ssg-result-snippet{color:#9ca3af}
:root:not([data-theme="light"]) .ssg-result-snippet mark{background:#854d0e;color:#fef08a}
:root:not([data-theme="light"]) .ssg-no-results{color:#6b7280}
:root:not([data-theme="light"]) .ssg-search-footer{border-color:#374151;color:#6b7280}
:root:not([data-theme="light"]) .ssg-search-footer kbd{background:#374151;border-color:#4b5563}
}
[data-theme="dark"] #ssg-search-btn{background:#1f2937;border-color:#374151;color:#9ca3af}
[data-theme="dark"] #ssg-search-btn:hover{border-color:#4b5563}
[data-theme="dark"] #ssg-search-btn kbd{background:#374151;border-color:#4b5563;color:#6b7280}
[data-theme="dark"] #ssg-search-box{background:#1f2937;color:#f9fafb}
[data-theme="dark"] #ssg-search-header{border-color:#374151}
[data-theme="dark"] #ssg-search-input{color:#f9fafb}
[data-theme="dark"] .ssg-result{color:#f9fafb;border-color:#374151}
[data-theme="dark"] .ssg-result:hover,[data-theme="dark"] .ssg-result.active{background:#374151}
[data-theme="dark"] .ssg-result-snippet{color:#9ca3af}
[data-theme="dark"] .ssg-result-snippet mark{background:#854d0e;color:#fef08a}
[data-theme="dark"] .ssg-no-results{color:#6b7280}
[data-theme="dark"] .ssg-search-footer{border-color:#374151;color:#6b7280}
[data-theme="dark"] .ssg-search-footer kbd{background:#374151;border-color:#4b5563}
</style>
<!-- Search trigger button -->
<button id="ssg-search-btn" type="button" aria-label="Search">
<svg viewBox="0 0 24 24"><circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/></svg>
<span>Search</span>
<kbd>K</kbd>
</button>
<!-- Search modal -->
<div id="ssg-search-overlay" role="dialog" aria-label="Search">
<div id="ssg-search-box">
<div id="ssg-search-header">
<svg viewBox="0 0 24 24"><circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/></svg>
<input id="ssg-search-input" type="search" placeholder="Search documentation..." autocomplete="off" aria-label="Search query"/>
</div>
<div id="ssg-search-results"></div>
<div class="ssg-search-footer"><span><kbd>Esc</kbd> close</span><span><kbd>&uarr;</kbd><kbd>&darr;</kbd> navigate</span><span><kbd>Enter</kbd> open</span></div>
</div>
</div>
<script>
(function(){
var idx=null,overlay=document.getElementById('ssg-search-overlay'),
input=document.getElementById('ssg-search-input'),
results=document.getElementById('ssg-search-results'),
btn=document.getElementById('ssg-search-btn'),active=-1;
function load(){if(idx)return Promise.resolve();return fetch('/search-index.json').then(function(r){return r.json()}).then(function(d){idx=d.entries||[]}).catch(function(){idx=[]})}
function open(){load().then(function(){overlay.classList.add('active');input.value='';results.innerHTML='';input.focus();active=-1})}
function close(){overlay.classList.remove('active');active=-1}
function highlight(text,q){if(!q)return esc(text);var re=new RegExp('('+q.replace(/[.*+?^${}()|[\]\\]/g,'\\$&')+')','gi');return esc(text).replace(re,'<mark>$1</mark>')}
function esc(s){var d=document.createElement('div');d.textContent=s;return d.innerHTML}
function snippet(content,q,len){len=len||150;if(!q)return esc(content.substring(0,len));var i=content.toLowerCase().indexOf(q.toLowerCase());if(i<0)return esc(content.substring(0,len));var s=Math.max(0,i-50),e=Math.min(content.length,i+len);var t=(s>0?'...':'')+content.substring(s,e)+(e<content.length?'...':'');return highlight(t,q)}
function search(q){if(!idx||!q){results.innerHTML='';return}q=q.trim();if(!q){results.innerHTML='';return}var ql=q.toLowerCase(),hits=[];
for(var i=0;i<idx.length&&hits.length<20;i++){var e=idx[i],s=0;if(e.title.toLowerCase().indexOf(ql)>=0)s+=10;if(e.content.toLowerCase().indexOf(ql)>=0)s+=5;for(var h=0;h<e.headings.length;h++){if(e.headings[h].toLowerCase().indexOf(ql)>=0){s+=3;break}}if(s>0)hits.push({entry:e,score:s})}
hits.sort(function(a,b){return b.score-a.score});
if(!hits.length){results.innerHTML='<div class="ssg-no-results">No results for &ldquo;'+esc(q)+'&rdquo;</div>';return}
var html='';for(var j=0;j<hits.length;j++){var e=hits[j].entry;html+='<a class="ssg-result" href="'+esc(e.url)+'">'+'<div class="ssg-result-title">'+highlight(e.title,q)+'</div>'+'<div class="ssg-result-snippet">'+snippet(e.content,q)+'</div></a>'}
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

        inject_search_ui(&path)?;

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

        inject_search_ui(&path)?;
        let first = fs::read_to_string(&path)?;

        inject_search_ui(&path)?;
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
        inject_search_ui(&path)?;

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
}

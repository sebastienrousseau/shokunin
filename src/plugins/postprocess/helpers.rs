// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Shared helpers for post-processing plugins.

use anyhow::Result;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Normalise a URL by collapsing double (or more) slashes in the path
/// portion, preserving the `://` in the scheme.
pub(super) fn normalise_url(url: &str) -> String {
    if let Some(scheme_end) = url.find("://") {
        let (scheme, rest) = url.split_at(scheme_end + 3);
        let cleaned: String = rest
            .chars()
            .fold((String::new(), false), |(mut acc, prev_slash), ch| {
                if ch == '/' && prev_slash {
                    (acc, true)
                } else {
                    acc.push(ch);
                    (acc, ch == '/')
                }
            })
            .0;
        format!("{scheme}{cleaned}")
    } else {
        url.to_string()
    }
}

/// Escape XML special characters.
pub(super) fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

/// Truncate a string at a word boundary, appending "..." if truncated.
pub(super) fn truncate_at_word(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        return s.to_string();
    }
    let mut end = max_len;
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    let truncated = &s[..end];
    match truncated.rfind(' ') {
        Some(pos) => format!("{}...", &s[..pos]),
        None => format!("{truncated}..."),
    }
}

/// Pass-through for a `read_dir` entry with a fault-injection hook so
/// tests can exercise the mid-iteration error branch (which no real
/// filesystem produces deterministically).
#[allow(clippy::missing_const_for_fn)] // fail_point! body is non-const
fn sidecar_dir_entry(
    entry: std::io::Result<fs::DirEntry>,
) -> std::io::Result<fs::DirEntry> {
    fail_point!("postprocess::sidecar-entry", |_| Err(
        std::io::Error::other("injected: postprocess::sidecar-entry")
    ));
    entry
}

/// Parse a `.meta.json` sidecar tolerantly: values are usually strings,
/// but the pipeline also emits numeric/bool fields (e.g. `word_count`).
/// Those are coerced to their string form instead of failing the whole
/// sidecar (which would silently drop the page from every feed).
/// `null` values are skipped; nested arrays/objects keep their compact
/// JSON encoding (matching the JSON-encoded-string convention used by
/// e.g. the MCP emitter's `agents` key).
fn parse_meta_sidecar(content: &str) -> Option<HashMap<String, String>> {
    let raw: serde_json::Map<String, serde_json::Value> =
        serde_json::from_str(content).ok()?;
    let mut meta = HashMap::with_capacity(raw.len());
    for (key, value) in raw {
        let coerced = match value {
            serde_json::Value::String(s) => s,
            serde_json::Value::Null => continue,
            other => other.to_string(),
        };
        let _ = meta.insert(key, coerced);
    }
    Some(meta)
}

/// Read `.meta.json` sidecar files from a directory to extract front
/// matter metadata for each page.
pub(super) fn read_meta_sidecars(
    site_dir: &Path,
) -> Result<Vec<(String, HashMap<String, String>)>> {
    let mut entries = Vec::new();
    let mut stack = vec![site_dir.to_path_buf()];
    while let Some(current) = stack.pop() {
        if !current.is_dir() {
            continue;
        }
        for entry in fs::read_dir(&current)? {
            let entry = sidecar_dir_entry(entry)?;
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path
                .file_name()
                .is_some_and(|n| n.to_string_lossy().ends_with(".meta.json"))
            {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Some(meta) = parse_meta_sidecar(&content) {
                        let rel = path
                            .parent()
                            .and_then(|p| p.strip_prefix(site_dir).ok())
                            .map(|p| p.to_string_lossy().into_owned())
                            .unwrap_or_default();
                        entries.push((rel, meta));
                    }
                }
            }
        }
    }
    Ok(entries)
}

/// Parsed components of an RFC 2822 date.
pub(super) struct Rfc2822Date {
    pub year: u32,
    pub month: u32,
    pub day: u32,
    pub hour: u32,
    pub min: u32,
    pub sec: u32,
    pub tz: String,
}

impl Rfc2822Date {
    pub(super) fn to_rfc3339(&self) -> String {
        let tz = if self.tz == "+0000" || self.tz == "GMT" || self.tz == "UTC" {
            "+00:00".to_string()
        } else if self.tz.len() == 5 {
            format!("{}:{}", &self.tz[..3], &self.tz[3..])
        } else {
            self.tz.clone()
        };
        format!(
            "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}{}",
            self.year, self.month, self.day, self.hour, self.min, self.sec, tz
        )
    }
}

/// Parse an RFC 2822 date leniently, ignoring incorrect weekday names.
pub(super) fn parse_rfc2822_lenient(rfc: &str) -> Option<Rfc2822Date> {
    // Strip optional weekday: "Thu, " prefix
    let rest = if let Some(pos) = rfc.find(", ") {
        rfc[pos + 2..].trim()
    } else {
        rfc.trim()
    };
    // Parse: "11 Apr 2026 06:06:06 +0000"
    let parts: Vec<&str> = rest.splitn(5, ' ').collect();
    if parts.len() < 4 {
        return None;
    }
    let day: u32 = parts[0].parse().ok()?;
    let month = match parts[1] {
        "Jan" => 1,
        "Feb" => 2,
        "Mar" => 3,
        "Apr" => 4,
        "May" => 5,
        "Jun" => 6,
        "Jul" => 7,
        "Aug" => 8,
        "Sep" => 9,
        "Oct" => 10,
        "Nov" => 11,
        "Dec" => 12,
        _ => return None,
    };
    let year: u32 = parts[2].parse().ok()?;
    let time_parts: Vec<&str> = parts[3].split(':').collect();
    if time_parts.len() != 3 {
        return None;
    }
    let hour: u32 = time_parts[0].parse().ok()?;
    let min: u32 = time_parts[1].parse().ok()?;
    let sec: u32 = time_parts[2].parse().ok()?;
    let tz = parts.get(4).unwrap_or(&"+0000");
    Some(Rfc2822Date {
        year,
        month,
        day,
        hour,
        min,
        sec,
        tz: tz.to_string(),
    })
}

/// Convert an RFC 2822 date string to ISO 8601 datetime.
pub(super) fn rfc2822_to_iso8601(rfc: &str) -> String {
    parse_rfc2822_lenient(rfc)
        .map_or_else(|| rfc.to_string(), |dt| dt.to_rfc3339())
}

/// Extract the first occurrence of a simple XML element value.
pub(super) fn extract_xml_value(xml: &str, tag: &str) -> Option<String> {
    let open = format!("<{tag}>");
    let close = format!("</{tag}>");
    if let Some(start) = xml.find(&open) {
        let after = &xml[start + open.len()..];
        if let Some(end) = after.find(&close) {
            let value = after[..end].trim().to_string();
            if !value.is_empty() {
                return Some(value);
            }
        }
    }
    None
}

/// Normalise URLs within a single XML line.
pub(super) fn normalise_url_in_xml_line(line: &str) -> String {
    let mut result = line.to_string();
    // Find URL-like patterns (http:// or https://) and normalise path slashes
    let patterns = ["https://", "http://"];
    for pat in &patterns {
        while let Some(start) = result.find(pat) {
            let after_scheme = start + pat.len();
            // Find the end of this URL (next < or whitespace or quote)
            let end = result[after_scheme..]
                .find(|c: char| {
                    c == '<' || c == '"' || c == '\'' || c.is_whitespace()
                })
                .map_or(result.len(), |i| i + after_scheme);
            let url = &result[start..end];
            let fixed = normalise_url(url);
            if fixed == url {
                break;
            }
            result = format!("{}{}{}", &result[..start], fixed, &result[end..]);
        }
    }
    result
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------
    // normalise_url
    // -----------------------------------------------------------------

    #[test]
    fn test_normalise_url_double_slash() {
        assert_eq!(
            normalise_url("https://example.com//index.html"),
            "https://example.com/index.html"
        );
    }

    #[test]
    fn test_normalise_url_preserves_scheme() {
        assert_eq!(
            normalise_url("https://example.com/path/to/file"),
            "https://example.com/path/to/file"
        );
    }

    #[test]
    fn test_normalise_url_multiple_slashes() {
        assert_eq!(
            normalise_url("https://example.com///a//b///c"),
            "https://example.com/a/b/c"
        );
    }

    #[test]
    fn test_normalise_url_no_scheme() {
        assert_eq!(normalise_url("example.com//path"), "example.com//path");
    }

    #[test]
    fn test_normalise_url_trailing_slash() {
        assert_eq!(
            normalise_url("https://example.com/"),
            "https://example.com/"
        );
    }

    #[test]
    fn test_normalise_url_http_scheme() {
        assert_eq!(
            normalise_url("http://example.com//a//b"),
            "http://example.com/a/b"
        );
    }

    // -----------------------------------------------------------------
    // xml_escape
    // -----------------------------------------------------------------

    #[test]
    fn test_xml_escape_ampersand() {
        assert_eq!(xml_escape("a&b"), "a&amp;b");
    }

    #[test]
    fn test_xml_escape_lt() {
        assert_eq!(xml_escape("a<b"), "a&lt;b");
    }

    #[test]
    fn test_xml_escape_gt() {
        assert_eq!(xml_escape("a>b"), "a&gt;b");
    }

    #[test]
    fn test_xml_escape_quot() {
        assert_eq!(xml_escape("a\"b"), "a&quot;b");
    }

    #[test]
    fn test_xml_escape_apos() {
        assert_eq!(xml_escape("a'b"), "a&apos;b");
    }

    #[test]
    fn test_xml_escape_all_combined() {
        assert_eq!(
            xml_escape("<tag attr=\"a&b\" val='c'>"),
            "&lt;tag attr=&quot;a&amp;b&quot; val=&apos;c&apos;&gt;"
        );
    }

    // -----------------------------------------------------------------
    // truncate_at_word
    // -----------------------------------------------------------------

    #[test]
    fn test_truncate_at_word_short() {
        assert_eq!(truncate_at_word("hello", 10), "hello");
    }

    #[test]
    fn test_truncate_at_word_long() {
        let result = truncate_at_word("hello world foo bar", 12);
        assert_eq!(result, "hello world...");
    }

    #[test]
    fn test_truncate_at_word_no_spaces() {
        let result = truncate_at_word("abcdefghij", 5);
        assert_eq!(result, "abcde...");
    }

    #[test]
    fn test_truncate_at_word_exact_length() {
        let result = truncate_at_word("hello", 5);
        assert_eq!(result, "hello");
    }

    // -----------------------------------------------------------------
    // rfc2822_to_iso8601
    // -----------------------------------------------------------------

    #[test]
    fn test_rfc2822_to_iso8601() {
        let result = rfc2822_to_iso8601("Thu, 11 Apr 2026 06:06:06 +0000");
        assert!(result.starts_with("2026-04-11"));
        assert!(result.contains('T'));
    }

    #[test]
    fn test_rfc2822_to_iso8601_passthrough() {
        let input = "2026-04-11";
        assert_eq!(rfc2822_to_iso8601(input), input);
    }

    // -----------------------------------------------------------------
    // Rfc2822Date
    // -----------------------------------------------------------------

    #[test]
    fn test_rfc2822_date_to_rfc3339_utc() {
        let dt = Rfc2822Date {
            year: 2026,
            month: 4,
            day: 11,
            hour: 6,
            min: 6,
            sec: 6,
            tz: "+0000".to_string(),
        };
        assert_eq!(dt.to_rfc3339(), "2026-04-11T06:06:06+00:00");
    }

    #[test]
    fn test_rfc2822_date_to_rfc3339_gmt() {
        let dt = Rfc2822Date {
            year: 2025,
            month: 1,
            day: 15,
            hour: 12,
            min: 0,
            sec: 0,
            tz: "GMT".to_string(),
        };
        assert_eq!(dt.to_rfc3339(), "2025-01-15T12:00:00+00:00");
    }

    #[test]
    fn test_rfc2822_date_to_rfc3339_utc_tz() {
        let dt = Rfc2822Date {
            year: 2025,
            month: 6,
            day: 1,
            hour: 0,
            min: 0,
            sec: 0,
            tz: "UTC".to_string(),
        };
        assert_eq!(dt.to_rfc3339(), "2025-06-01T00:00:00+00:00");
    }

    #[test]
    fn test_rfc2822_date_to_rfc3339_positive_offset() {
        let dt = Rfc2822Date {
            year: 2026,
            month: 12,
            day: 25,
            hour: 18,
            min: 30,
            sec: 45,
            tz: "+0530".to_string(),
        };
        assert_eq!(dt.to_rfc3339(), "2026-12-25T18:30:45+05:30");
    }

    #[test]
    fn test_rfc2822_date_to_rfc3339_negative_offset() {
        let dt = Rfc2822Date {
            year: 2026,
            month: 7,
            day: 4,
            hour: 9,
            min: 15,
            sec: 0,
            tz: "-0700".to_string(),
        };
        assert_eq!(dt.to_rfc3339(), "2026-07-04T09:15:00-07:00");
    }

    #[test]
    fn test_rfc2822_date_to_rfc3339_unknown_tz() {
        let dt = Rfc2822Date {
            year: 2026,
            month: 1,
            day: 1,
            hour: 0,
            min: 0,
            sec: 0,
            tz: "EST".to_string(),
        };
        assert_eq!(dt.to_rfc3339(), "2026-01-01T00:00:00EST");
    }

    // -----------------------------------------------------------------
    // parse_rfc2822_lenient
    // -----------------------------------------------------------------

    #[test]
    fn test_parse_rfc2822_lenient_no_weekday() {
        let dt = parse_rfc2822_lenient("11 Apr 2026 06:06:06 +0000");
        assert!(dt.is_some());
        let dt = dt.unwrap();
        assert_eq!(dt.day, 11);
        assert_eq!(dt.month, 4);
        assert_eq!(dt.year, 2026);
    }

    #[test]
    fn test_parse_rfc2822_lenient_invalid() {
        assert!(parse_rfc2822_lenient("not a date").is_none());
    }

    #[test]
    fn test_parse_rfc2822_lenient_too_few_parts() {
        assert!(parse_rfc2822_lenient("11 Apr").is_none());
    }

    #[test]
    fn test_parse_rfc2822_lenient_bad_month() {
        assert!(parse_rfc2822_lenient("11 Xxx 2026 06:06:06 +0000").is_none());
    }

    #[test]
    fn test_parse_rfc2822_lenient_bad_time() {
        assert!(parse_rfc2822_lenient("11 Apr 2026 06:06 +0000").is_none());
    }

    #[test]
    fn test_parse_rfc2822_lenient_no_tz_defaults() {
        let dt = parse_rfc2822_lenient("11 Apr 2026 06:06:06");
        assert!(dt.is_some());
        let dt = dt.unwrap();
        assert_eq!(dt.tz, "+0000");
    }

    // -----------------------------------------------------------------
    // extract_xml_value
    // -----------------------------------------------------------------

    #[test]
    fn test_extract_xml_value() {
        let xml = "<channel><title>Hello</title><link>https://example.com</link></channel>";
        assert_eq!(extract_xml_value(xml, "title"), Some("Hello".to_string()));
        assert_eq!(
            extract_xml_value(xml, "link"),
            Some("https://example.com".to_string())
        );
        assert_eq!(extract_xml_value(xml, "missing"), None);
    }

    #[test]
    fn test_extract_xml_value_empty_value() {
        let xml = "<title></title>";
        assert_eq!(extract_xml_value(xml, "title"), None);
    }

    #[test]
    fn test_extract_xml_value_whitespace_only() {
        let xml = "<title>   </title>";
        assert_eq!(extract_xml_value(xml, "title"), None);
    }

    // -----------------------------------------------------------------
    // normalise_url_in_xml_line
    // -----------------------------------------------------------------

    #[test]
    fn test_normalise_url_in_xml_line() {
        let line = "  <loc>https://example.com//page//index.html</loc>";
        let result = normalise_url_in_xml_line(line);
        assert_eq!(result, "  <loc>https://example.com/page/index.html</loc>");
    }

    #[test]
    fn test_normalise_url_in_xml_line_no_url() {
        let line = "  <lastmod>2025-09-01</lastmod>";
        let result = normalise_url_in_xml_line(line);
        assert_eq!(result, line, "Non-URL lines should be unchanged");
    }

    // -----------------------------------------------------------------
    // truncate_at_word: UTF-8 boundary backtracking
    // -----------------------------------------------------------------

    #[test]
    fn test_truncate_at_word_backs_up_to_char_boundary() {
        // max_len = 2 lands in the middle of the 2-byte 'é', so the
        // truncation point must back up to the previous boundary.
        let result = truncate_at_word("aé bcd", 2);
        assert_eq!(result, "a...");
    }

    // -----------------------------------------------------------------
    // read_meta_sidecars
    // -----------------------------------------------------------------

    #[test]
    fn test_read_meta_sidecars_non_directory_input() {
        let tmp = tempfile::tempdir().unwrap();
        let file = tmp.path().join("plain.txt");
        fs::write(&file, "not a dir").unwrap();
        let entries = read_meta_sidecars(&file).unwrap();
        assert!(entries.is_empty(), "file input yields no sidecars");
    }

    #[test]
    fn test_read_meta_sidecars_collects_nested_sidecar() {
        let tmp = tempfile::tempdir().unwrap();
        let page = tmp.path().join("blog").join("hello");
        fs::create_dir_all(&page).unwrap();
        fs::write(
            page.join("page.meta.json"),
            r#"{"title":"Hello","description":"D"}"#,
        )
        .unwrap();
        let entries = read_meta_sidecars(tmp.path()).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].0, "blog/hello");
        assert_eq!(
            entries[0].1.get("title").map(String::as_str),
            Some("Hello")
        );
    }

    #[test]
    fn test_read_meta_sidecars_skips_malformed_json() {
        let tmp = tempfile::tempdir().unwrap();
        fs::write(tmp.path().join("bad.meta.json"), "{ not json").unwrap();
        let entries = read_meta_sidecars(tmp.path()).unwrap();
        assert!(entries.is_empty(), "malformed sidecars are skipped");
    }

    #[test]
    #[cfg(unix)]
    fn test_read_meta_sidecars_unreadable_subdir_errors() {
        use std::os::unix::fs::PermissionsExt;
        let tmp = tempfile::tempdir().unwrap();
        let locked = tmp.path().join("locked");
        fs::create_dir_all(&locked).unwrap();
        fs::set_permissions(&locked, fs::Permissions::from_mode(0o000))
            .unwrap();
        let result = read_meta_sidecars(tmp.path());
        // Restore perms so tempdir cleanup works.
        let _ = fs::set_permissions(&locked, fs::Permissions::from_mode(0o755));
        assert!(result.is_err(), "unreadable subdir must error");
    }

    // -----------------------------------------------------------------
    // parse_meta_sidecar: non-string value coercion (issue: numeric
    // word_count sidecar fields must not drop the page)
    // -----------------------------------------------------------------

    #[test]
    fn test_parse_meta_sidecar_coerces_numbers_and_bools() {
        let meta = parse_meta_sidecar(
            r#"{"title":"T","word_count":342,"draft":false}"#,
        )
        .unwrap();
        assert_eq!(meta.get("title").map(String::as_str), Some("T"));
        assert_eq!(meta.get("word_count").map(String::as_str), Some("342"));
        assert_eq!(meta.get("draft").map(String::as_str), Some("false"));
    }

    #[test]
    fn test_parse_meta_sidecar_skips_null_and_encodes_nested() {
        let meta = parse_meta_sidecar(
            r#"{"title":"T","banner":null,"agents":{"disallow":["mcp"]}}"#,
        )
        .unwrap();
        assert!(!meta.contains_key("banner"), "null values are dropped");
        assert_eq!(
            meta.get("agents").map(String::as_str),
            Some(r#"{"disallow":["mcp"]}"#)
        );
    }

    #[test]
    fn test_parse_meta_sidecar_rejects_non_object() {
        assert!(parse_meta_sidecar("[1,2,3]").is_none());
        assert!(parse_meta_sidecar("nope").is_none());
    }

    // -----------------------------------------------------------------
    // parse_rfc2822_lenient: remaining month arms + per-field failures
    // -----------------------------------------------------------------

    #[test]
    fn test_parse_rfc2822_lenient_all_months() {
        let months = [
            ("Jan", 1),
            ("Feb", 2),
            ("Mar", 3),
            ("Apr", 4),
            ("May", 5),
            ("Jun", 6),
            ("Jul", 7),
            ("Aug", 8),
            ("Sep", 9),
            ("Oct", 10),
            ("Nov", 11),
            ("Dec", 12),
        ];
        for (name, number) in months {
            let input = format!("11 {name} 2026 06:06:06 +0000");
            let dt = parse_rfc2822_lenient(&input).unwrap();
            assert_eq!(dt.month, number, "month {name}");
        }
    }

    #[test]
    fn test_parse_rfc2822_lenient_bad_day() {
        assert!(parse_rfc2822_lenient("xx Apr 2026 06:06:06 +0000").is_none());
    }

    #[test]
    fn test_parse_rfc2822_lenient_bad_year() {
        assert!(parse_rfc2822_lenient("11 Apr 20x6 06:06:06 +0000").is_none());
    }

    #[test]
    fn test_parse_rfc2822_lenient_bad_hour_min_sec() {
        assert!(parse_rfc2822_lenient("11 Apr 2026 xx:06:06 +0000").is_none());
        assert!(parse_rfc2822_lenient("11 Apr 2026 06:xx:06 +0000").is_none());
        assert!(parse_rfc2822_lenient("11 Apr 2026 06:06:xx +0000").is_none());
    }

    // -----------------------------------------------------------------
    // extract_xml_value: open tag without a closing tag
    // -----------------------------------------------------------------

    #[test]
    fn test_extract_xml_value_unclosed_tag() {
        assert_eq!(extract_xml_value("<title>abc", "title"), None);
    }
}

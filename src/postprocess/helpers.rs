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
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path
                .file_name()
                .is_some_and(|n| n.to_string_lossy().ends_with(".meta.json"))
            {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(meta) = serde_json::from_str::<
                        HashMap<String, String>,
                    >(&content)
                    {
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
    pub(super) fn to_iso_date(&self) -> String {
        format!("{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }

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

/// Convert an RFC 2822 date string to ISO 8601 date (YYYY-MM-DD).
///
/// Tolerates incorrect weekday names (common in generated feeds) by
/// stripping the leading "Day, " prefix and parsing the remainder.
pub(super) fn rfc2822_to_iso_date(rfc: &str) -> Option<String> {
    parse_rfc2822_lenient(rfc).map(|dt| dt.to_iso_date())
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
    // rfc2822_to_iso_date / rfc2822_to_iso8601
    // -----------------------------------------------------------------

    #[test]
    fn test_rfc2822_to_iso_date() {
        let result = rfc2822_to_iso_date("Thu, 11 Apr 2026 06:06:06 +0000");
        assert_eq!(result, Some("2026-04-11".to_string()));
    }

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
    fn test_rfc2822_date_to_iso_date() {
        let dt = Rfc2822Date {
            year: 2026,
            month: 4,
            day: 11,
            hour: 6,
            min: 6,
            sec: 6,
            tz: "+0000".to_string(),
        };
        assert_eq!(dt.to_iso_date(), "2026-04-11");
    }

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
}

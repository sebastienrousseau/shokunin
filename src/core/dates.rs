// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Flexible, dependency-free date parsing shared by the feed and
//! sitemap post-processing plugins.
//!
//! Issue #586 / v0.0.47 plan §2 item 1.4 (spec A4): the native
//! `rss.rs`, `atom.rs`, `json_feed.rs`, `news_sitemap.rs`, and
//! `sitemap.rs` plugins each used to parse dates independently, and
//! only understood RFC 2822. Front matter written as `July 1, 2026`
//! or `2026-07-01` fell through and produced the upstream
//! `'day' component could not be parsed` warning spam. This module
//! is the single parsing chain they all share.
//!
//! [`parse_flexible_date`] accepts, in priority order:
//!
//! 1. **RFC 2822** — `Wed, 01 Jul 2026 07:07:07 +0000` (weekday
//!    optional and *not* verified, matching the lenient behaviour the
//!    plugins already relied on; seconds optional; named zones from
//!    the RFC 2822 obsolete table accepted).
//! 2. **Long form** — `July 1, 2026` and `1 July 2026` (full month
//!    names or three-letter abbreviations; midnight UTC assumed).
//! 3. **ISO 8601 date** — `2026-07-01` (midnight UTC assumed).
//! 4. **ISO 8601 datetime** — `2026-07-01T07:07:07Z`, with `Z`,
//!    `±hh:mm`, or `±hhmm` offsets (UTC assumed when absent).
//!
//! Everything is hand-rolled — month-name tables, leap-year aware
//! day validation, timezone offset parsing — so no new dependency is
//! introduced. Parsing is fully deterministic: no locale lookups and
//! no system-time reads.
//!
//! The output side offers the exact shapes the plugins emit today:
//! [`FlexibleDate::to_rfc2822`] for RSS `<pubDate>`,
//! [`FlexibleDate::to_rfc3339`] / [`FlexibleDate::to_w3c_date`] for
//! Atom `<updated>` and the news-sitemap `<news:publication_date>`,
//! and [`FlexibleDate::to_iso_date`] for sitemap `<lastmod>`.

use std::fmt;

/// The formats attempted by [`parse_flexible_date`], in priority order.
///
/// Referenced by [`DateParseError`] so call sites can log exactly what
/// was tried (plan §2 item 1.4: "log which field/format failed").
///
/// # Examples
///
/// ```rust
/// use ssg::dates::ATTEMPTED_FORMATS;
///
/// assert_eq!(ATTEMPTED_FORMATS.len(), 4);
/// assert!(ATTEMPTED_FORMATS[0].starts_with("RFC 2822"));
/// ```
pub const ATTEMPTED_FORMATS: [&str; 4] = [
    "RFC 2822 (e.g. `Wed, 01 Jul 2026 07:07:07 +0000`)",
    "long form (e.g. `July 1, 2026` or `1 July 2026`)",
    "ISO 8601 date (e.g. `2026-07-01`)",
    "ISO 8601 datetime (e.g. `2026-07-01T07:07:07Z`)",
];

/// Three-letter month abbreviations used for RFC 2822 output.
const MONTH_ABBR: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct",
    "Nov", "Dec",
];

/// Full month names (lowercase) used for input matching.
const MONTH_FULL: [&str; 12] = [
    "january",
    "february",
    "march",
    "april",
    "may",
    "june",
    "july",
    "august",
    "september",
    "october",
    "november",
    "december",
];

/// The input format that [`parse_flexible_date`] matched.
///
/// Callers that must preserve their current output byte-for-byte
/// (e.g. the RSS plugin passes RFC 2822 strings through verbatim,
/// wrong weekday and all) can branch on this instead of re-formatting.
///
/// # Examples
///
/// ```rust
/// use ssg::dates::{parse_flexible_date, DateFormat};
///
/// let dt = parse_flexible_date("2026-07-01").unwrap();
/// assert_eq!(dt.format, DateFormat::IsoDate);
///
/// let dt = parse_flexible_date("July 1, 2026").unwrap();
/// assert_eq!(dt.format, DateFormat::LongForm);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DateFormat {
    /// RFC 2822, e.g. `Wed, 01 Jul 2026 07:07:07 +0000`.
    Rfc2822,
    /// Long form, e.g. `July 1, 2026` or `1 July 2026`.
    LongForm,
    /// ISO 8601 calendar date, e.g. `2026-07-01`.
    IsoDate,
    /// ISO 8601 datetime, e.g. `2026-07-01T07:07:07Z`.
    IsoDateTime,
}

/// A parsed calendar date-time with a fixed UTC offset.
///
/// This is deliberately a plain component struct (spec A4 allows "your
/// own small struct or components") rather than a wrapper around a
/// date crate — the project pins its dependency set and the feed
/// plugins only need formatting, not arithmetic.
///
/// # Examples
///
/// ```rust
/// use ssg::dates::parse_flexible_date;
///
/// let dt = parse_flexible_date("2026-07-01T07:07:07Z").unwrap();
/// assert_eq!((dt.year, dt.month, dt.day), (2026, 7, 1));
/// assert_eq!((dt.hour, dt.minute, dt.second), (7, 7, 7));
/// assert_eq!(dt.offset_minutes, 0);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FlexibleDate {
    /// Calendar year (1–9999).
    pub year: i32,
    /// Calendar month (1–12).
    pub month: u8,
    /// Day of month (1–31, leap-year validated at parse time).
    pub day: u8,
    /// Hour (0–23).
    pub hour: u8,
    /// Minute (0–59).
    pub minute: u8,
    /// Second (0–59).
    pub second: u8,
    /// Offset from UTC in minutes (e.g. `+0530` is `330`).
    pub offset_minutes: i32,
    /// Which input format matched during parsing.
    pub format: DateFormat,
}

impl FlexibleDate {
    /// Format as RFC 2822 for RSS `<pubDate>` /
    /// `<lastBuildDate>`: `Wed, 01 Jul 2026 07:07:07 +0000`.
    ///
    /// The weekday is *computed* (Sakamoto's algorithm), so an input
    /// carrying a wrong weekday name is corrected on the way out.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ssg::dates::parse_flexible_date;
    ///
    /// // "Mon" is wrong — 2026-07-01 is a Wednesday; output corrects it.
    /// let dt = parse_flexible_date("Mon, 01 Jul 2026 07:07:07 +0000").unwrap();
    /// assert_eq!(dt.to_rfc2822(), "Wed, 01 Jul 2026 07:07:07 +0000");
    /// ```
    #[must_use]
    pub fn to_rfc2822(&self) -> String {
        let month = MONTH_ABBR
            .get(usize::from(self.month.saturating_sub(1)))
            .copied()
            .unwrap_or(MONTH_ABBR[0]);
        format!(
            "{}, {:02} {} {:04} {:02}:{:02}:{:02} {}",
            self.weekday_abbr(),
            self.day,
            month,
            self.year,
            self.hour,
            self.minute,
            self.second,
            self.offset_string("")
        )
    }

    /// Format as RFC 3339 / ISO 8601 datetime for Atom
    /// `<updated>`/`<published>` and JSON Feed `date_published`:
    /// `2026-07-01T07:07:07+00:00`.
    ///
    /// UTC is rendered as `+00:00` (not `Z`) to match the output the
    /// plugins have always produced — golden feed fixtures assert the
    /// numeric form.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ssg::dates::parse_flexible_date;
    ///
    /// let dt = parse_flexible_date("2026-07-01T07:07:07+05:30").unwrap();
    /// assert_eq!(dt.to_rfc3339(), "2026-07-01T07:07:07+05:30");
    ///
    /// // Bare dates render as midnight UTC in the numeric form.
    /// let dt = parse_flexible_date("2026-07-01").unwrap();
    /// assert_eq!(dt.to_rfc3339(), "2026-07-01T00:00:00+00:00");
    /// ```
    #[must_use]
    pub fn to_rfc3339(&self) -> String {
        format!(
            "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}{}",
            self.year,
            self.month,
            self.day,
            self.hour,
            self.minute,
            self.second,
            self.offset_string(":")
        )
    }

    /// Format as a W3C datetime for the news sitemap
    /// `<news:publication_date>`. Identical to [`Self::to_rfc3339`]
    /// (the W3C datetime profile of ISO 8601 is what Google News
    /// accepts); provided under its spec name so call sites read like
    /// the sitemap spec.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ssg::dates::parse_flexible_date;
    ///
    /// let dt = parse_flexible_date("July 1, 2026").unwrap();
    /// assert_eq!(dt.to_w3c_date(), dt.to_rfc3339());
    /// assert_eq!(dt.to_w3c_date(), "2026-07-01T00:00:00+00:00");
    /// ```
    #[must_use]
    pub fn to_w3c_date(&self) -> String {
        self.to_rfc3339()
    }

    /// Format as a bare ISO 8601 calendar date for sitemap
    /// `<lastmod>`: `2026-07-01`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ssg::dates::parse_flexible_date;
    ///
    /// let dt = parse_flexible_date("Wed, 01 Jul 2026 07:07:07 +0000").unwrap();
    /// assert_eq!(dt.to_iso_date(), "2026-07-01");
    /// ```
    #[must_use]
    pub fn to_iso_date(&self) -> String {
        format!("{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }

    /// Three-letter weekday abbreviation via Sakamoto's algorithm.
    fn weekday_abbr(&self) -> &'static str {
        const OFFSETS: [i64; 12] = [0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
        const NAMES: [&str; 7] =
            ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
        let mut y = i64::from(self.year);
        if self.month < 3 {
            y -= 1;
        }
        let month_idx = usize::from(self.month.saturating_sub(1)) % 12;
        let idx = (y + y.div_euclid(4) - y.div_euclid(100)
            + y.div_euclid(400)
            + OFFSETS[month_idx]
            + i64::from(self.day))
        .rem_euclid(7) as usize;
        NAMES[idx]
    }

    /// Render the UTC offset with the given hour/minute separator
    /// (`""` for RFC 2822 `+0000`, `":"` for RFC 3339 `+00:00`).
    fn offset_string(&self, sep: &str) -> String {
        let sign = if self.offset_minutes < 0 { '-' } else { '+' };
        let abs = self.offset_minutes.unsigned_abs();
        format!("{sign}{:02}{sep}{:02}", abs / 60, abs % 60)
    }
}

/// Error for input that matched none of the supported date formats.
///
/// Returned by [`parse_flexible_date`]. Its `Display` output names
/// every attempted format so call-site `log::warn!` lines say exactly
/// what was tried and on which value (plan §2 item 1.4).
///
/// # Examples
///
/// ```rust
/// use ssg::dates::parse_flexible_date;
///
/// let err = parse_flexible_date("not a date").unwrap_err();
/// let msg = err.to_string();
/// assert!(msg.contains("not a date"));
/// assert!(msg.contains("attempted formats"));
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DateParseError {
    /// The rejected input (truncated to 64 chars for log hygiene).
    input: String,
}

impl DateParseError {
    /// Build an error for the given rejected input.
    fn new(input: &str) -> Self {
        let mut owned: String = input.chars().take(64).collect();
        if owned.len() < input.len() {
            owned.push('…');
        }
        Self { input: owned }
    }

    /// The formats that were attempted, in priority order.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ssg::dates::{parse_flexible_date, ATTEMPTED_FORMATS};
    ///
    /// let err = parse_flexible_date("nope").unwrap_err();
    /// assert_eq!(err.attempted_formats(), &ATTEMPTED_FORMATS);
    /// ```
    #[must_use]
    pub const fn attempted_formats(&self) -> &'static [&'static str] {
        &ATTEMPTED_FORMATS
    }

    /// The rejected input value (possibly truncated).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ssg::dates::parse_flexible_date;
    ///
    /// let err = parse_flexible_date("mystery value").unwrap_err();
    /// assert_eq!(err.input(), "mystery value");
    /// ```
    #[must_use]
    pub fn input(&self) -> &str {
        &self.input
    }
}

impl fmt::Display for DateParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "could not parse date {:?}; attempted formats: {}",
            self.input,
            ATTEMPTED_FORMATS.join(", ")
        )
    }
}

impl std::error::Error for DateParseError {}

/// `true` for Gregorian leap years (divisible by 4, except centuries
/// not divisible by 400).
///
/// # Examples
///
/// ```rust
/// use ssg::dates::is_leap_year;
///
/// assert!(is_leap_year(2024));
/// assert!(is_leap_year(2000)); // century divisible by 400
/// assert!(!is_leap_year(1900)); // century not divisible by 400
/// assert!(!is_leap_year(2026));
/// ```
#[must_use]
pub const fn is_leap_year(year: i32) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}

/// Number of days in the given month of the given year (leap-year
/// aware). Returns 0 for an out-of-range month.
///
/// # Examples
///
/// ```rust
/// use ssg::dates::days_in_month;
///
/// assert_eq!(days_in_month(2026, 7), 31);
/// assert_eq!(days_in_month(2024, 2), 29); // leap February
/// assert_eq!(days_in_month(2026, 2), 28);
/// assert_eq!(days_in_month(2026, 13), 0); // out of range
/// ```
#[must_use]
pub const fn days_in_month(year: i32, month: u8) -> u8 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap_year(year) {
                29
            } else {
                28
            }
        }
        _ => 0,
    }
}

/// Parse a date string, trying RFC 2822, long form, ISO 8601 date,
/// then ISO 8601 datetime (spec A4 priority order).
///
/// Deterministic and locale-independent: month names come from fixed
/// English tables and no system time is read.
///
/// # Examples
///
/// ```rust
/// use ssg::dates::parse_flexible_date;
///
/// let dt = parse_flexible_date("Wed, 01 Jul 2026 07:07:07 +0000")
///     .expect("RFC 2822 parses");
/// assert_eq!(dt.to_rfc3339(), "2026-07-01T07:07:07+00:00");
///
/// let dt = parse_flexible_date("July 1, 2026").expect("long form parses");
/// assert_eq!(dt.to_rfc2822(), "Wed, 01 Jul 2026 00:00:00 +0000");
///
/// let dt = parse_flexible_date("2026-07-01").expect("ISO date parses");
/// assert_eq!(dt.to_iso_date(), "2026-07-01");
///
/// assert!(parse_flexible_date("not a date").is_err());
/// ```
pub fn parse_flexible_date(
    input: &str,
) -> Result<FlexibleDate, DateParseError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(DateParseError::new(input));
    }
    parse_rfc2822(trimmed)
        .or_else(|| parse_long_form(trimmed))
        .or_else(|| parse_iso8601(trimmed))
        .ok_or_else(|| DateParseError::new(input))
}

/// Validate components and assemble a [`FlexibleDate`].
#[allow(clippy::too_many_arguments)]
fn build_date(
    year: i32,
    month: u8,
    day: u8,
    hour: u8,
    minute: u8,
    second: u8,
    offset_minutes: i32,
    format: DateFormat,
) -> Option<FlexibleDate> {
    if !(1..=9999).contains(&year) {
        return None;
    }
    if month == 0 || month > 12 {
        return None;
    }
    if day == 0 || day > days_in_month(year, month) {
        return None;
    }
    if hour > 23 || minute > 59 || second > 59 {
        return None;
    }
    Some(FlexibleDate {
        year,
        month,
        day,
        hour,
        minute,
        second,
        offset_minutes,
        format,
    })
}

/// Map a month name (`Jul`, `July`, case-insensitive) to 1–12.
fn month_from_name(name: &str) -> Option<u8> {
    let lower = name.to_ascii_lowercase();
    MONTH_FULL
        .iter()
        .position(|full| {
            *full == lower || (lower.len() == 3 && full.starts_with(&lower))
        })
        .map(|idx| idx as u8 + 1)
}

/// Parse a `hh:mm[:ss]` time token. Seconds default to 0 (RFC 2822
/// permits omitting them).
fn parse_hms(token: &str) -> Option<(u8, u8, u8)> {
    let mut parts = token.split(':');
    // `split` always yields a first item, so the empty-string fallback
    // is purely defensive: "" fails the numeric parse just below.
    let hour: u8 = parts.next().unwrap_or_default().parse().ok()?;
    let minute: u8 = parts.next()?.parse().ok()?;
    let second: u8 = match parts.next() {
        Some(sec) => sec.parse().ok()?,
        None => 0,
    };
    if parts.next().is_some() {
        return None;
    }
    Some((hour, minute, second))
}

/// Parse a timezone token into minutes east of UTC. Accepts `Z`,
/// `±hhmm`, `±hh:mm`, and the RFC 2822 obsolete named zones.
fn parse_zone(token: &str) -> Option<i32> {
    match token.to_ascii_uppercase().as_str() {
        "Z" | "UT" | "GMT" | "UTC" => return Some(0),
        // RFC 2822 §4.3 obsolete zone names.
        "EST" => return Some(-5 * 60),
        "EDT" => return Some(-4 * 60),
        "CST" => return Some(-6 * 60),
        "CDT" => return Some(-5 * 60),
        "MST" => return Some(-7 * 60),
        "MDT" => return Some(-6 * 60),
        "PST" => return Some(-8 * 60),
        "PDT" => return Some(-7 * 60),
        _ => {}
    }
    let bytes = token.as_bytes();
    let sign = match bytes.first()? {
        b'+' => 1,
        b'-' => -1,
        _ => return None,
    };
    let digits = &bytes[1..];
    let (hh, mm) = match digits {
        // ±hhmm (RFC 2822 / compact RFC 3339)
        [h1, h2, m1, m2] => (ascii_pair(*h1, *h2)?, ascii_pair(*m1, *m2)?),
        // ±hh:mm (RFC 3339)
        [h1, h2, b':', m1, m2] => {
            (ascii_pair(*h1, *h2)?, ascii_pair(*m1, *m2)?)
        }
        _ => return None,
    };
    if hh > 23 || mm > 59 {
        return None;
    }
    Some(sign * (i32::from(hh) * 60 + i32::from(mm)))
}

/// Combine two ASCII digit bytes into a number.
const fn ascii_pair(first: u8, second: u8) -> Option<u8> {
    if first.is_ascii_digit() && second.is_ascii_digit() {
        Some((first - b'0') * 10 + (second - b'0'))
    } else {
        None
    }
}

/// Parse a run of ASCII digit bytes into a u32.
fn ascii_number(bytes: &[u8]) -> Option<u32> {
    if bytes.is_empty() {
        return None;
    }
    bytes.iter().try_fold(0u32, |acc, b| {
        if b.is_ascii_digit() {
            acc.checked_mul(10)?.checked_add(u32::from(b - b'0'))
        } else {
            None
        }
    })
}

/// Lenient RFC 2822: `[Www, ]DD Mon YYYY hh:mm[:ss] [zone]`.
///
/// The weekday name is stripped without verification — generated
/// feeds routinely carry the wrong weekday and the previous per-plugin
/// parser already tolerated that.
fn parse_rfc2822(input: &str) -> Option<FlexibleDate> {
    // Strip an optional "Www," weekday prefix (letters only, so the
    // comma in long-form "July 1, 2026" never matches).
    let rest = match input.split_once(',') {
        Some((weekday, tail))
            if !weekday.is_empty()
                && weekday.len() <= 9
                && weekday.chars().all(|c| c.is_ascii_alphabetic()) =>
        {
            tail.trim_start()
        }
        _ => input,
    };
    let tokens: Vec<&str> = rest.split_whitespace().collect();
    if !(4..=5).contains(&tokens.len()) {
        return None;
    }
    let day: u8 = tokens[0].parse().ok()?;
    let month = month_from_name(tokens[1])?;
    if tokens[2].len() != 4 {
        return None;
    }
    let year = ascii_number(tokens[2].as_bytes())? as i32;
    let (hour, minute, second) = parse_hms(tokens[3])?;
    let offset = match tokens.get(4) {
        Some(zone) => parse_zone(zone)?,
        None => 0,
    };
    build_date(
        year,
        month,
        day,
        hour,
        minute,
        second,
        offset,
        DateFormat::Rfc2822,
    )
}

/// Long form: `Month D[,] YYYY` or `D Month[,] YYYY`. Midnight UTC.
fn parse_long_form(input: &str) -> Option<FlexibleDate> {
    let cleaned = input.replace(',', " ");
    let tokens: Vec<&str> = cleaned.split_whitespace().collect();
    if tokens.len() != 3 {
        return None;
    }
    // "July 1, 2026" (month first) or "1 July 2026" (day first); `None` if
    // neither leading token names a month.
    let (month, day_token) = month_from_name(tokens[0])
        .map(|m| (m, tokens[1]))
        .or_else(|| month_from_name(tokens[1]).map(|m| (m, tokens[0])))?;
    if day_token.len() > 2 {
        return None;
    }
    let day: u8 = day_token.parse().ok()?;
    if tokens[2].len() != 4 {
        return None;
    }
    let year = ascii_number(tokens[2].as_bytes())? as i32;
    build_date(year, month, day, 0, 0, 0, 0, DateFormat::LongForm)
}

/// ISO 8601: `YYYY-MM-DD` alone, or followed by
/// `[T ]hh:mm[:ss][.frac][Z|±hh:mm|±hhmm]`.
fn parse_iso8601(input: &str) -> Option<FlexibleDate> {
    let bytes = input.as_bytes();
    if bytes.len() < 10 || bytes[4] != b'-' || bytes[7] != b'-' {
        return None;
    }
    let year = ascii_number(&bytes[0..4])? as i32;
    let month = ascii_number(&bytes[5..7])? as u8;
    let day = ascii_number(&bytes[8..10])? as u8;
    if bytes.len() == 10 {
        return build_date(year, month, day, 0, 0, 0, 0, DateFormat::IsoDate);
    }
    if !matches!(bytes[10], b'T' | b't' | b' ') {
        return None;
    }
    // `bytes[10]` is ASCII (checked above), so byte 11 is always a
    // char boundary; the fallback is purely defensive and produces an
    // empty clock that fails `parse_hms` below.
    let rest = input.get(11..).unwrap_or("");
    // The zone (if any) starts at the first Z/z/+/- after the time.
    // `find` returns a char-boundary index, so `split_at` cannot panic.
    let (time_part, zone_part) = match rest.find(['Z', 'z', '+', '-']) {
        Some(pos) => rest.split_at(pos),
        None => (rest, ""),
    };
    // Split off (and validate, but ignore) fractional seconds.
    let clock = match time_part.split_once('.') {
        Some((clock, frac)) => {
            if frac.is_empty() || !frac.bytes().all(|b| b.is_ascii_digit()) {
                return None;
            }
            clock
        }
        None => time_part,
    };
    let (hour, minute, second) = parse_hms(clock)?;
    let offset = if zone_part.is_empty() {
        0
    } else {
        parse_zone(zone_part)?
    };
    build_date(
        year,
        month,
        day,
        hour,
        minute,
        second,
        offset,
        DateFormat::IsoDateTime,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------
    // RFC 2822
    // -----------------------------------------------------------------

    #[test]
    fn rfc2822_spec_example() {
        let dt = parse_flexible_date("Wed, 01 Jul 2026 07:07:07 +0000")
            .expect("spec A4 example parses");
        assert_eq!(dt.format, DateFormat::Rfc2822);
        assert_eq!(dt.to_rfc3339(), "2026-07-01T07:07:07+00:00");
        assert_eq!(dt.to_rfc2822(), "Wed, 01 Jul 2026 07:07:07 +0000");
        assert_eq!(dt.to_iso_date(), "2026-07-01");
    }

    #[test]
    fn rfc2822_wrong_weekday_is_tolerated() {
        // 2026-04-11 is a Saturday; feeds label it Thursday. The
        // legacy per-plugin parser ignored the weekday — so do we.
        let dt = parse_flexible_date("Thu, 11 Apr 2026 06:06:06 +0000")
            .expect("wrong weekday still parses");
        assert_eq!(dt.to_rfc3339(), "2026-04-11T06:06:06+00:00");
        // ...and re-formatting computes the *correct* weekday.
        assert_eq!(dt.to_rfc2822(), "Sat, 11 Apr 2026 06:06:06 +0000");
    }

    #[test]
    fn rfc2822_no_weekday_no_zone() {
        let dt = parse_flexible_date("11 Apr 2026 06:06:06")
            .expect("weekday and zone are optional");
        assert_eq!(dt.offset_minutes, 0);
        assert_eq!(dt.to_rfc3339(), "2026-04-11T06:06:06+00:00");
    }

    #[test]
    fn rfc2822_optional_seconds() {
        let dt = parse_flexible_date("11 Apr 2026 06:06 +0000")
            .expect("RFC 2822 seconds are optional");
        assert_eq!(dt.second, 0);
    }

    #[test]
    fn rfc2822_positive_hhmm_offset() {
        let dt = parse_flexible_date("Fri, 25 Dec 2026 18:30:45 +0530")
            .expect("+hhmm offset parses");
        assert_eq!(dt.offset_minutes, 330);
        assert_eq!(dt.to_rfc3339(), "2026-12-25T18:30:45+05:30");
        assert_eq!(dt.to_rfc2822(), "Fri, 25 Dec 2026 18:30:45 +0530");
    }

    #[test]
    fn rfc2822_negative_hhmm_offset() {
        let dt = parse_flexible_date("Sat, 04 Jul 2026 09:15:00 -0700")
            .expect("-hhmm offset parses");
        assert_eq!(dt.offset_minutes, -420);
        assert_eq!(dt.to_rfc3339(), "2026-07-04T09:15:00-07:00");
    }

    #[test]
    fn rfc2822_named_zones() {
        let gmt = parse_flexible_date("11 Apr 2026 06:06:06 GMT").unwrap();
        assert_eq!(gmt.offset_minutes, 0);
        let est = parse_flexible_date("11 Apr 2026 06:06:06 EST").unwrap();
        assert_eq!(est.offset_minutes, -300);
        assert!(parse_flexible_date("11 Apr 2026 06:06:06 XYZ").is_err());
    }

    #[test]
    fn rfc2822_full_month_name_with_time() {
        let dt = parse_flexible_date("1 July 2026 06:06:06 +0000")
            .expect("full month name with time parses");
        assert_eq!(dt.month, 7);
        assert_eq!(dt.format, DateFormat::Rfc2822);
    }

    // -----------------------------------------------------------------
    // Long form
    // -----------------------------------------------------------------

    #[test]
    fn long_form_month_first_single_digit_day() {
        let dt = parse_flexible_date("July 1, 2026")
            .expect("spec A4 long form parses");
        assert_eq!(dt.format, DateFormat::LongForm);
        assert_eq!((dt.year, dt.month, dt.day), (2026, 7, 1));
        assert_eq!(dt.to_rfc2822(), "Wed, 01 Jul 2026 00:00:00 +0000");
        assert_eq!(dt.to_w3c_date(), "2026-07-01T00:00:00+00:00");
    }

    #[test]
    fn long_form_day_first() {
        let dt = parse_flexible_date("1 July 2026")
            .expect("day-first long form parses");
        assert_eq!((dt.year, dt.month, dt.day), (2026, 7, 1));
    }

    #[test]
    fn long_form_case_insensitive_and_no_comma() {
        let dt = parse_flexible_date("december 25 2026").unwrap();
        assert_eq!((dt.month, dt.day), (12, 25));
    }

    #[test]
    fn long_form_rejects_bad_month_and_day() {
        assert!(parse_flexible_date("Juvember 1, 2026").is_err());
        assert!(parse_flexible_date("July 32, 2026").is_err());
        assert!(parse_flexible_date("July 0, 2026").is_err());
    }

    // -----------------------------------------------------------------
    // ISO 8601
    // -----------------------------------------------------------------

    #[test]
    fn iso_date_only() {
        let dt = parse_flexible_date("2026-07-01").expect("ISO date parses");
        assert_eq!(dt.format, DateFormat::IsoDate);
        assert_eq!(dt.to_rfc3339(), "2026-07-01T00:00:00+00:00");
        assert_eq!(dt.to_iso_date(), "2026-07-01");
    }

    #[test]
    fn iso_datetime_zulu() {
        let dt = parse_flexible_date("2026-07-01T07:07:07Z")
            .expect("Z-suffixed datetime parses");
        assert_eq!(dt.format, DateFormat::IsoDateTime);
        assert_eq!(dt.to_rfc3339(), "2026-07-01T07:07:07+00:00");
    }

    #[test]
    fn iso_datetime_with_colon_offset() {
        let dt = parse_flexible_date("2026-07-01T07:07:07+05:30").unwrap();
        assert_eq!(dt.offset_minutes, 330);
        let dt = parse_flexible_date("2026-07-01T07:07:07-07:00").unwrap();
        assert_eq!(dt.offset_minutes, -420);
    }

    #[test]
    fn iso_datetime_with_compact_offset() {
        let dt = parse_flexible_date("2026-07-01T07:07:07+0530").unwrap();
        assert_eq!(dt.offset_minutes, 330);
    }

    #[test]
    fn iso_datetime_fractional_seconds_ignored() {
        let dt = parse_flexible_date("2026-07-01T07:07:07.123Z").unwrap();
        assert_eq!(dt.second, 7);
        assert!(parse_flexible_date("2026-07-01T07:07:07.abcZ").is_err());
    }

    #[test]
    fn iso_datetime_without_offset_defaults_utc() {
        let dt = parse_flexible_date("2026-07-01T07:07:07").unwrap();
        assert_eq!(dt.offset_minutes, 0);
    }

    #[test]
    fn iso_rejects_invalid_components() {
        assert!(parse_flexible_date("2026-13-01").is_err());
        assert!(parse_flexible_date("2026-00-01").is_err());
        assert!(parse_flexible_date("2026-04-31").is_err());
        assert!(parse_flexible_date("2026-07-01T24:00:00").is_err());
        assert!(parse_flexible_date("2026-07-01T07:60:00").is_err());
        assert!(parse_flexible_date("2026-07-01T07:07:07+2500").is_err());
    }

    // -----------------------------------------------------------------
    // Leap years
    // -----------------------------------------------------------------

    #[test]
    fn leap_day_validation() {
        assert!(parse_flexible_date("2024-02-29").is_ok());
        assert!(parse_flexible_date("2023-02-29").is_err());
        // Century rules: 2000 is a leap year, 2100 is not.
        assert!(parse_flexible_date("2000-02-29").is_ok());
        assert!(parse_flexible_date("2100-02-29").is_err());
        assert!(parse_flexible_date("Thu, 29 Feb 2024 12:00:00 +0000").is_ok());
        assert!(parse_flexible_date("February 29, 2024").is_ok());
        assert!(parse_flexible_date("February 29, 2023").is_err());
    }

    #[test]
    fn days_in_month_table() {
        assert_eq!(days_in_month(2026, 1), 31);
        assert_eq!(days_in_month(2026, 4), 30);
        assert_eq!(days_in_month(2024, 2), 29);
        assert_eq!(days_in_month(2025, 2), 28);
        assert_eq!(days_in_month(2026, 13), 0);
        assert!(is_leap_year(2000));
        assert!(!is_leap_year(1900));
    }

    // -----------------------------------------------------------------
    // Garbage and error reporting
    // -----------------------------------------------------------------

    #[test]
    fn garbage_input_is_err() {
        for garbage in [
            "",
            "   ",
            "not a date",
            "yesterday",
            "13/01/2026",
            "2026",
            "--",
        ] {
            assert!(
                parse_flexible_date(garbage).is_err(),
                "{garbage:?} should not parse"
            );
        }
    }

    #[test]
    fn error_names_all_attempted_formats() {
        let err = parse_flexible_date("not a date").unwrap_err();
        assert_eq!(err.input(), "not a date");
        assert_eq!(err.attempted_formats().len(), 4);
        let msg = err.to_string();
        assert!(msg.contains("RFC 2822"), "message names RFC 2822: {msg}");
        assert!(msg.contains("long form"), "message names long form: {msg}");
        assert!(
            msg.contains("ISO 8601 date"),
            "message names ISO date: {msg}"
        );
        assert!(
            msg.contains("ISO 8601 datetime"),
            "message names ISO datetime: {msg}"
        );
    }

    #[test]
    fn error_truncates_long_input() {
        let long = "x".repeat(200);
        let err = parse_flexible_date(&long).unwrap_err();
        assert!(err.input().chars().count() <= 65);
        assert!(err.input().ends_with('…'));
    }

    // -----------------------------------------------------------------
    // Formatting details
    // -----------------------------------------------------------------

    #[test]
    fn weekday_computation_across_calendar() {
        // Known anchors.
        let cases = [
            ("2026-07-01", "Wed"),
            ("2024-02-29", "Thu"),
            ("2000-01-01", "Sat"),
            ("1990-01-01", "Mon"),
            ("2100-12-31", "Fri"),
        ];
        for (iso, weekday) in cases {
            let dt = parse_flexible_date(iso).unwrap();
            let rendered = dt.to_rfc2822();
            assert!(
                rendered.starts_with(weekday),
                "{iso} should be {weekday}, got {rendered}"
            );
        }
    }

    #[test]
    fn rfc2822_round_trips_through_itself() {
        let dt = parse_flexible_date("2026-07-01T07:07:07+05:30").unwrap();
        let reparsed = parse_flexible_date(&dt.to_rfc2822()).unwrap();
        assert_eq!(dt.to_rfc3339(), reparsed.to_rfc3339());
    }

    // -----------------------------------------------------------------
    // build_date — component validation
    // -----------------------------------------------------------------

    #[test]
    fn build_date_rejects_out_of_range_year() {
        // Year 0 fails the `(1..=9999)` guard via the public parser.
        assert!(parse_flexible_date("0000-01-01").is_err());
    }

    // -----------------------------------------------------------------
    // parse_hms — malformed clock tokens
    // -----------------------------------------------------------------

    #[test]
    fn parse_hms_rejects_malformed_tokens() {
        // Non-numeric hour.
        assert_eq!(parse_hms("xx:30"), None);
        // Missing minute component entirely.
        assert_eq!(parse_hms("12"), None);
        // Non-numeric minute.
        assert_eq!(parse_hms("12:xx"), None);
        // Non-numeric second.
        assert_eq!(parse_hms("12:30:xx"), None);
        // Too many components.
        assert_eq!(parse_hms("12:30:45:59"), None);
        // Empty token: defensive first-subtag fallback parses "".
        assert_eq!(parse_hms(""), None);
    }

    // -----------------------------------------------------------------
    // parse_zone — named zones and malformed offsets
    // -----------------------------------------------------------------

    #[test]
    fn parse_zone_maps_every_rfc2822_named_zone() {
        let cases = [
            ("EST", -5 * 60),
            ("EDT", -4 * 60),
            ("CST", -6 * 60),
            ("CDT", -5 * 60),
            ("MST", -7 * 60),
            ("MDT", -6 * 60),
            ("PST", -8 * 60),
            ("PDT", -7 * 60),
        ];
        for (name, minutes) in cases {
            assert_eq!(parse_zone(name), Some(minutes), "{name}");
        }
    }

    #[test]
    fn parse_zone_rejects_malformed_offsets() {
        // Empty token: no first byte.
        assert_eq!(parse_zone(""), None);
        // No sign byte.
        assert_eq!(parse_zone("0500"), None);
        // ±hhmm with a non-digit hour pair.
        assert_eq!(parse_zone("+aa30"), None);
        // ±hhmm with a non-digit minute pair.
        assert_eq!(parse_zone("+12a0"), None);
        // ±hh:mm with a non-digit hour pair.
        assert_eq!(parse_zone("+aa:30"), None);
        // ±hh:mm with a non-digit minute pair.
        assert_eq!(parse_zone("+12:a0"), None);
        // Wrong digit count.
        assert_eq!(parse_zone("+123"), None);
        // Out-of-range values.
        assert_eq!(parse_zone("+2460"), None);
    }

    // -----------------------------------------------------------------
    // ascii_number — emptiness, overflow, and non-digits
    // -----------------------------------------------------------------

    #[test]
    fn ascii_number_rejects_empty_overflow_and_non_digits() {
        assert_eq!(ascii_number(b""), None);
        // 11 nines overflows u32 via `checked_mul`.
        assert_eq!(ascii_number(b"99999999999"), None);
        assert_eq!(ascii_number(b"12a4"), None);
        assert_eq!(ascii_number(b"2026"), Some(2026));
    }

    // -----------------------------------------------------------------
    // parse_rfc2822 — per-token rejection paths
    // -----------------------------------------------------------------

    #[test]
    fn rfc2822_rejects_each_malformed_token() {
        // Day not numeric.
        assert!(parse_flexible_date("aa Jul 2026 12:00").is_err());
        // Month name unknown.
        assert!(parse_flexible_date("01 Foo 2026 12:00").is_err());
        // Year not 4 digits.
        assert!(parse_flexible_date("01 Jul 26 12:00").is_err());
        // Year contains a non-digit.
        assert!(parse_flexible_date("01 Jul 2o26 12:00").is_err());
        // Clock malformed.
        assert!(parse_flexible_date("01 Jul 2026 xx:00").is_err());
    }

    // -----------------------------------------------------------------
    // parse_long_form — per-token rejection paths
    // -----------------------------------------------------------------

    #[test]
    fn long_form_rejects_each_malformed_token() {
        // Day token longer than 2 chars.
        assert!(parse_flexible_date("July 123 2026").is_err());
        // Day token not numeric.
        assert!(parse_flexible_date("July aa 2026").is_err());
        // Year not 4 digits.
        assert!(parse_flexible_date("July 1 26").is_err());
        // Year contains a non-digit.
        assert!(parse_flexible_date("July 1 2o26").is_err());
    }

    // -----------------------------------------------------------------
    // parse_iso8601 — per-component rejection paths
    // -----------------------------------------------------------------

    #[test]
    fn iso8601_rejects_each_malformed_component() {
        // Year, month, and day with non-digits.
        assert!(parse_flexible_date("2o26-07-01").is_err());
        assert!(parse_flexible_date("2026-o7-01").is_err());
        assert!(parse_flexible_date("2026-07-o1").is_err());
        // Separator after the date is not T/t/space.
        assert!(parse_flexible_date("2026-07-01x12:00").is_err());
        // Malformed clock in the datetime form.
        assert!(parse_flexible_date("2026-07-01T1a:00").is_err());
    }
}

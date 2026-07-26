// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Integration tests for `ssg::dates` — the shared flexible date
//! parsing chain (issue #586 / v0.0.47 plan §2 item 1.4, spec A4).
//!
//! The property tests generate dates in 1990–2100, format them into
//! each of the three accepted input families (RFC 2822, long form,
//! ISO 8601), and assert round-trip equality through
//! `parse_flexible_date`.

#![allow(clippy::unwrap_used, clippy::expect_used)]
use proptest::prelude::*;
use ssg::dates::{
    days_in_month, parse_flexible_date, DateFormat, FlexibleDate,
};

/// Full month names for building long-form inputs.
const MONTH_NAMES: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

/// Build a `FlexibleDate` fixture (UTC) for formatting tests.
const fn utc_date(
    year: i32,
    month: u8,
    day: u8,
    hour: u8,
    minute: u8,
    second: u8,
) -> FlexibleDate {
    FlexibleDate {
        year,
        month,
        day,
        hour,
        minute,
        second,
        offset_minutes: 0,
        format: DateFormat::IsoDateTime,
    }
}

/// Proptest strategy: a valid (year, month, day) in 1990–2100 with a
/// leap-year-aware day, plus a time of day.
fn arb_datetime() -> impl Strategy<Value = (i32, u8, u8, u8, u8, u8)> {
    (1990i32..=2100, 1u8..=12).prop_flat_map(|(year, month)| {
        (
            Just(year),
            Just(month),
            1u8..=days_in_month(year, month),
            0u8..24,
            0u8..60,
            0u8..60,
        )
    })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    /// RFC 2822 round trip: format → parse → identical components
    /// and identical re-formatting.
    #[test]
    fn roundtrip_rfc2822(
        (year, month, day, hour, minute, second) in arb_datetime()
    ) {
        let dt = utc_date(year, month, day, hour, minute, second);
        let formatted = dt.to_rfc2822();
        let parsed = parse_flexible_date(&formatted)
            .expect("self-formatted RFC 2822 must parse");
        prop_assert_eq!(parsed.format, DateFormat::Rfc2822);
        prop_assert_eq!(
            (parsed.year, parsed.month, parsed.day),
            (year, month, day)
        );
        prop_assert_eq!(
            (parsed.hour, parsed.minute, parsed.second),
            (hour, minute, second)
        );
        prop_assert_eq!(parsed.offset_minutes, 0);
        prop_assert_eq!(parsed.to_rfc2822(), formatted);
    }

    /// Long-form round trip (both `July 1, 2026` and `1 July 2026`
    /// spellings): format → parse → identical calendar date.
    #[test]
    fn roundtrip_long_form(
        (year, month, day, _h, _m, _s) in arb_datetime()
    ) {
        let name = MONTH_NAMES[usize::from(month) - 1];
        for formatted in [
            format!("{name} {day}, {year}"),
            format!("{day} {name} {year}"),
        ] {
            let parsed = parse_flexible_date(&formatted)
                .expect("self-formatted long form must parse");
            prop_assert_eq!(parsed.format, DateFormat::LongForm);
            prop_assert_eq!(
                (parsed.year, parsed.month, parsed.day),
                (year, month, day)
            );
            // Long form carries no time: midnight UTC.
            prop_assert_eq!(
                (parsed.hour, parsed.minute, parsed.second, parsed.offset_minutes),
                (0, 0, 0, 0)
            );
        }
    }

    /// ISO 8601 date round trip: format → parse → identical date and
    /// identical re-formatting.
    #[test]
    fn roundtrip_iso_date(
        (year, month, day, _h, _m, _s) in arb_datetime()
    ) {
        let dt = utc_date(year, month, day, 0, 0, 0);
        let formatted = dt.to_iso_date();
        let parsed = parse_flexible_date(&formatted)
            .expect("self-formatted ISO date must parse");
        prop_assert_eq!(parsed.format, DateFormat::IsoDate);
        prop_assert_eq!(
            (parsed.year, parsed.month, parsed.day),
            (year, month, day)
        );
        prop_assert_eq!(parsed.to_iso_date(), formatted);
    }

    /// ISO 8601 datetime round trip: format → parse → identical
    /// components and identical re-formatting.
    #[test]
    fn roundtrip_iso_datetime(
        (year, month, day, hour, minute, second) in arb_datetime()
    ) {
        let dt = utc_date(year, month, day, hour, minute, second);
        let formatted = dt.to_rfc3339();
        let parsed = parse_flexible_date(&formatted)
            .expect("self-formatted RFC 3339 must parse");
        prop_assert_eq!(parsed.format, DateFormat::IsoDateTime);
        prop_assert_eq!(parsed.to_rfc3339(), formatted);
        prop_assert_eq!(
            (parsed.hour, parsed.minute, parsed.second),
            (hour, minute, second)
        );
    }

    /// All three input families agree: for any generated date, the
    /// RFC 2822, long-form, and ISO renderings of the same calendar
    /// day parse to the same `to_iso_date()`.
    #[test]
    fn all_formats_agree_on_calendar_day(
        (year, month, day, _h, _m, _s) in arb_datetime()
    ) {
        let name = MONTH_NAMES[usize::from(month) - 1];
        let iso = utc_date(year, month, day, 0, 0, 0).to_iso_date();
        let inputs = [
            utc_date(year, month, day, 0, 0, 0).to_rfc2822(),
            format!("{name} {day}, {year}"),
            iso.clone(),
        ];
        for input in inputs {
            let parsed = parse_flexible_date(&input)
                .expect("all renderings must parse");
            prop_assert_eq!(parsed.to_iso_date(), iso.clone());
        }
    }
}

// ---------------------------------------------------------------------
// Deterministic edge cases (spec A4 acceptance list)
// ---------------------------------------------------------------------

#[test]
fn leap_day_parses_only_in_leap_years() {
    assert!(parse_flexible_date("2024-02-29").is_ok());
    assert!(parse_flexible_date("February 29, 2024").is_ok());
    assert!(parse_flexible_date("Thu, 29 Feb 2024 00:00:00 +0000").is_ok());
    assert!(parse_flexible_date("2023-02-29").is_err());
    assert!(
        parse_flexible_date("2100-02-29").is_err(),
        "2100 is not leap"
    );
    assert!(parse_flexible_date("2000-02-29").is_ok(), "2000 is leap");
}

#[test]
fn single_digit_day_long_form() {
    let dt = parse_flexible_date("July 1, 2026").expect("must parse");
    assert_eq!(dt.to_iso_date(), "2026-07-01");
    let dt = parse_flexible_date("1 July 2026").expect("must parse");
    assert_eq!(dt.to_iso_date(), "2026-07-01");
}

#[test]
fn hhmm_offsets_round_trip() {
    let dt = parse_flexible_date("Wed, 01 Jul 2026 07:07:07 +0530")
        .expect("+hhmm parses");
    assert_eq!(dt.offset_minutes, 330);
    assert_eq!(dt.to_rfc3339(), "2026-07-01T07:07:07+05:30");
    assert_eq!(dt.to_rfc2822(), "Wed, 01 Jul 2026 07:07:07 +0530");

    let dt = parse_flexible_date("Wed, 01 Jul 2026 07:07:07 -0700")
        .expect("-hhmm parses");
    assert_eq!(dt.offset_minutes, -420);
    assert_eq!(dt.to_rfc3339(), "2026-07-01T07:07:07-07:00");

    let dt = parse_flexible_date("2026-07-01T07:07:07+0530")
        .expect("ISO compact offset parses");
    assert_eq!(dt.offset_minutes, 330);
}

#[test]
fn garbage_input_yields_typed_error_naming_formats() {
    let err = parse_flexible_date("definitely not a date")
        .expect_err("garbage must not parse");
    let msg = err.to_string();
    assert!(msg.contains("RFC 2822"), "names RFC 2822: {msg}");
    assert!(msg.contains("long form"), "names long form: {msg}");
    assert!(msg.contains("ISO 8601 date"), "names ISO date: {msg}");
    assert!(
        msg.contains("ISO 8601 datetime"),
        "names ISO datetime: {msg}"
    );
    assert_eq!(err.attempted_formats().len(), 4);
}

#[test]
fn spec_a4_examples_all_parse() {
    // The exact examples from the v0.0.47 plan §2 item 1.4.
    let rfc = parse_flexible_date("Wed, 01 Jul 2026 07:07:07 +0000")
        .expect("RFC 2822 example");
    let long = parse_flexible_date("July 1, 2026").expect("long form example");
    let iso = parse_flexible_date("2026-07-01").expect("ISO example");
    assert_eq!(rfc.to_iso_date(), "2026-07-01");
    assert_eq!(long.to_iso_date(), "2026-07-01");
    assert_eq!(iso.to_iso_date(), "2026-07-01");
}

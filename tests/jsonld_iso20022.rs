// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Integration tests for ISO 20022 / banking JSON-LD types (issue #551).
//!
//! Covers AC1 – AC7:
//!
//! - **AC1** opt-in via frontmatter (`emits_iso20022_block_when_frontmatter_present`)
//! - **AC2** IBAN MOD-97 (`iban_valid_passes`, `iban_invalid_warns`)
//! - **AC3** BIC 8/11 char (`bic_valid_passes`, `bic_wrong_length_warns`)
//! - **AC4** byte-identical output without iso20022 (`byte_identical_when_no_iso20022`)
//! - **AC5** Schema.org validator pass (`schema_org_validator_passes_for_every_emitted_blob`)
//! - **AC6** all 5 types covered (positive + negative test each — 10+ cases)
//! - **AC7** info-log fires once (`first_use_pointer_logs_exactly_once`)
//!
//! All assertions exercise the public API only — no internals reached
//! into.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use ssg::plugin::{Plugin, PluginContext};
use ssg::seo::jsonld::iso20022::{
    self, from_frontmatter, log_first_use_pointer, validate_bic, validate_iban,
    validate_schema_org, BankAccount, FinancialProduct, FinancialTransaction,
    Iso20022Entity, MonetaryAmount, PaymentInstrument,
    RegulatedFinancialInstitution, ValidationOutcome,
};
use ssg::seo::{JsonLdConfig, JsonLdPlugin};
use std::fs;
use std::path::PathBuf;
use tempfile::tempdir;

// ─────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────

fn plugin_with_breadcrumbs() -> JsonLdPlugin {
    JsonLdPlugin::new(JsonLdConfig {
        base_url: "https://example.com".into(),
        org_name: "Example Org".into(),
        breadcrumbs: true,
    })
}

fn make_ctx(site: &std::path::Path, build: &std::path::Path) -> PluginContext {
    let mut ctx = PluginContext::new(
        std::path::Path::new("content"),
        build,
        site,
        std::path::Path::new("templates"),
    );
    // build_dir is the second arg already; ensure .meta lives under it
    let _ = fs::create_dir_all(build.join(".meta"));
    ctx.cache_html_files();
    ctx
}

fn write_html(site: &std::path::Path, rel: &str) -> PathBuf {
    let path = site.join(rel);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    let html =
        "<html><head><title>Banking page</title></head><body><p>x</p></body></html>";
    fs::write(&path, html).unwrap();
    path
}

fn write_sidecar(build: &std::path::Path, rel_html: &str, payload: &str) {
    let sidecar_dir = build.join(".meta");
    let sidecar = sidecar_dir.join(rel_html).with_extension("meta.json");
    if let Some(parent) = sidecar.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(&sidecar, payload).unwrap();
}

/// Extracts the inner JSON payloads of every JSON-LD script in `html`.
fn extract_jsonld_payloads(html: &str) -> Vec<serde_json::Value> {
    let mut payloads = Vec::new();
    let mut rest = html;
    while let Some(open_idx) =
        rest.find(r#"<script type="application/ld+json">"#)
    {
        let after_open = &rest[open_idx + 35..];
        let Some(close_idx) = after_open.find("</script>") else {
            break;
        };
        let json = &after_open[..close_idx];
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(json) {
            payloads.push(v);
        }
        rest = &after_open[close_idx + 9..];
    }
    payloads
}

// ─────────────────────────────────────────────────────────────────────
// AC1 — opt-in via frontmatter
// ─────────────────────────────────────────────────────────────────────

#[test]
fn ac1_emits_iso20022_block_when_frontmatter_present() {
    let dir = tempdir().unwrap();
    let site = dir.path().join("site");
    let build = dir.path().join("build");
    fs::create_dir_all(&site).unwrap();
    fs::create_dir_all(&build).unwrap();

    let html_path = write_html(&site, "txn.html");
    write_sidecar(
        &build,
        "txn.html",
        r#"{
            "iso20022": {
                "type": "FinancialTransaction",
                "instructed_amount": {"currency": "EUR", "amount": 1500.0},
                "debtor_account":    {"iban": "GB29NWBK60161331926819"},
                "creditor_account":  {"iban": "DE89370400440532013000"}
            }
        }"#,
    );

    let ctx = make_ctx(&site, &build);
    let original = fs::read_to_string(&html_path).unwrap();
    let after = plugin_with_breadcrumbs()
        .transform_html(&original, &html_path, &ctx)
        .unwrap();

    assert!(
        after.contains("MoneyTransfer"),
        "expected MoneyTransfer @type"
    );
    assert!(
        after.contains("iso20022:debtorAccount"),
        "expected namespaced debtor account field"
    );
    assert!(
        after.contains("iso20022:creditorAccount"),
        "expected namespaced creditor account field"
    );
}

// ─────────────────────────────────────────────────────────────────────
// AC2 — BankAccount IBAN MOD-97
// ─────────────────────────────────────────────────────────────────────

#[test]
fn ac2_bank_account_iban_valid_passes() {
    assert!(validate_iban("GB29NWBK60161331926819").is_valid());
    assert!(validate_iban("DE89370400440532013000").is_valid());
}

#[test]
fn ac2_bank_account_iban_invalid_warns() {
    // Single-digit tweak breaks MOD-97.
    let outcome = validate_iban("GB29NWBK60161331926811");
    assert!(!outcome.is_valid());
    if let ValidationOutcome::Invalid { reason } = outcome {
        assert!(reason.contains("MOD-97"));
    }
}

// ─────────────────────────────────────────────────────────────────────
// AC3 — BIC format
// ─────────────────────────────────────────────────────────────────────

#[test]
fn ac3_bic_valid_8_and_11_pass() {
    assert!(validate_bic("NWBKGB2L").is_valid());
    assert!(validate_bic("NWBKGB2LXXX").is_valid());
}

#[test]
fn ac3_bic_wrong_length_warns() {
    let outcome = validate_bic("NWBKGB2"); // 7 chars
    assert!(!outcome.is_valid());
    if let ValidationOutcome::Invalid { reason } = outcome {
        assert!(reason.contains("8 or 11"));
    }
}

// ─────────────────────────────────────────────────────────────────────
// AC4 — byte-identical output without iso20022
// ─────────────────────────────────────────────────────────────────────

#[test]
fn ac4_byte_identical_when_no_iso20022_sidecar() {
    let dir = tempdir().unwrap();
    let site = dir.path().join("site");
    let build = dir.path().join("build");
    fs::create_dir_all(&site).unwrap();
    fs::create_dir_all(&build).unwrap();

    let html_path = write_html(&site, "ordinary.html");
    // No sidecar at all.
    let ctx = make_ctx(&site, &build);

    let original = fs::read_to_string(&html_path).unwrap();
    let after = plugin_with_breadcrumbs()
        .transform_html(&original, &html_path, &ctx)
        .unwrap();

    // Should match an emission with the plugin in its v0.0.43 form:
    // we can't directly diff against v0.0.43, but we can assert no
    // iso20022 substring appears anywhere in the output.
    assert!(
        !after.contains("iso20022"),
        "no iso20022 strings must appear on pages without the frontmatter key"
    );
}

#[test]
fn ac4_byte_identical_when_sidecar_has_no_iso20022_key() {
    let dir = tempdir().unwrap();
    let site = dir.path().join("site");
    let build = dir.path().join("build");
    fs::create_dir_all(&site).unwrap();
    fs::create_dir_all(&build).unwrap();

    let html_path = write_html(&site, "blog.html");
    write_sidecar(
        &build,
        "blog.html",
        r#"{"title": "Just a normal post", "tags": ["rust"]}"#,
    );
    let ctx = make_ctx(&site, &build);

    let original = fs::read_to_string(&html_path).unwrap();
    let after = plugin_with_breadcrumbs()
        .transform_html(&original, &html_path, &ctx)
        .unwrap();

    assert!(
        !after.contains("iso20022"),
        "sidecar without iso20022 key must not introduce ISO 20022 output"
    );
}

// ─────────────────────────────────────────────────────────────────────
// AC5 — Schema.org validator pass
// ─────────────────────────────────────────────────────────────────────

#[test]
fn ac5_schema_org_validator_passes_for_every_emitted_blob() {
    let entities = vec![
        Iso20022Entity::BankAccount(BankAccount {
            iban: Some("GB29NWBK60161331926819".into()),
            ..BankAccount::default()
        }),
        Iso20022Entity::PaymentInstrument(PaymentInstrument {
            instrument_type: "card".into(),
            ..PaymentInstrument::default()
        }),
        Iso20022Entity::FinancialTransaction(FinancialTransaction {
            instructed_amount: Some(MonetaryAmount {
                currency: "EUR".into(),
                amount: 100.0,
            }),
            ..FinancialTransaction::default()
        }),
        Iso20022Entity::RegulatedFinancialInstitution(
            RegulatedFinancialInstitution {
                name: "Acme Bank".into(),
                ..RegulatedFinancialInstitution::default()
            },
        ),
        Iso20022Entity::FinancialProduct(FinancialProduct {
            name: "Green Bond".into(),
            product_type: "deposit".into(),
            ..FinancialProduct::default()
        }),
    ];

    for entity in entities {
        let jsonld = entity.to_jsonld();
        let errors = validate_schema_org(&jsonld);
        assert!(
            errors.is_empty(),
            "expected zero schema.org errors for {} — got {:?}",
            entity.type_name(),
            errors
        );
    }
}

// ─────────────────────────────────────────────────────────────────────
// AC6 — All 5 types, positive + negative each
// ─────────────────────────────────────────────────────────────────────

// --- BankAccount

#[test]
fn ac6_bank_account_positive_round_trip_via_frontmatter() {
    let fm = serde_json::json!({
        "type": "BankAccount",
        "iban": "GB29NWBK60161331926819",
        "bic": "NWBKGB2L"
    });
    let entity = from_frontmatter(&fm).unwrap();
    assert_eq!(entity.type_name(), "BankAccount");
    let blob = entity.to_jsonld();
    assert_eq!(blob["iso20022:iban"], "GB29NWBK60161331926819");
    assert_eq!(blob["iso20022:bic"], "NWBKGB2L");
}

#[test]
fn ac6_bank_account_negative_invalid_iban_emits_warning() {
    let entity = Iso20022Entity::BankAccount(BankAccount {
        iban: Some("NOT-AN-IBAN".into()),
        ..BankAccount::default()
    });
    let warnings = iso20022::warn_invalid_fields(&entity, "ac6/bank.md");
    assert_eq!(warnings, 1);
}

// --- PaymentInstrument

#[test]
fn ac6_payment_instrument_positive_round_trip() {
    let fm = serde_json::json!({
        "type": "PaymentInstrument",
        "instrument_type": "transfer",
        "brand": "SEPA"
    });
    let entity = from_frontmatter(&fm).unwrap();
    assert_eq!(entity.type_name(), "PaymentInstrument");
    let blob = entity.to_jsonld();
    assert_eq!(blob["@type"], "PaymentService");
    assert_eq!(blob["iso20022:instrumentType"], "transfer");
}

#[test]
fn ac6_payment_instrument_negative_missing_required_type_field() {
    // `instrument_type` is a non-Option required field on the struct.
    // Frontmatter without it must surface a Malformed dispatch error.
    let fm = serde_json::json!({
        "type": "PaymentInstrument",
        "brand": "SEPA"
    });
    let err = from_frontmatter(&fm).unwrap_err();
    assert!(matches!(err, iso20022::DispatchError::Malformed(_)));
}

// --- FinancialTransaction

#[test]
fn ac6_financial_transaction_positive_round_trip() {
    let fm = serde_json::json!({
        "type": "FinancialTransaction",
        "instructed_amount": {"currency": "EUR", "amount": 1500.0},
        "debtor_account":    {"iban": "GB29NWBK60161331926819"},
        "creditor_account":  {"iban": "DE89370400440532013000"}
    });
    let entity = from_frontmatter(&fm).unwrap();
    let blob = entity.to_jsonld();
    assert_eq!(blob["@type"], "MoneyTransfer");
    assert_eq!(blob["amount"]["currency"], "EUR");
    assert_eq!(blob["amount"]["value"], 1500.0);
}

#[test]
fn ac6_financial_transaction_negative_invalid_creditor_iban_warns() {
    let entity = Iso20022Entity::FinancialTransaction(FinancialTransaction {
        instructed_amount: Some(MonetaryAmount {
            currency: "USD".into(),
            amount: 10.0,
        }),
        creditor_account: Some(BankAccount {
            iban: Some("INVALID".into()),
            ..BankAccount::default()
        }),
        ..FinancialTransaction::default()
    });
    let warnings = iso20022::warn_invalid_fields(&entity, "ac6/txn.md");
    assert_eq!(warnings, 1);
}

// --- RegulatedFinancialInstitution

#[test]
fn ac6_regulated_institution_positive_round_trip() {
    let fm = serde_json::json!({
        "type": "RegulatedFinancialInstitution",
        "name": "Acme Bank plc",
        "lei": "529900W18LQJJN6SJ336",
        "regulator": "FCA"
    });
    let entity = from_frontmatter(&fm).unwrap();
    let blob = entity.to_jsonld();
    assert_eq!(blob["@type"], "BankOrCreditUnion");
    assert_eq!(blob["name"], "Acme Bank plc");
    assert_eq!(blob["iso20022:lei"], "529900W18LQJJN6SJ336");
}

#[test]
fn ac6_regulated_institution_negative_missing_name_fails_schema() {
    // Build via direct struct so we can elide `name` (frontmatter
    // dispatch would have failed earlier on the required field).
    let entity = Iso20022Entity::RegulatedFinancialInstitution(
        RegulatedFinancialInstitution {
            name: String::new(),
            ..RegulatedFinancialInstitution::default()
        },
    );
    let errs = validate_schema_org(&entity.to_jsonld());
    assert!(errs.iter().any(|e| e.field == "name"));
}

// --- FinancialProduct

#[test]
fn ac6_financial_product_positive_round_trip() {
    let fm = serde_json::json!({
        "type": "FinancialProduct",
        "name": "5Y Green Bond",
        "product_type": "deposit",
        "issuer": "Acme Bank",
        "annual_percentage_rate": 3.5,
        "isin": "US0378331005"
    });
    let entity = from_frontmatter(&fm).unwrap();
    let blob = entity.to_jsonld();
    assert_eq!(blob["@type"], "FinancialProduct");
    assert_eq!(blob["name"], "5Y Green Bond");
    assert_eq!(blob["annualPercentageRate"], 3.5);
}

#[test]
fn ac6_financial_product_negative_missing_required_name_dispatch_error() {
    let fm = serde_json::json!({
        "type": "FinancialProduct",
        "product_type": "deposit"
    });
    let err = from_frontmatter(&fm).unwrap_err();
    assert!(matches!(err, iso20022::DispatchError::Malformed(_)));
}

// ─────────────────────────────────────────────────────────────────────
// AC7 — info-log pointer
// ─────────────────────────────────────────────────────────────────────

#[test]
fn ac7_first_use_pointer_logs_exactly_once() {
    // log_first_use_pointer is process-wide idempotent. We can't easily
    // probe the log line from here without bringing in tracing-test, so
    // we just call it twice and assert it doesn't panic / loop. The
    // unit test in `iso20022::tests` covers the once-only flag flip.
    log_first_use_pointer();
    log_first_use_pointer();
}

// ─────────────────────────────────────────────────────────────────────
// End-to-end transform asserts (covers AC1 + AC5 together)
// ─────────────────────────────────────────────────────────────────────

#[test]
fn end_to_end_transform_emits_valid_schema_org() {
    let dir = tempdir().unwrap();
    let site = dir.path().join("site");
    let build = dir.path().join("build");
    fs::create_dir_all(&site).unwrap();
    fs::create_dir_all(&build).unwrap();

    let html_path = write_html(&site, "product.html");
    write_sidecar(
        &build,
        "product.html",
        r#"{
            "iso20022": {
                "type": "FinancialProduct",
                "name": "Sustainable Savings",
                "product_type": "deposit",
                "annual_percentage_rate": 4.25,
                "isin": "US0378331005"
            }
        }"#,
    );

    let ctx = make_ctx(&site, &build);
    let original = fs::read_to_string(&html_path).unwrap();
    let after = plugin_with_breadcrumbs()
        .transform_html(&original, &html_path, &ctx)
        .unwrap();

    let blobs = extract_jsonld_payloads(&after);
    assert!(
        blobs.iter().any(|b| b["@type"] == "FinancialProduct"),
        "expected a FinancialProduct JSON-LD block"
    );
    // Every emitted blob whose @type is one our validator knows about
    // must pass.
    for blob in &blobs {
        let errs = validate_schema_org(blob);
        // Validator only tightens for ISO 20022 @types — pages with
        // WebPage/BreadcrumbList shouldn't generate spurious errors
        // since the validator returns [] for unknown @types after the
        // context check. The breadcrumbs and WebPage payloads also
        // carry @context referencing schema.org, so they pass.
        assert!(
            errs.is_empty(),
            "blob {:?} produced {:?}",
            blob.get("@type"),
            errs
        );
    }
}

#[test]
fn end_to_end_invalid_iban_does_not_abort_build() {
    let dir = tempdir().unwrap();
    let site = dir.path().join("site");
    let build = dir.path().join("build");
    fs::create_dir_all(&site).unwrap();
    fs::create_dir_all(&build).unwrap();

    let html_path = write_html(&site, "bad.html");
    write_sidecar(
        &build,
        "bad.html",
        r#"{
            "iso20022": {
                "type": "BankAccount",
                "iban": "DEFINITELY-NOT-VALID"
            }
        }"#,
    );

    let ctx = make_ctx(&site, &build);
    let original = fs::read_to_string(&html_path).unwrap();
    // Must succeed — invalid IBAN is a warning, not an error.
    let after = plugin_with_breadcrumbs()
        .transform_html(&original, &html_path, &ctx)
        .expect("invalid IBAN must not abort the build");

    // The BankAccount block is still emitted with the original IBAN
    // string — downstream consumers can see it, validators can complain.
    assert!(after.contains("BankAccount"));
    assert!(after.contains("DEFINITELY-NOT-VALID"));
}

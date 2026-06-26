#![allow(clippy::unwrap_used, clippy::expect_used)]
// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! # ISO 20022 Example — fintech-domain JSON-LD (v0.0.44)
//!
//! Demonstrates the ISO 20022 JSON-LD extension on the SEO plugin:
//!
//! 1. Builds a debtor [`BankAccount`] + creditor [`BankAccount`].
//! 2. Builds a [`MonetaryAmount`] in EUR.
//! 3. Composes a [`FinancialTransaction`] (`MoneyTransfer`).
//! 4. Validates the supplied IBAN + BIC via [`validate_iban`] /
//!    [`validate_bic`].
//! 5. Prints the Schema.org-compatible JSON-LD blob.
//!
//! ## Run it
//!
//! ```sh
//! cargo run --example iso20022_example
//! ```

use ssg::seo::jsonld::iso20022::{
    validate_bic, validate_iban, BankAccount, FinancialTransaction,
    MonetaryAmount,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Two accounts — the canonical Belgian + German test IBANs that
    //    pass MOD-97 (every fintech tutorial uses them).
    let debtor = BankAccount {
        name: Some("Alice Dupont".into()),
        iban: Some("BE68539007547034".into()),
        bic: Some("GKCCBEBB".into()),
    };
    let creditor = BankAccount {
        name: Some("Acme GmbH".into()),
        iban: Some("DE89370400440532013000".into()),
        bic: Some("COBADEFFXXX".into()),
    };

    // 2. Amount and 3. composite transaction.
    let amount = MonetaryAmount {
        currency: "EUR".into(),
        amount: 1234.56,
    };
    let txn = FinancialTransaction {
        instructed_amount: Some(amount.clone()),
        debtor_account: Some(debtor.clone()),
        creditor_account: Some(creditor.clone()),
        execution_date: Some("2026-06-26".into()),
        end_to_end_id: Some("E2E-20260626-001".into()),
    };

    // 4. Validate IBAN + BIC on both sides. Print outcomes so the user
    //    can see which checks ran.
    for (label, iban, bic) in [
        (
            "debtor",
            debtor.iban.as_deref().unwrap(),
            debtor.bic.as_deref().unwrap(),
        ),
        (
            "creditor",
            creditor.iban.as_deref().unwrap(),
            creditor.bic.as_deref().unwrap(),
        ),
    ] {
        let iban_ok = validate_iban(iban).is_valid();
        let bic_ok = validate_bic(bic).is_valid();
        println!(
            "[iso20022] {label}: iban={iban} ({}) bic={bic} ({})",
            if iban_ok { "valid" } else { "INVALID" },
            if bic_ok { "valid" } else { "INVALID" },
        );
    }

    // 5. Print the resulting JSON-LD payload.
    let jsonld = txn.to_jsonld();
    println!("[iso20022] JSON-LD payload for the transaction:");
    println!("{}", serde_json::to_string_pretty(&jsonld)?);

    Ok(())
}

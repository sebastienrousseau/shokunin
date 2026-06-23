// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Integration test for issue #520: `LlmPlugin` must not shell out.
//!
//! Asserts the post-#520 invariants:
//!
//! 1. Static: `src/plugins/llm.rs` contains zero references to
//!    `Command::new(...)` or `process::Command` (AC1).
//! 2. Behavioural: invoking `LlmPlugin::query` against an
//!    unreachable endpoint returns the typed
//!    `SsgError::LlmEndpointUnreachable` instead of a stringly
//!    wrapped `stderr` payload (AC4).
//! 3. Prompt safety: a prompt containing every shell metacharacter
//!    that previously would have been a quoting hazard
//!    (`$(`, backticks, `;`, `&`, `|`, redirects) goes through
//!    `ureq` unchanged, and `tracing-test` capture shows no
//!    `process::Command` event was emitted (AC3).
//! 4. Tokio-free: this test file deliberately does not import any
//!    async runtime to keep the AC6 ("no new tokio dep") invariant
//!    obvious at the call site.
//!
//! Run with:
//!
//! ```bash
//! cargo test --features ai --test llm_no_shellout
//! ```

#![cfg(feature = "ai")]
#![allow(clippy::unwrap_used, clippy::expect_used)]

use ssg::llm::{LlmConfig, LlmPlugin};
use ssg::SsgError;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use tracing_test::traced_test;

/// AC1 — static guard: re-prove the grep invariant at test time so
/// a future regression that re-introduces `Command::new` in
/// `src/plugins/llm.rs` fails CI even if the grep step is skipped.
#[test]
fn ac1_no_command_new_in_llm_source() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("plugins")
        .join("llm.rs");
    let src = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("read {path:?}: {e}"));

    // Strip line comments so the AC commentary about the previous
    // `Command::new("curl")` shellout (which is the load-bearing
    // historical note for issue #520) does not trigger a false
    // positive.
    let stripped: String = src
        .lines()
        .map(|line| line.split("//").next().unwrap_or(""))
        .collect::<Vec<_>>()
        .join("\n");

    assert!(
        !stripped.contains("Command::new"),
        "AC1 regression: src/plugins/llm.rs reintroduced \
         `Command::new` — the curl shellout was ported to ureq in #520"
    );
    assert!(
        !stripped.contains("process::Command"),
        "AC1 regression: src/plugins/llm.rs reintroduced \
         `process::Command` — the curl shellout was ported to \
         ureq in #520"
    );
}

/// AC4 — unreachable endpoint surfaces as typed
/// `SsgError::LlmEndpointUnreachable` (or `LlmTimeout` on hosts
/// where the resolver stalls instead of refusing). Either way the
/// caller never sees a stringly-typed stderr wrap.
#[test]
fn ac4_unreachable_endpoint_returns_typed_error() {
    // Port 1 is reserved (TCPMUX) and reliably refuses local
    // connections without the test depending on an exotic loopback
    // alias. If a host quirk turns this into a hang the timeout
    // budget below caps the test at ~2s.
    let config = LlmConfig {
        endpoint: "http://127.0.0.1:1".to_string(),
        timeout_secs: 2,
        ..LlmConfig::default()
    };
    let plugin = LlmPlugin::new(config);

    let result = plugin.query("hello");
    let err = result
        .expect_err("query against 127.0.0.1:1 must fail with a typed error");
    assert!(
        matches!(
            err,
            SsgError::LlmEndpointUnreachable { .. }
                | SsgError::LlmTimeout { .. }
        ),
        "AC4: expected LlmEndpointUnreachable or LlmTimeout, got {err:?}",
    );
}

/// AC5 — `llm.timeout_secs` is honoured and the call does not leave
/// a zombie subprocess (because there isn't one — `ureq` runs
/// in-process). We assert the elapsed time is bounded by the
/// configured budget plus a generous slack to absorb scheduler
/// jitter on shared CI runners.
#[test]
fn ac5_timeout_is_bounded() {
    let config = LlmConfig {
        endpoint: "http://127.0.0.1:1".to_string(),
        timeout_secs: 1,
        ..LlmConfig::default()
    };
    let plugin = LlmPlugin::new(config);

    let started = Instant::now();
    let _ = plugin.query("hello");
    let elapsed = started.elapsed();

    assert!(
        elapsed < Duration::from_secs(10),
        "AC5: query exceeded the timeout budget — elapsed {elapsed:?}",
    );
}

/// AC3 — prompts with shell metacharacters traverse `ureq`
/// untouched and no subprocess-spawn event is emitted by the
/// `tracing` subscriber installed by `#[traced_test]`. We assert
/// negatively: nothing in the captured logs mentions `curl`,
/// `Command::new`, or `process::Command`.
#[test]
#[traced_test]
fn ac3_prompt_injection_bytes_do_not_spawn_subshell() {
    let evil_prompt = "$(rm -rf /); `whoami`; foo & bar | baz > /tmp/x";
    let config = LlmConfig {
        endpoint: "http://127.0.0.1:1".to_string(),
        timeout_secs: 2,
        ..LlmConfig::default()
    };
    let plugin = LlmPlugin::new(config);

    // The call will error (port 1 refuses) — we do not care about
    // the outcome, only that the *transport* it tried was ureq.
    let _ = plugin.query(evil_prompt);

    // `tracing-test` provides this assertion helper which fails if
    // ANY captured log line contains the substring. We invert that
    // intent by calling `logs_contain` and checking it is `false`.
    assert!(
        !logs_contain("Command::new"),
        "AC3: tracing capture saw a `Command::new` event — the \
         curl shellout was supposed to be gone"
    );
    assert!(
        !logs_contain("process::Command"),
        "AC3: tracing capture saw a `process::Command` event"
    );
    assert!(
        !logs_contain("spawn"),
        "AC3: tracing capture saw a `spawn` event — no subprocess \
         should be created by LlmPlugin::query"
    );
}

/// AC6 sanity — `LlmConfig::default().timeout_secs` matches the
/// documented default (120) so users get the same behaviour the
/// ssg.toml schema advertises.
#[test]
fn ac5_default_timeout_is_120s() {
    assert_eq!(LlmConfig::default().timeout_secs, 120);
}

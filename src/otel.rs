// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! OpenTelemetry build-pipeline tracing scaffolding (issue #422).
//!
//! This module is intentionally a *scaffold*. The full deliverable
//! ships in two phases:
//!
//! 1. **Phase A (this commit)** — `otel` Cargo feature, `--trace` CLI
//!    flag, an `init_if_enabled` initialiser that attaches a
//!    `tracing-subscriber` JSON formatter to stdout, and one demo
//!    span around `pipeline::execute_build_pipeline` via
//!    `#[tracing::instrument]`.
//!
//! 2. **Phase B (deferred follow-up)** — per-plugin spans inside
//!    `PluginManager::run_*`, file-count + duration + peak-RSS
//!    delta fields, OTLP/gRPC + Jaeger exporter wiring, and a
//!    Grafana dashboard JSON. Phase B requires `tokio` to be
//!    introduced; the rest of SSG is rayon-based, so that
//!    architectural decision is deliberately deferred.
//!
//! When the `otel` feature is **off**, this module compiles to an
//! empty stub — `init_if_enabled` is a no-op. Callers may invoke it
//! unconditionally and it will simply do nothing.

/// Initialises tracing if both:
///
/// 1. The crate was compiled with the `otel` feature, and
/// 2. `enabled` is `true` (typically driven by the `--trace` CLI flag).
///
/// On `(true, true)`: installs a `tracing-subscriber` global
/// dispatcher with JSON formatting to stdout, level filter from
/// `RUST_LOG` (default `info`).
///
/// In any other case: returns immediately, no global state mutated.
///
/// # Returns
///
/// `true` if a subscriber was installed; `false` otherwise.
pub fn init_if_enabled(enabled: bool) -> bool {
    if !enabled {
        return false;
    }
    real::init()
}

#[cfg(feature = "otel")]
mod real {
    use tracing_subscriber::{
        fmt::format::FmtSpan, prelude::*, EnvFilter,
    };

    pub(super) fn init() -> bool {
        // Idempotent: if a subscriber is already installed (e.g.
        // double `--trace` invocation in a script that re-enters),
        // silently no-op.
        let filter = EnvFilter::try_from_default_env()
            .or_else(|_| EnvFilter::try_new("info"))
            .unwrap_or_default();

        let layer = tracing_subscriber::fmt::layer()
            .json()
            .with_span_events(FmtSpan::CLOSE)
            .with_target(true);

        let installed = tracing_subscriber::registry()
            .with(filter)
            .with(layer)
            .try_init()
            .is_ok();

        if installed {
            tracing::info!(
                target = "ssg::otel",
                "OpenTelemetry build tracing enabled (JSON to stdout)"
            );
        }
        installed
    }
}

#[cfg(not(feature = "otel"))]
mod real {
    /// When the `otel` feature is disabled the runtime is absent;
    /// even if `--trace` is passed, we emit a warning via the
    /// existing `log` facade and return `false`. The CLI flag is
    /// still parsed so scripts work across feature-on/feature-off
    /// builds without conditional logic.
    pub(super) fn init() -> bool {
        log::warn!(
            "[--trace] requested but this binary was built without the \
             `otel` feature. Rebuild with `cargo build --features otel` \
             to enable build-pipeline tracing."
        );
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_disabled_is_noop_returns_false() {
        // Always returns false when the caller passes `false`,
        // regardless of feature state.
        assert!(!init_if_enabled(false));
    }

    #[cfg(not(feature = "otel"))]
    #[test]
    fn init_enabled_without_feature_warns_and_returns_false() {
        // Without the feature compiled in, the second call also
        // returns false (it logs a warning via `log` rather than
        // panicking).
        assert!(!init_if_enabled(true));
    }
}

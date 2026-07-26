// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Integration tests for `ssg::otel` (feature-gated).

#![allow(clippy::unwrap_used, clippy::expect_used)]
#[cfg(feature = "otel")]
mod gated {
    use ssg::otel::init_if_enabled;

    #[test]
    fn init_if_enabled_returns_true_when_enabled() {
        let activated = init_if_enabled(true);
        assert!(activated);
    }

    #[test]
    fn init_if_enabled_returns_false_when_disabled() {
        assert!(!init_if_enabled(false));
    }
}

// Provide at least one unconditional test so this module always compiles
// to a non-empty cdylib regardless of feature flags.
#[test]
fn module_compiles() {
    let _ = std::any::type_name::<()>();
}

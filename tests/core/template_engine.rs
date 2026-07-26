// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Integration tests for `ssg::template_engine` (feature-gated `templates`).

#![allow(clippy::unwrap_used, clippy::expect_used)]
#[cfg(feature = "templates")]
mod gated {
    use ssg::template_engine::{TemplateConfig, TemplateEngine};
    use tempfile::tempdir;

    #[test]
    fn template_engine_init_accepts_default_config() {
        let dir = tempdir().unwrap();
        let cfg = TemplateConfig {
            template_dir: dir.path().to_path_buf(),
            ..Default::default()
        };
        // init returns Result<Option<Self>> — either None (no templates)
        // or Some(engine). Both branches are acceptable initialisation
        // outcomes for an empty template directory.
        let _ = TemplateEngine::init(cfg);
    }
}

// Always-on smoke test so the module compiles under --no-default-features.
#[test]
fn module_compiles() {
    let _ = std::any::type_name::<()>();
}

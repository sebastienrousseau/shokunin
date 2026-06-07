// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Integration tests for `ssg::plugins::image_plugin` (feature-gated `image-optimization`).

#[cfg(feature = "image-optimization")]
mod gated {
    use ssg::image_plugin::ImageOptimizationPlugin;
    use ssg::plugin::Plugin;

    #[test]
    fn image_plugin_default_constructs() {
        let p = ImageOptimizationPlugin::default();
        assert!(!p.name().is_empty());
    }
}

#[test]
fn module_compiles() {
    let _ = std::any::type_name::<()>();
}

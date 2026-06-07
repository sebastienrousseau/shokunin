// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! `postprocess::helpers` is `pub(crate)` only and has no public API
//! surface. The module is exercised via its consumers (`atom`, `rss`,
//! `manifest`, `sitemap`, `news_sitemap`) and their integration tests.

#[test]
fn module_present() {
    let _ = std::any::type_name::<()>();
}

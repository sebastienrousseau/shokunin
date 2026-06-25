// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Cross-cutting utility modules.
//!
//! - [`html_rewriter`] — streaming `lol_html` wrapper used by image,
//!   search, CSP, and html-fix plugins (issue #525).
//! - [`head_dom`] — parser-driven HTML `<head>` manipulation used by the
//!   SEO, `og_image`, `llm`, `ai`, `i18n`, `highlight`, `sbom`, `atom`,
//!   `json_feed`, and `jsonld` plugins (issues #538, #539, #540).

pub mod head_dom;
pub mod html_rewriter;

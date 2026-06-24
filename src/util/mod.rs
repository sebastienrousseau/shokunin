// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Cross-cutting utility modules.
//!
//! Currently houses the streaming HTML rewriter wrapper
//! ([`html_rewriter`]) used by the image, search, CSP, and html-fix
//! plugins to replace the fragile `str::find` / `str::rfind` rewriting
//! that previously lived in each plugin (issue #525).

pub mod html_rewriter;

// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Integration tests for `src/server/` — one binary, one submodule per source file.

#![allow(clippy::unwrap_used, clippy::expect_used)]
mod event_watch;
mod hmr;
mod livereload;
mod server;
mod watch;

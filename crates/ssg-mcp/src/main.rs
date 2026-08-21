// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! `ssg-mcp` — Model Context Protocol server on stdio.
//!
//! Thin by design: everything testable lives in the library, so this binary
//! is only the wiring between stdin/stdout and [`ssg_mcp::serve`].

fn main() -> std::io::Result<()> {
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    ssg_mcp::serve(stdin.lock(), stdout.lock())
}

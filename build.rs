// Copyright © 2023-2024 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! This is the main function for the build script.
//!
//! Currently, it only instructs Cargo to re-run this build script if `build.rs` is changed.
fn main() {
    // Avoid unnecessary re-building.
    println!("cargo:rerun-if-changed=build.rs");
}

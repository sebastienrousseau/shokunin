// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Integration tests for `ssg::streaming`.

use ssg::streaming::{should_stream, MemoryBudget};
use tempfile::tempdir;

#[test]
fn memory_budget_from_mb_constructs_a_budget() {
    let _ = MemoryBudget::from_mb(256);
}

#[test]
fn default_budget_is_usable() {
    let _ = MemoryBudget::default_budget();
}

#[test]
fn should_stream_returns_true_when_explicitly_set() {
    let dir = tempdir().unwrap();
    let b = MemoryBudget::default_budget();
    assert!(should_stream(dir.path(), &b, true));
}

#[test]
fn should_stream_returns_false_on_small_tree_without_override() {
    let dir = tempdir().unwrap();
    let b = MemoryBudget::default_budget();
    assert!(!should_stream(dir.path(), &b, false));
}

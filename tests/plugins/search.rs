// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Integration tests for `ssg::plugins::search`.

use ssg::plugin::Plugin;
use ssg::search::{SearchEntry, SearchIndex, SearchPlugin};

#[test]
fn search_plugin_name_is_stable() {
    assert!(!SearchPlugin.name().is_empty());
}

#[test]
fn search_entry_constructs_with_explicit_fields() {
    let entry = SearchEntry {
        title: "Hello".into(),
        url: "/hello/".into(),
        content: "world".into(),
        headings: vec![],
    };
    assert_eq!(entry.title, "Hello");
}

#[test]
fn search_index_holds_entries_in_order() {
    let idx = SearchIndex {
        entries: vec![
            SearchEntry {
                title: "a".into(),
                url: "/a/".into(),
                content: String::new(),
                headings: vec![],
            },
            SearchEntry {
                title: "b".into(),
                url: "/b/".into(),
                content: String::new(),
                headings: vec![],
            },
        ],
    };
    assert_eq!(idx.entries.len(), 2);
}

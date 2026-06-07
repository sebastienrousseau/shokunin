// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Integration tests for `ssg::depgraph::DepGraph`.

use std::path::PathBuf;

use ssg::depgraph::DepGraph;
use tempfile::tempdir;

#[test]
fn new_graph_has_zero_pages() {
    let g = DepGraph::new();
    assert_eq!(g.page_count(), 0);
}

#[test]
fn add_dep_increases_page_count() {
    let mut g = DepGraph::new();
    g.add_dep(&PathBuf::from("page.md"), &PathBuf::from("template.html"));
    assert_eq!(g.page_count(), 1);
}

#[test]
fn deps_for_returns_recorded_dependencies() {
    let mut g = DepGraph::new();
    let page = PathBuf::from("page.md");
    g.add_dep(&page, &PathBuf::from("template.html"));
    g.add_dep(&page, &PathBuf::from("partial.html"));
    let deps = g.deps_for(&page).unwrap();
    assert_eq!(deps.len(), 2);
}

#[test]
fn invalidated_pages_finds_dependents_of_changed_file() {
    let mut g = DepGraph::new();
    g.add_dep(&PathBuf::from("a.md"), &PathBuf::from("shared.html"));
    g.add_dep(&PathBuf::from("b.md"), &PathBuf::from("shared.html"));
    let invalidated =
        g.invalidated_pages(&[PathBuf::from("shared.html")]);
    // invalidated_pages returns the union of {changed} ∪ {pages depending
    // on changed} — so we expect 3 entries: shared.html + a.md + b.md.
    assert_eq!(invalidated.len(), 3);
}

#[test]
fn clear_empties_the_graph() {
    let mut g = DepGraph::new();
    g.add_dep(&PathBuf::from("p.md"), &PathBuf::from("t.html"));
    g.clear();
    assert_eq!(g.page_count(), 0);
}

#[test]
fn save_and_load_round_trip_via_site_dir() {
    let dir = tempdir().unwrap();
    let mut g = DepGraph::new();
    g.add_dep(&PathBuf::from("p.md"), &PathBuf::from("t.html"));
    g.save(dir.path()).unwrap();
    let reloaded = DepGraph::load(dir.path());
    assert_eq!(reloaded.page_count(), 1);
}

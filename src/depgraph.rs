// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Page dependency graph for incremental rebuilds.
//!
//! Tracks which pages depend on which templates, shortcodes, and data
//! files. When a dependency changes, only the pages that use it are
//! invalidated and recompiled.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

const DEP_GRAPH_FILE: &str = ".ssg-deps.json";

/// Dependency graph mapping pages to their dependencies.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DepGraph {
    /// page relative path → set of dependency relative paths
    deps: HashMap<PathBuf, HashSet<PathBuf>>,
}

impl DepGraph {
    /// Creates an empty dependency graph.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Loads from `.ssg-deps.json` in the site directory.
    /// Returns an empty graph if the file is missing or corrupt.
    #[must_use]
    pub fn load(site_dir: &Path) -> Self {
        let path = site_dir.join(DEP_GRAPH_FILE);
        match fs::read_to_string(&path) {
            Ok(json) => serde_json::from_str(&json).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    /// Persists the graph to `.ssg-deps.json`.
    pub fn save(&self, site_dir: &Path) -> Result<()> {
        let path = site_dir.join(DEP_GRAPH_FILE);
        let json = serde_json::to_string_pretty(self)?;
        fs::write(&path, json)?;
        Ok(())
    }

    /// Records that `page` depends on `dep`.
    pub fn add_dep(&mut self, page: &Path, dep: &Path) {
        let _ = self
            .deps
            .entry(page.to_path_buf())
            .or_default()
            .insert(dep.to_path_buf());
    }

    /// Returns the dependencies for a given page.
    #[must_use]
    pub fn deps_for(&self, page: &Path) -> Option<&HashSet<PathBuf>> {
        self.deps.get(page)
    }

    /// Given a list of changed files, returns all pages that need
    /// rebuilding — either because they changed directly, or because
    /// one of their dependencies changed.
    #[must_use]
    pub fn invalidated_pages(&self, changed: &[PathBuf]) -> Vec<PathBuf> {
        let changed_set: HashSet<&PathBuf> = changed.iter().collect();
        let mut invalidated: HashSet<PathBuf> = HashSet::new();

        // Pages whose own content changed
        for path in changed {
            let _ = invalidated.insert(path.clone());
        }

        // Pages whose dependencies changed
        for (page, deps) in &self.deps {
            if deps.iter().any(|d| changed_set.contains(d)) {
                let _ = invalidated.insert(page.clone());
            }
        }

        let mut result: Vec<PathBuf> = invalidated.into_iter().collect();
        result.sort();
        result
    }

    /// Returns the total number of tracked pages.
    #[must_use]
    pub fn page_count(&self) -> usize {
        self.deps.len()
    }

    /// Clears all dependency entries.
    pub fn clear(&mut self) {
        self.deps.clear();
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn empty_graph() {
        let graph = DepGraph::new();
        let changed = vec![PathBuf::from("content/index.md")];
        let result = graph.invalidated_pages(&changed);
        assert_eq!(result, vec![PathBuf::from("content/index.md")]);
    }

    #[test]
    fn direct_change() {
        let mut graph = DepGraph::new();
        let page = PathBuf::from("content/about.md");
        let tmpl = PathBuf::from("templates/base.html");
        graph.add_dep(&page, &tmpl);

        let changed = vec![page.clone()];
        let result = graph.invalidated_pages(&changed);
        assert_eq!(result, vec![page]);
    }

    #[test]
    fn dependency_change() {
        let mut graph = DepGraph::new();
        let page_a = PathBuf::from("content/index.md");
        let page_b = PathBuf::from("content/about.md");
        let tmpl = PathBuf::from("templates/base.html");
        graph.add_dep(&page_a, &tmpl);
        graph.add_dep(&page_b, &tmpl);

        let changed = vec![tmpl];
        let result = graph.invalidated_pages(&changed);
        // Both pages depend on the template, plus the template itself
        assert!(result.contains(&page_a));
        assert!(result.contains(&page_b));
        assert_eq!(result.len(), 3); // page_a, page_b, tmpl
    }

    #[test]
    fn transitive_not_tracked() {
        // Only direct dependencies matter; transitive closure is not computed.
        let mut graph = DepGraph::new();
        let page = PathBuf::from("content/index.md");
        let partial = PathBuf::from("templates/partial.html");
        let base = PathBuf::from("templates/base.html");
        // page → partial, partial → base (but we only track direct deps)
        graph.add_dep(&page, &partial);
        graph.add_dep(&partial, &base);

        // Changing base should NOT invalidate page (no direct dep)
        let changed = vec![base.clone()];
        let result = graph.invalidated_pages(&changed);
        assert!(result.contains(&base));
        assert!(result.contains(&partial)); // partial depends on base
        assert!(!result.contains(&page)); // page does NOT depend on base
    }

    #[test]
    fn save_and_load_round_trip() {
        let dir = tempdir().expect("test invariant");
        let mut graph = DepGraph::new();
        let page = PathBuf::from("content/index.md");
        let tmpl = PathBuf::from("templates/base.html");
        graph.add_dep(&page, &tmpl);

        graph.save(dir.path()).expect("test invariant");
        let loaded = DepGraph::load(dir.path());

        assert_eq!(loaded.page_count(), 1);
        let deps = loaded.deps_for(&page).expect("test invariant");
        assert!(deps.contains(&tmpl));
    }

    #[test]
    fn load_missing_file() {
        let dir = tempdir().expect("test invariant");
        let graph = DepGraph::load(dir.path());
        assert_eq!(graph.page_count(), 0);
    }

    #[test]
    fn load_corrupt_json() {
        let dir = tempdir().expect("test invariant");
        let path = dir.path().join(".ssg-deps.json");
        fs::write(&path, "not valid json {{{{").expect("test invariant");
        let graph = DepGraph::load(dir.path());
        assert_eq!(graph.page_count(), 0);
    }

    #[test]
    fn add_multiple_deps() {
        let mut graph = DepGraph::new();
        let page = PathBuf::from("content/post.md");
        let dep_a = PathBuf::from("templates/post.html");
        let dep_b = PathBuf::from("shortcodes/gallery.html");
        let dep_c = PathBuf::from("data/authors.json");
        graph.add_dep(&page, &dep_a);
        graph.add_dep(&page, &dep_b);
        graph.add_dep(&page, &dep_c);

        // Change only one dependency
        let changed = vec![dep_b.clone()];
        let result = graph.invalidated_pages(&changed);
        assert!(result.contains(&page));
        assert!(result.contains(&dep_b));
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn no_false_positives() {
        let mut graph = DepGraph::new();
        let page = PathBuf::from("content/index.md");
        let tmpl = PathBuf::from("templates/base.html");
        graph.add_dep(&page, &tmpl);

        // Change an unrelated file
        let unrelated = PathBuf::from("static/logo.png");
        let changed = vec![unrelated.clone()];
        let result = graph.invalidated_pages(&changed);
        // Only the changed file itself, not the page
        assert_eq!(result, vec![unrelated]);
    }
}

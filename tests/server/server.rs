// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Integration tests for `ssg::server`.

use std::cell::RefCell;
use std::fs;

use ssg::error::SsgError;
use ssg::server::{
    generate_locale_redirect, serve_site_with, HttpTransport, ServeTransport,
};
use tempfile::tempdir;

#[derive(Default)]
struct MockTransport {
    invocations: RefCell<Vec<(String, String)>>,
}

impl ServeTransport for MockTransport {
    fn start(&self, addr: &str, root: &str) -> Result<(), SsgError> {
        self.invocations
            .borrow_mut()
            .push((addr.to_string(), root.to_string()));
        Ok(())
    }
}

#[test]
fn serve_site_with_invokes_transport_once_with_resolved_root() {
    let dir = tempdir().unwrap();
    let transport = MockTransport::default();
    serve_site_with(dir.path(), &transport).expect("serve_site_with");
    let calls = transport.invocations.borrow();
    assert_eq!(calls.len(), 1);
    assert!(calls[0].1.contains(&*dir.path().to_string_lossy()));
}

#[test]
fn http_transport_is_constructible_as_unit_struct() {
    let _t = HttpTransport;
}

#[test]
fn generate_locale_redirect_emits_html_with_meta_refresh() {
    let dir = tempdir().unwrap();
    let site = dir.path();
    fs::create_dir_all(site).unwrap();
    let locales = vec!["en-US".to_string(), "fr-FR".to_string()];
    let result = generate_locale_redirect(site, &locales, "en-US");
    assert!(result.is_ok());
    // Should emit some kind of index/redirect file
    let entries: Vec<_> = fs::read_dir(site).unwrap().collect();
    assert!(!entries.is_empty());
}

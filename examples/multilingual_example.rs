#![allow(clippy::unwrap_used, clippy::expect_used)]
// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! # Multilingual Static Site Generator Example
//!
//! This example demonstrates how to generate a multilingual static site
//! with a language selector at the root of the `public` directory.

use anyhow::Result;
use http_handle::Server;
use ssg::plugin::{PluginContext, PluginManager};
use ssg::search::SearchPlugin;
use ssg::seo::SeoPlugin;
use staticdatagen::compiler::service::compile;
use std::fs;
use std::path::Path;

/// Supported locales as (code, native name) pairs.
/// Matches the language set offered on bankstatementparser.com.
const LANGUAGES: &[(&str, &str)] = &[
    ("en", "English"),
    ("fr", "Français"),
    ("ar", "العربية"),
    ("bn", "বাংলা"),
    ("cs", "Čeština"),
    ("de", "Deutsch"),
    ("es", "Español"),
    ("ha", "Hausa"),
    ("he", "עברית"),
    ("hi", "हिन्दी"),
    ("id", "Indonesia"),
    ("it", "Italiano"),
    ("ja", "日本語"),
    ("ko", "한국어"),
    ("nl", "Nederlands"),
    ("pl", "Polski"),
    ("pt", "Português"),
    ("ro", "Română"),
    ("ru", "Русский"),
    ("sv", "Svenska"),
    ("th", "ไทย"),
    ("tl", "Filipino"),
    ("tr", "Türkçe"),
    ("uk", "Українська"),
    ("vi", "Tiếng Việt"),
    ("yo", "Yorùbá"),
    ("zh", "简体中文"),
    ("zh-tw", "繁體中文"),
];

fn main() -> Result<()> {
    // Define supported languages
    let languages: Vec<&str> =
        LANGUAGES.iter().map(|(code, _)| *code).collect();

    // Root directory for public files
    let public_root = Path::new("./examples/public");
    fs::create_dir_all(public_root)?;

    // Generate sites for all languages
    for lang in &languages {
        println!("Processing language: {}", lang);

        // Define paths specific to the language
        let build_dir = Path::new("./examples/build").join(lang);
        let site_dir = public_root.join(lang);
        let content_dir = Path::new("./examples/content").join(lang);
        let template_dir = Path::new("./examples/templates").join(lang);

        // Call the compile function to generate the website
        println!("    🔍 Compiling content for language: {lang}...");
        match compile(&build_dir, &content_dir, &site_dir, &template_dir) {
            Ok(()) => println!(
                "    ✅ Successfully compiled static site for language: {lang}"
            ),
            Err(e) => {
                println!("    ❌ Error compiling site for {lang}: {e:?}");
                return Err(e);
            }
        }

        // Run plugins (SEO + Search) for this language
        let mut plugins = PluginManager::new();
        plugins.register(SeoPlugin);
        plugins.register(SearchPlugin);
        let ctx = PluginContext::new(
            &content_dir,
            &build_dir,
            &site_dir,
            &template_dir,
        );
        plugins.run_after_compile(&ctx)?;
        println!("    🔌 Plugins complete for {lang}");
    }

    // Promote English to the site root: copy every file from `public/en/`
    // into `public/` so that `/` serves English directly. Other locales remain
    // at `/<lang>/`. This mirrors the convention used by sites like
    // bankstatementparser.com where the default language has no path prefix.
    let en_root = public_root.join("en");
    if en_root.exists() {
        copy_dir_recursive(&en_root, public_root)?;
        println!("    ✅ Promoted English to site root");
    }

    // Serve the root public directory
    let server = Server::new("127.0.0.1:3000", public_root.to_str().unwrap());
    println!("Serving site at http://127.0.0.1:3000");
    let _ = server.start();

    Ok(())
}

/// Recursively copies every file from `src` into `dst`, creating `dst` and
/// any intermediate directories as needed. Existing files in `dst` are
/// overwritten so that promoting a locale to the site root is idempotent.
fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
    if !dst.exists() {
        fs::create_dir_all(dst)?;
    }
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let dst_path = dst.join(entry.file_name());
        if file_type.is_dir() {
            copy_dir_recursive(&entry.path(), &dst_path)?;
        } else {
            let _ = fs::copy(entry.path(), &dst_path)?;
        }
    }
    Ok(())
}

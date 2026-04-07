// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Image optimization plugin.
//!
//! Processes images to generate WebP variants and responsive `<picture>`
//! elements with `srcset`, `loading="lazy"`, and `decoding="async"`.

#[cfg(feature = "image-optimization")]
use crate::plugin::{Plugin, PluginContext};
#[cfg(feature = "image-optimization")]
use anyhow::{Context, Result};
#[cfg(feature = "image-optimization")]
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

/// Responsive image widths for srcset generation.
#[cfg(feature = "image-optimization")]
const WIDTHS: &[u32] = &[320, 640, 1024, 1920];

// WebP encoding quality is controlled by the image crate defaults.

/// Plugin that optimises images and rewrites HTML with `<picture>` tags.
///
/// Runs in `after_compile`:
/// 1. Scans `site_dir` for JPEG/PNG images
/// 2. Generates WebP variants at responsive widths
/// 3. Rewrites `<img>` tags to `<picture>` with `srcset`
/// 4. Adds `loading="lazy"`, `decoding="async"`, `width`, `height`
#[cfg(feature = "image-optimization")]
#[derive(Debug, Clone, Copy)]
pub struct ImageOptimizationPlugin;

#[cfg(feature = "image-optimization")]
impl Plugin for ImageOptimizationPlugin {
    fn name(&self) -> &'static str {
        "image-optimization"
    }

    fn after_compile(&self, ctx: &PluginContext) -> Result<()> {
        if !ctx.site_dir.exists() {
            return Ok(());
        }

        let images = collect_images(&ctx.site_dir)?;
        if images.is_empty() {
            return Ok(());
        }

        let optimized_dir = ctx.site_dir.join("optimized");
        fs::create_dir_all(&optimized_dir)?;

        let mut manifest: HashMap<String, ImageManifest> = HashMap::new();

        for img_path in &images {
            match process_image(img_path, &ctx.site_dir, &optimized_dir) {
                Ok(entry) => {
                    let _ = manifest.insert(entry.original_rel.clone(), entry);
                }
                Err(e) => {
                    log::warn!("[image] Failed to process {img_path:?}: {e}");
                }
            }
        }

        // Rewrite HTML files
        let html_files = collect_html_files(&ctx.site_dir)?;
        for html_path in &html_files {
            let html = fs::read_to_string(html_path)?;
            let rewritten = rewrite_img_tags(&html, &manifest);
            if rewritten != html {
                fs::write(html_path, rewritten)?;
            }
        }

        log::info!(
            "[image] Optimised {} image(s), {} variant(s) generated",
            manifest.len(),
            manifest.values().map(|m| m.variants.len()).sum::<usize>()
        );
        Ok(())
    }
}

#[cfg(feature = "image-optimization")]
#[derive(Debug, Clone)]
struct ImageVariant {
    rel_path: String,
    width: u32,
}

#[cfg(feature = "image-optimization")]
#[derive(Debug, Clone)]
struct ImageManifest {
    original_rel: String,
    original_width: u32,
    original_height: u32,
    variants: Vec<ImageVariant>,
}

/// Processes a single image: resize + encode to WebP at responsive widths.
#[cfg(feature = "image-optimization")]
fn process_image(
    img_path: &Path,
    site_dir: &Path,
    optimized_dir: &Path,
) -> Result<ImageManifest> {
    let img = image::open(img_path)
        .with_context(|| format!("Failed to open {img_path:?}"))?;

    let (orig_w, orig_h) = (img.width(), img.height());
    let rel = img_path
        .strip_prefix(site_dir)
        .unwrap_or(img_path)
        .to_string_lossy()
        .replace('\\', "/");

    let stem = img_path.file_stem().unwrap_or_default().to_string_lossy();

    let mut variants = Vec::new();

    for &width in WIDTHS {
        if width >= orig_w {
            continue; // Skip sizes larger than original
        }

        let ratio = f64::from(width) / f64::from(orig_w);
        let height = (f64::from(orig_h) * ratio) as u32;
        let resized = img.resize_exact(
            width,
            height,
            image::imageops::FilterType::Lanczos3,
        );

        // Save WebP variant
        let variant_name = format!("{stem}-{width}w.webp");
        let variant_path = optimized_dir.join(&variant_name);
        resized
            .save(&variant_path)
            .with_context(|| format!("Failed to save {variant_path:?}"))?;

        let variant_rel = format!("optimized/{variant_name}");
        variants.push(ImageVariant {
            rel_path: variant_rel,
            width,
        });
    }

    Ok(ImageManifest {
        original_rel: rel,
        original_width: orig_w,
        original_height: orig_h,
        variants,
    })
}

/// Rewrites `<img src="...">` tags to `<picture>` with srcset.
#[cfg(feature = "image-optimization")]
fn rewrite_img_tags(
    html: &str,
    manifest: &HashMap<String, ImageManifest>,
) -> String {
    let mut result = html.to_string();

    for (original_rel, entry) in manifest {
        if entry.variants.is_empty() {
            continue;
        }

        // Build srcset
        let srcset: String = entry
            .variants
            .iter()
            .map(|v| format!("/{} {}w", v.rel_path, v.width))
            .collect::<Vec<_>>()
            .join(", ");

        // Find and replace <img src="...original_rel...">
        let patterns = [
            format!("\"{original_rel}\""),
            format!("\"/{original_rel}\""),
        ];

        for pattern in &patterns {
            if let Some(img_start) = result.find(pattern) {
                // Find the <img that contains this src
                let search_back = &result[..img_start + pattern.len()];
                if let Some(tag_start) = search_back.rfind("<img") {
                    let tag_end = result[tag_start..]
                        .find('>')
                        .map_or(result.len(), |e| tag_start + e + 1);

                    let old_tag = &result[tag_start..tag_end];

                    // Extract existing alt attribute
                    let alt = extract_attr(old_tag, "alt").unwrap_or_default();

                    let picture = format!(
                        "<picture>\
                         <source type=\"image/webp\" \
                         srcset=\"{}\" \
                         sizes=\"(max-width: 640px) 100vw, (max-width: 1024px) 50vw, 33vw\">\
                         <img src=\"/{}\" alt=\"{}\" \
                         width=\"{}\" height=\"{}\" \
                         loading=\"lazy\" decoding=\"async\">\
                         </picture>",
                        srcset,
                        original_rel,
                        alt,
                        entry.original_width,
                        entry.original_height,
                    );

                    result = format!(
                        "{}{}{}",
                        &result[..tag_start],
                        picture,
                        &result[tag_end..]
                    );
                    break; // Only replace first occurrence per image
                }
            }
        }
    }

    result
}

#[cfg(feature = "image-optimization")]
fn extract_attr(tag: &str, attr: &str) -> Option<String> {
    let pattern = format!("{attr}=\"");
    let start = tag.find(&pattern)? + pattern.len();
    let end = tag[start..].find('"')? + start;
    Some(tag[start..end].to_string())
}

/// Collect all `.jpg`/`.jpeg`/`.png` files under `dir`, skipping any
/// that live inside an `optimized/` subdirectory (the plugin's own
/// output directory — must not be re-processed).
#[cfg(feature = "image-optimization")]
fn collect_images(dir: &Path) -> Result<Vec<PathBuf>> {
    let all = crate::walk::walk_files_multi(dir, &["jpg", "jpeg", "png"])?;
    Ok(all
        .into_iter()
        .filter(|p| !p.components().any(|c| c.as_os_str() == "optimized"))
        .collect())
}

#[cfg(feature = "image-optimization")]
fn collect_html_files(dir: &Path) -> Result<Vec<PathBuf>> {
    crate::walk::walk_files(dir, "html")
}

#[cfg(all(test, feature = "image-optimization"))]
mod tests {
    use super::*;
    use tempfile::tempdir;

    // -------------------------------------------------------------------
    // Test fixtures
    // -------------------------------------------------------------------

    /// Writes a tiny programmatically-generated JPEG to the given path.
    /// Uses the `image` crate's own encoder so no binary assets need to
    /// be checked into the repository.
    fn write_test_jpeg(path: &Path, w: u32, h: u32) {
        let buf = image::ImageBuffer::from_fn(w, h, |x, y| {
            image::Rgb([(x % 256) as u8, (y % 256) as u8, 128])
        });
        image::DynamicImage::ImageRgb8(buf)
            .save_with_format(path, image::ImageFormat::Jpeg)
            .expect("write jpeg");
    }

    /// Writes a tiny programmatically-generated PNG to the given path.
    fn write_test_png(path: &Path, w: u32, h: u32) {
        let buf = image::ImageBuffer::from_fn(w, h, |x, y| {
            image::Rgba([(x % 256) as u8, (y % 256) as u8, 200, 255])
        });
        image::DynamicImage::ImageRgba8(buf)
            .save_with_format(path, image::ImageFormat::Png)
            .expect("write png");
    }

    /// Builds an in-memory `ImageManifest` with the supplied variants.
    fn manifest_with(
        original_rel: &str,
        width: u32,
        height: u32,
        variant_widths: &[u32],
    ) -> HashMap<String, ImageManifest> {
        let stem = original_rel
            .rsplit('/')
            .next()
            .unwrap_or(original_rel)
            .rsplit('.')
            .nth(1)
            .unwrap_or("img");
        let variants = variant_widths
            .iter()
            .map(|&w| ImageVariant {
                rel_path: format!("optimized/{stem}-{w}w.webp"),
                width: w,
            })
            .collect();
        let mut m = HashMap::new();
        let _ = m.insert(
            original_rel.to_string(),
            ImageManifest {
                original_rel: original_rel.to_string(),
                original_width: width,
                original_height: height,
                variants,
            },
        );
        m
    }

    // -------------------------------------------------------------------
    // ImageOptimizationPlugin — derive surface
    // -------------------------------------------------------------------

    #[test]
    fn image_optimization_plugin_is_copy_after_move() {
        // Guards the `Copy` derive added in v0.0.34.
        let plugin = ImageOptimizationPlugin;
        let _consumed = plugin;
        assert_eq!(plugin.name(), "image-optimization");
    }

    #[test]
    fn name_returns_static_image_optimization_identifier() {
        assert_eq!(ImageOptimizationPlugin.name(), "image-optimization");
    }

    // -------------------------------------------------------------------
    // extract_attr — table-driven over the success / failure paths
    // -------------------------------------------------------------------

    #[test]
    fn extract_attr_table_driven_inputs() {
        let cases: &[(&str, &str, Option<&str>)] = &[
            (r#"<img src="x.jpg" alt="Photo">"#, "alt", Some("Photo")),
            (r#"<img src="x.jpg">"#, "alt", None),
            (r#"<img alt="">"#, "alt", Some("")),
            (
                r#"<img alt="multi word value">"#,
                "alt",
                Some("multi word value"),
            ),
            (r#"<img src="x.jpg" alt="P">"#, "src", Some("x.jpg")),
            (r#"<img>"#, "src", None),
        ];
        for &(tag, attr, expected) in cases {
            let actual = extract_attr(tag, attr);
            assert_eq!(
                actual.as_deref(),
                expected,
                "extract_attr({tag:?}, {attr:?}) should be {expected:?}"
            );
        }
    }

    // -------------------------------------------------------------------
    // rewrite_img_tags — happy path + edge cases
    // -------------------------------------------------------------------

    #[test]
    fn rewrite_img_tags_replaces_img_with_picture_element() {
        let manifest =
            manifest_with("images/photo.jpg", 2000, 1500, &[640, 1024]);
        let html = r#"<img src="images/photo.jpg" alt="A photo">"#;

        let result = rewrite_img_tags(html, &manifest);

        assert!(result.contains("<picture>"));
        assert!(result.contains("</picture>"));
        assert!(result.contains(r#"type="image/webp""#));
        assert!(result.contains("srcset="));
        assert!(result.contains("640w"));
        assert!(result.contains("1024w"));
        assert!(result.contains(r#"loading="lazy""#));
        assert!(result.contains(r#"decoding="async""#));
        assert!(result.contains(r#"width="2000""#));
        assert!(result.contains(r#"height="1500""#));
        assert!(result.contains(r#"alt="A photo""#));
    }

    #[test]
    fn rewrite_img_tags_preserves_alt_text() {
        // The alt-extraction round-trip at line 199 must propagate
        // accessibility text unchanged.
        let manifest = manifest_with("a.jpg", 2000, 1000, &[640]);
        let html = r#"<img src="a.jpg" alt="Important context">"#;
        let result = rewrite_img_tags(html, &manifest);
        assert!(result.contains(r#"alt="Important context""#));
    }

    #[test]
    fn rewrite_img_tags_handles_missing_alt_with_empty_string() {
        // The `unwrap_or_default()` at line 199 must produce alt=""
        // rather than panicking on the None case.
        let manifest = manifest_with("a.jpg", 2000, 1000, &[640]);
        let html = r#"<img src="a.jpg">"#;
        let result = rewrite_img_tags(html, &manifest);
        assert!(result.contains(r#"alt="""#));
    }

    #[test]
    fn rewrite_img_tags_handles_absolute_src_path() {
        // The `patterns` array at line 182 covers both `"path"` and
        // `"/path"` quote variants.
        let manifest = manifest_with("images/a.jpg", 2000, 1000, &[640]);
        let html = r#"<img src="/images/a.jpg" alt="">"#;
        let result = rewrite_img_tags(html, &manifest);
        assert!(result.contains("<picture>"));
    }

    #[test]
    fn rewrite_img_tags_no_match_returns_unchanged() {
        // If the manifest references an image that isn't actually in
        // the HTML, the input must be returned unchanged.
        let manifest = manifest_with("ghost.jpg", 100, 100, &[640]);
        let html = r#"<p>no images here</p>"#;
        let result = rewrite_img_tags(html, &manifest);
        assert_eq!(result, html);
    }

    #[test]
    fn rewrite_img_tags_skips_entries_with_no_variants() {
        // The `if entry.variants.is_empty() { continue }` guard at
        // line 169 — entries with zero variants should not produce
        // any rewrite.
        let manifest = manifest_with("a.jpg", 2000, 1000, &[]);
        let html = r#"<img src="a.jpg" alt="x">"#;
        let result = rewrite_img_tags(html, &manifest);
        assert_eq!(result, html, "no variants → no rewrite");
    }

    #[test]
    fn rewrite_img_tags_builds_srcset_with_width_descriptors() {
        // The srcset construction at line 174-179 should produce
        // entries of form `/<path> <width>w` joined by ", ".
        let manifest =
            manifest_with("a.jpg", 4000, 3000, &[320, 640, 1024, 1920]);
        let html = r#"<img src="a.jpg" alt="">"#;
        let result = rewrite_img_tags(html, &manifest);
        for w in [320, 640, 1024, 1920] {
            assert!(
                result.contains(&format!("{w}w")),
                "srcset should contain {w}w:\n{result}"
            );
        }
        assert!(result.matches(", ").count() >= 3);
    }

    #[test]
    fn rewrite_img_tags_only_replaces_first_occurrence_per_image() {
        // The `break` at line 223 limits the rewrite to one tag per
        // pattern per image — guards against runaway substitution.
        let manifest = manifest_with("a.jpg", 2000, 1000, &[640]);
        let html = r#"<img src="a.jpg"><img src="a.jpg">"#;
        let result = rewrite_img_tags(html, &manifest);
        // Exactly one <picture> wrapper.
        assert_eq!(result.matches("<picture>").count(), 1);
    }

    // -------------------------------------------------------------------
    // collect_images — extension filter + optimized-dir skip
    // -------------------------------------------------------------------

    #[test]
    fn collect_images_skips_optimized_subdirectory() {
        // The `current.file_name() == "optimized"` skip at line 250
        // prevents the plugin from re-processing its own output.
        let dir = tempdir().expect("tempdir");
        let site = dir.path().join("site");
        let opt = site.join("optimized");
        fs::create_dir_all(&opt).unwrap();

        fs::write(site.join("photo.jpg"), [0xFF, 0xD8]).unwrap();
        fs::write(opt.join("photo-640w.webp"), [0]).unwrap();

        let images = collect_images(&site).unwrap();
        assert_eq!(images.len(), 1);
        assert!(images[0].ends_with("photo.jpg"));
    }

    #[test]
    fn collect_images_filters_to_jpg_jpeg_png_only() {
        let dir = tempdir().expect("tempdir");
        for name in ["a.jpg", "b.jpeg", "c.png", "d.gif", "e.webp", "f.txt"] {
            fs::write(dir.path().join(name), [0]).unwrap();
        }
        let images = collect_images(dir.path()).unwrap();
        assert_eq!(images.len(), 3, "only jpg/jpeg/png should be collected");
    }

    #[test]
    fn collect_images_extension_match_is_case_insensitive() {
        // The `to_lowercase()` at line 259 must handle uppercase
        // extensions like `IMG.JPG` from camera exports.
        let dir = tempdir().expect("tempdir");
        for name in ["A.JPG", "B.PNG", "C.JPEG"] {
            fs::write(dir.path().join(name), [0]).unwrap();
        }
        let images = collect_images(dir.path()).unwrap();
        assert_eq!(images.len(), 3);
    }

    #[test]
    fn collect_images_recurses_into_nested_subdirectories() {
        let dir = tempdir().expect("tempdir");
        let nested = dir.path().join("a").join("b");
        fs::create_dir_all(&nested).unwrap();
        fs::write(dir.path().join("top.jpg"), [0]).unwrap();
        fs::write(nested.join("deep.png"), [0]).unwrap();

        let images = collect_images(dir.path()).unwrap();
        assert_eq!(images.len(), 2);
    }

    #[test]
    fn collect_images_returns_empty_for_missing_directory() {
        let dir = tempdir().expect("tempdir");
        let result = collect_images(&dir.path().join("missing")).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn collect_images_returns_results_sorted() {
        let dir = tempdir().expect("tempdir");
        for name in ["zebra.jpg", "apple.jpg", "mango.jpg"] {
            fs::write(dir.path().join(name), [0]).unwrap();
        }
        let images = collect_images(dir.path()).unwrap();
        let names: Vec<_> = images
            .iter()
            .map(|p| p.file_name().unwrap().to_str().unwrap())
            .collect();
        assert_eq!(names, vec!["apple.jpg", "mango.jpg", "zebra.jpg"]);
    }

    // -------------------------------------------------------------------
    // collect_html_files — recursion + filtering
    // -------------------------------------------------------------------

    #[test]
    fn collect_html_files_filters_non_html_extensions() {
        let dir = tempdir().expect("tempdir");
        fs::write(dir.path().join("a.html"), "").unwrap();
        fs::write(dir.path().join("b.css"), "").unwrap();

        let result = collect_html_files(dir.path()).unwrap();
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn collect_html_files_recurses_and_sorts() {
        let dir = tempdir().expect("tempdir");
        let nested = dir.path().join("blog");
        fs::create_dir_all(&nested).unwrap();
        fs::write(dir.path().join("index.html"), "").unwrap();
        fs::write(nested.join("post.html"), "").unwrap();

        let result = collect_html_files(dir.path()).unwrap();
        assert_eq!(result.len(), 2);
    }

    // -------------------------------------------------------------------
    // after_compile — short-circuit paths (no real images)
    // -------------------------------------------------------------------

    #[test]
    fn after_compile_missing_site_dir_returns_ok() {
        let dir = tempdir().expect("tempdir");
        let missing = dir.path().join("missing");
        let ctx =
            PluginContext::new(dir.path(), dir.path(), &missing, dir.path());
        ImageOptimizationPlugin
            .after_compile(&ctx)
            .expect("missing site is not an error");
        assert!(!missing.exists());
    }

    // -------------------------------------------------------------------
    // process_image — real JPEG/PNG round-trip
    // -------------------------------------------------------------------

    #[test]
    fn process_image_generates_webp_variants_below_original_width() {
        let dir = tempdir().expect("tempdir");
        let site = dir.path().join("site");
        let opt = site.join("optimized");
        fs::create_dir_all(&opt).unwrap();

        let src = site.join("hero.jpg");
        write_test_jpeg(&src, 2000, 1000);

        let manifest = process_image(&src, &site, &opt).unwrap();
        assert_eq!(manifest.original_width, 2000);
        assert_eq!(manifest.original_height, 1000);
        assert_eq!(manifest.original_rel, "hero.jpg");

        // Every WIDTH in WIDTHS that is strictly less than 2000 must
        // produce a variant. For WIDTHS = [320, 640, 1024, 1920]
        // that's all four.
        assert_eq!(manifest.variants.len(), 4);
        for v in &manifest.variants {
            assert!(opt
                .join(v.rel_path.trim_start_matches("optimized/"))
                .exists());
            assert!(v.width < 2000);
        }
    }

    #[test]
    fn process_image_skips_widths_larger_than_original() {
        // The `if width >= orig_w { continue; }` guard at line 126
        // must skip every WIDTH >= 500 (which is all of 640/1024/1920).
        let dir = tempdir().expect("tempdir");
        let site = dir.path().join("site");
        let opt = site.join("optimized");
        fs::create_dir_all(&opt).unwrap();

        let src = site.join("small.png");
        write_test_png(&src, 500, 500);

        let manifest = process_image(&src, &site, &opt).unwrap();
        // Only WIDTHS[0]=320 should survive (320 < 500).
        assert_eq!(manifest.variants.len(), 1);
        assert_eq!(manifest.variants[0].width, 320);
    }

    #[test]
    fn process_image_rejects_unreadable_source_path() {
        // The `image::open(path)` call at line 111 propagates Err
        // via `.with_context(...)`.
        let dir = tempdir().expect("tempdir");
        let opt = dir.path().join("opt");
        fs::create_dir_all(&opt).unwrap();
        let missing = dir.path().join("does-not-exist.jpg");
        assert!(process_image(&missing, dir.path(), &opt).is_err());
    }

    // -------------------------------------------------------------------
    // after_compile — end-to-end on real images
    // -------------------------------------------------------------------

    #[test]
    fn after_compile_processes_real_images_and_rewrites_html() {
        let dir = tempdir().expect("tempdir");
        let site = dir.path().join("site");
        let images = site.join("images");
        fs::create_dir_all(&images).unwrap();

        write_test_jpeg(&images.join("photo.jpg"), 2000, 1500);
        fs::write(
            site.join("index.html"),
            r#"<html><head></head><body><img src="/images/photo.jpg" alt="Test"></body></html>"#,
        )
        .unwrap();

        let ctx = PluginContext::new(dir.path(), dir.path(), &site, dir.path());
        ImageOptimizationPlugin.after_compile(&ctx).unwrap();

        // Original file preserved.
        assert!(images.join("photo.jpg").exists());
        // Optimized directory populated.
        assert!(site.join("optimized").exists());
        // HTML rewritten to <picture>.
        let html = fs::read_to_string(site.join("index.html")).unwrap();
        assert!(html.contains("<picture>"));
        assert!(html.contains("image/webp"));
        assert!(html.contains(r#"alt="Test""#));
    }

    #[test]
    fn after_compile_failed_image_processing_logs_warn_and_continues() {
        // Write a file that LOOKS like a jpg but isn't (invalid bytes).
        // `image::open` returns Err → plugin's match Err arm at line 63
        // logs and continues. Pipeline must still succeed overall.
        let dir = tempdir().expect("tempdir");
        let site = dir.path().join("site");
        fs::create_dir_all(&site).unwrap();

        fs::write(site.join("broken.jpg"), b"this is not really a jpeg")
            .unwrap();

        let ctx = PluginContext::new(dir.path(), dir.path(), &site, dir.path());
        ImageOptimizationPlugin
            .after_compile(&ctx)
            .expect("broken image must not propagate");
    }

    #[test]
    fn after_compile_no_images_short_circuits_without_creating_optimized_dir() {
        // The `images.is_empty()` early return at line 49 must not
        // create the `optimized/` directory.
        let dir = tempdir().expect("tempdir");
        let site = dir.path().join("site");
        fs::create_dir_all(&site).unwrap();
        // Add a non-image file to prove the empty check works on the
        // filtered set, not the raw directory listing.
        fs::write(site.join("index.html"), "<p></p>").unwrap();

        let ctx = PluginContext::new(dir.path(), dir.path(), &site, dir.path());
        ImageOptimizationPlugin.after_compile(&ctx).unwrap();
        assert!(!site.join("optimized").exists());
    }
}

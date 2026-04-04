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
/// 1. Scans site_dir for JPEG/PNG images
/// 2. Generates WebP variants at responsive widths
/// 3. Rewrites `<img>` tags to `<picture>` with `srcset`
/// 4. Adds `loading="lazy"`, `decoding="async"`, `width`, `height`
#[cfg(feature = "image-optimization")]
#[derive(Debug)]
pub struct ImageOptimizationPlugin;

#[cfg(feature = "image-optimization")]
impl Plugin for ImageOptimizationPlugin {
    fn name(&self) -> &str {
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
                    log::warn!(
                        "[image] Failed to process {:?}: {}",
                        img_path,
                        e
                    );
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
        .with_context(|| format!("Failed to open {:?}", img_path))?;

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

        let ratio = width as f64 / orig_w as f64;
        let height = (orig_h as f64 * ratio) as u32;
        let resized = img.resize_exact(
            width,
            height,
            image::imageops::FilterType::Lanczos3,
        );

        // Save WebP variant
        let variant_name = format!("{}-{}w.webp", stem, width);
        let variant_path = optimized_dir.join(&variant_name);
        resized
            .save(&variant_path)
            .with_context(|| format!("Failed to save {:?}", variant_path))?;

        let variant_rel = format!("optimized/{}", variant_name);
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
            format!("\"{}\"", original_rel),
            format!("\"/{original_rel}\""),
        ];

        for pattern in &patterns {
            if let Some(img_start) = result.find(pattern) {
                // Find the <img that contains this src
                let search_back = &result[..img_start + pattern.len()];
                if let Some(tag_start) = search_back.rfind("<img") {
                    let tag_end = result[tag_start..]
                        .find('>')
                        .map(|e| tag_start + e + 1)
                        .unwrap_or(result.len());

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
    let pattern = format!("{}=\"", attr);
    let start = tag.find(&pattern)? + pattern.len();
    let end = tag[start..].find('"')? + start;
    Some(tag[start..end].to_string())
}

#[cfg(feature = "image-optimization")]
fn collect_images(dir: &Path) -> Result<Vec<PathBuf>> {
    let image_exts = ["jpg", "jpeg", "png"];
    let mut files = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(current) = stack.pop() {
        if !current.is_dir() {
            continue;
        }
        // Skip the optimized directory
        if current.file_name().is_some_and(|n| n == "optimized") {
            continue;
        }
        for entry in fs::read_dir(&current)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if let Some(ext) = path.extension() {
                let ext_lower = ext.to_string_lossy().to_lowercase();
                if image_exts.contains(&ext_lower.as_str()) {
                    files.push(path);
                }
            }
        }
    }
    files.sort();
    Ok(files)
}

#[cfg(feature = "image-optimization")]
fn collect_html_files(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(current) = stack.pop() {
        if !current.is_dir() {
            continue;
        }
        for entry in fs::read_dir(&current)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|e| e == "html") {
                files.push(path);
            }
        }
    }
    files.sort();
    Ok(files)
}

#[cfg(all(test, feature = "image-optimization"))]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_rewrite_img_tags() {
        let mut manifest = HashMap::new();
        let _ = manifest.insert(
            "images/photo.jpg".to_string(),
            ImageManifest {
                original_rel: "images/photo.jpg".to_string(),
                original_width: 2000,
                original_height: 1500,
                variants: vec![
                    ImageVariant {
                        rel_path: "optimized/photo-640w.webp".to_string(),
                        width: 640,
                    },
                    ImageVariant {
                        rel_path: "optimized/photo-1024w.webp".to_string(),
                        width: 1024,
                    },
                ],
            },
        );

        let html = r#"<img src="images/photo.jpg" alt="A photo">"#;
        let result = rewrite_img_tags(html, &manifest);
        assert!(result.contains("<picture>"));
        assert!(result.contains("srcset="));
        assert!(result.contains("640w"));
        assert!(result.contains("1024w"));
        assert!(result.contains("loading=\"lazy\""));
        assert!(result.contains("width=\"2000\""));
    }

    #[test]
    fn test_extract_attr() {
        assert_eq!(
            extract_attr(r#"<img src="x.jpg" alt="Photo">"#, "alt"),
            Some("Photo".to_string())
        );
        assert_eq!(extract_attr(r#"<img src="x.jpg">"#, "alt"), None);
    }

    #[test]
    fn test_collect_images_skips_optimized() {
        let dir = tempdir().unwrap();
        let site = dir.path().join("site");
        let opt = site.join("optimized");
        fs::create_dir_all(&opt).unwrap();

        fs::write(site.join("photo.jpg"), &[0xFF, 0xD8]).unwrap();
        fs::write(opt.join("photo-640w.webp"), &[0]).unwrap();

        let images = collect_images(&site).unwrap();
        assert_eq!(images.len(), 1);
        assert!(images[0].ends_with("photo.jpg"));
    }
}

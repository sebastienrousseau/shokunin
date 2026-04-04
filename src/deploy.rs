// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Deployment adapter generation.
//!
//! Generates platform-specific configuration files for common hosting
//! providers, including cache headers and security headers.

use crate::plugin::{Plugin, PluginContext};
use anyhow::Result;
use std::fs;

/// Supported deployment targets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeployTarget {
    Netlify,
    Vercel,
    CloudflarePages,
    GithubPages,
}

/// Plugin that generates deployment configuration files.
#[derive(Debug)]
pub struct DeployPlugin {
    target: DeployTarget,
}

impl DeployPlugin {
    /// Creates a new `DeployPlugin` for the given target.
    pub fn new(target: DeployTarget) -> Self {
        Self { target }
    }
}

impl Plugin for DeployPlugin {
    fn name(&self) -> &str {
        "deploy"
    }

    fn after_compile(&self, ctx: &PluginContext) -> Result<()> {
        if !ctx.site_dir.exists() {
            return Ok(());
        }

        match self.target {
            DeployTarget::Netlify => generate_netlify(&ctx.site_dir)?,
            DeployTarget::Vercel => generate_vercel(&ctx.site_dir)?,
            DeployTarget::CloudflarePages => {
                generate_cloudflare(&ctx.site_dir)?;
            }
            DeployTarget::GithubPages => {
                generate_github_pages(&ctx.site_dir)?;
            }
        }

        log::info!("[deploy] Generated {:?} config", self.target);
        Ok(())
    }
}

/// Security headers shared across all platforms.
const SECURITY_HEADERS: &[(&str, &str)] = &[
    ("X-Content-Type-Options", "nosniff"),
    ("X-Frame-Options", "DENY"),
    ("X-XSS-Protection", "1; mode=block"),
    ("Referrer-Policy", "strict-origin-when-cross-origin"),
    (
        "Permissions-Policy",
        "camera=(), microphone=(), geolocation=()",
    ),
];

fn generate_netlify(site_dir: &std::path::Path) -> Result<()> {
    let mut headers = String::from("/*\n");
    for (k, v) in SECURITY_HEADERS {
        headers.push_str(&format!("  {} = {}\n", k, v));
    }
    headers.push_str(
        "\n/assets/*\n  Cache-Control: public, max-age=31536000, immutable\n",
    );
    headers.push_str("\n/*.html\n  Cache-Control: public, max-age=3600\n");

    fs::write(site_dir.join("_headers"), &headers)?;
    fs::write(site_dir.join("_redirects"), "")?;

    let toml = r#"[build]
  publish = "public"
  command = "cargo run -- -c content -o public -t templates"

[[headers]]
  for = "/assets/*"
  [headers.values]
    Cache-Control = "public, max-age=31536000, immutable"
"#;
    fs::write(site_dir.join("netlify.toml"), toml)?;
    Ok(())
}

fn generate_vercel(site_dir: &std::path::Path) -> Result<()> {
    let mut headers_arr = Vec::new();
    for (k, v) in SECURITY_HEADERS {
        headers_arr.push(serde_json::json!({"key": k, "value": v}));
    }

    let config = serde_json::json!({
        "headers": [
            {
                "source": "/(.*)",
                "headers": headers_arr
            },
            {
                "source": "/assets/(.*)",
                "headers": [{"key": "Cache-Control", "value": "public, max-age=31536000, immutable"}]
            }
        ]
    });

    let json = serde_json::to_string_pretty(&config)?;
    fs::write(site_dir.join("vercel.json"), json)?;
    Ok(())
}

fn generate_cloudflare(site_dir: &std::path::Path) -> Result<()> {
    let mut headers = String::from("/*\n");
    for (k, v) in SECURITY_HEADERS {
        headers.push_str(&format!("  {} : {}\n", k, v));
    }
    headers.push_str(
        "\n/assets/*\n  Cache-Control: public, max-age=31536000, immutable\n",
    );

    fs::write(site_dir.join("_headers"), &headers)?;
    fs::write(site_dir.join("_redirects"), "")?;
    Ok(())
}

fn generate_github_pages(site_dir: &std::path::Path) -> Result<()> {
    // .nojekyll prevents GitHub Pages from processing with Jekyll
    fs::write(site_dir.join(".nojekyll"), "")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_netlify_generation() {
        let dir = tempdir().unwrap();
        generate_netlify(dir.path()).unwrap();
        assert!(dir.path().join("_headers").exists());
        assert!(dir.path().join("netlify.toml").exists());

        let headers = fs::read_to_string(dir.path().join("_headers")).unwrap();
        assert!(headers.contains("X-Content-Type-Options"));
        assert!(headers.contains("immutable"));
    }

    #[test]
    fn test_vercel_generation() {
        let dir = tempdir().unwrap();
        generate_vercel(dir.path()).unwrap();
        let json = fs::read_to_string(dir.path().join("vercel.json")).unwrap();
        assert!(json.contains("X-Content-Type-Options"));
        assert!(json.contains("immutable"));
    }

    #[test]
    fn test_cloudflare_generation() {
        let dir = tempdir().unwrap();
        generate_cloudflare(dir.path()).unwrap();
        assert!(dir.path().join("_headers").exists());
    }

    #[test]
    fn test_github_pages_generation() {
        let dir = tempdir().unwrap();
        generate_github_pages(dir.path()).unwrap();
        assert!(dir.path().join(".nojekyll").exists());
    }

    #[test]
    fn test_deploy_plugin() {
        let dir = tempdir().unwrap();
        let site = dir.path().join("site");
        fs::create_dir_all(&site).unwrap();

        let ctx = PluginContext::new(dir.path(), dir.path(), &site, dir.path());

        let plugin = DeployPlugin::new(DeployTarget::GithubPages);
        plugin.after_compile(&ctx).unwrap();
        assert!(site.join(".nojekyll").exists());
    }
}

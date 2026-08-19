// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! SSG site configuration and builder.

use super::error::CliError;
use super::validation::{validate_path_safety, validate_url};
use super::{default_config, MAX_CONFIG_SIZE};
use clap::ArgMatches;
use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    str::FromStr,
};

/// Image-optimization tunables (issue #521). Surfaces the
/// `[image]` section of `ssg.toml`:
///
/// ```toml
/// [image]
/// avif_quality = 70   # 1..=100, default 70 (visually transparent)
/// lazy_avif    = false  # set true to skip AVIF for non-priority images
/// ```
///
/// Both fields are optional; absent fields fall back to the same
/// defaults baked into [`crate::plugins_group::image_plugin::ImageOptimizationPlugin`].
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ImageConfig {
    /// AVIF encoding quality (1..=100). Defaults to 70.
    #[serde(default = "default_avif_quality")]
    pub avif_quality: u8,
    /// If true, AVIF encoding is skipped for images without a
    /// `priority="high"` shortcode marker — see issue #521 AC5.
    /// Defaults to false (AVIF for every responsive variant).
    #[serde(default)]
    pub lazy_avif: bool,
}

const fn default_avif_quality() -> u8 {
    70
}

impl Default for ImageConfig {
    fn default() -> Self {
        Self {
            avif_quality: default_avif_quality(),
            lazy_avif: false,
        }
    }
}

/// Edge-runtime header emitter configuration (issue #550). Surfaces
/// the `[edge_headers]` section of `ssg.toml`:
///
/// ```toml
/// [edge_headers]
/// targets = ["cloudflare", "netlify", "vercel"]
///
/// [edge_headers.overrides]
/// permissions-policy = "geolocation=(self)"
/// ```
///
/// When `targets` is empty (the default), the
/// [`crate::postprocess::EdgeHeadersPlugin`] is a no-op: nothing is
/// emitted into `dist/`. Listing one or more of `"cloudflare"`,
/// `"netlify"`, or `"vercel"` opts in to per-platform header config
/// generation; unknown target strings are logged and ignored.
///
/// `overrides` is a header-name → header-value map (case-insensitive
/// on the platform side, but stored verbatim) that lets a site author
/// replace any of the five baseline headers without recompiling.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EdgeHeadersConfig {
    /// Edge platforms to emit configuration for. Recognised values:
    /// `"cloudflare"`, `"netlify"`, `"vercel"`. Anything else is
    /// logged and skipped. Empty (the default) disables the plugin.
    #[serde(default)]
    pub targets: Vec<String>,
    /// Header-name → header-value overrides applied on top of the
    /// baseline defaults. Names are matched case-insensitively when
    /// the emitter merges overrides, so `"permissions-policy"`,
    /// `"Permissions-Policy"`, and `"PERMISSIONS-POLICY"` all win.
    #[serde(default)]
    pub overrides: BTreeMap<String, String>,
}

impl EdgeHeadersConfig {
    /// Returns `true` when at least one valid target is configured —
    /// the registration site in `register_default_plugins` uses this to
    /// decide whether to register the emitter.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ssg::cmd::EdgeHeadersConfig;
    ///
    /// let cfg = EdgeHeadersConfig::default();
    /// // No targets configured by default.
    /// assert!(!cfg.is_enabled());
    /// ```
    #[must_use]
    pub const fn is_enabled(&self) -> bool {
        !self.targets.is_empty()
    }
}

/// Digest algorithm used for Subresource Integrity `integrity=`
/// attributes on externalized assets (v0.0.47 plan §3 item 2.3).
///
/// Applies to the SRI attributes emitted by the fingerprint plugin
/// (`crate::assets::FingerprintPlugin`) and the CSP inline-extraction
/// plugin (`crate::csp::CspPlugin`). It deliberately does **not**
/// govern CSP *directive source hashes* (the `'sha256-…'` entries
/// inside a Content-Security-Policy header/meta value) — those stay
/// SHA-256 for the broadest UA compatibility.
///
/// Serialized in `ssg.toml` as the lowercase strings `"sha256"`,
/// `"sha384"`, and `"sha512"`; kept in lockstep with the
/// `security.sri_algorithm` enum in `ssg.schema.json`.
///
/// # Examples
///
/// ```rust
/// use ssg::cmd::SriAlgorithm;
///
/// // SHA-384 is the default, matching the documented posture.
/// assert_eq!(SriAlgorithm::default(), SriAlgorithm::Sha384);
///
/// // Every emitted integrity value starts with the algorithm prefix.
/// let sri = SriAlgorithm::Sha512.integrity(b"body{margin:0}");
/// assert!(sri.starts_with("sha512-"));
/// ```
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize,
)]
#[serde(rename_all = "lowercase")]
pub enum SriAlgorithm {
    /// SHA-256 — the pre-v0.0.47 behaviour, kept for back-compat.
    Sha256,
    /// SHA-384 — the default; matches the README/SECURITY.md claim.
    #[default]
    Sha384,
    /// SHA-512 — the strongest digest the SRI spec admits.
    Sha512,
}

impl SriAlgorithm {
    /// Returns the SRI prefix token for this algorithm
    /// (`"sha256"`, `"sha384"`, or `"sha512"`).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ssg::cmd::SriAlgorithm;
    ///
    /// assert_eq!(SriAlgorithm::default().prefix(), "sha384");
    /// assert_eq!(SriAlgorithm::Sha256.prefix(), "sha256");
    /// ```
    #[must_use]
    pub const fn prefix(self) -> &'static str {
        match self {
            Self::Sha256 => "sha256",
            Self::Sha384 => "sha384",
            Self::Sha512 => "sha512",
        }
    }

    /// Computes the full SRI attribute value for `data`:
    /// `<prefix>-<base64(digest(data))>`.
    ///
    /// Browsers compare the `integrity` attribute against
    /// `base64(digest(body))` per the
    /// [W3C SRI spec](https://www.w3.org/TR/SRI/#the-integrity-attribute),
    /// so the returned string is exactly what a UA will validate the
    /// response body against.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ssg::cmd::SriAlgorithm;
    ///
    /// // SHA-384("") — well-known empty-input digest.
    /// assert_eq!(
    ///     SriAlgorithm::Sha384.integrity(b""),
    ///     "sha384-OLBgp1GsljhM2TJ+sbHjaiH9txEUvgdDTAzHv2P24donTt6/529l+9Ua0vFImLlb"
    /// );
    /// ```
    #[must_use]
    pub fn integrity(self, data: &[u8]) -> String {
        use base64::{engine::general_purpose::STANDARD, Engine as _};
        use sha2::{Digest as _, Sha256, Sha384, Sha512};

        let b64 = match self {
            Self::Sha256 => STANDARD.encode(Sha256::digest(data)),
            Self::Sha384 => STANDARD.encode(Sha384::digest(data)),
            Self::Sha512 => STANDARD.encode(Sha512::digest(data)),
        };
        format!("{}-{}", self.prefix(), b64)
    }
}

/// Security tunables (v0.0.47 plan §3 item 2.3). Surfaces the
/// `[security]` section of `ssg.toml`:
///
/// ```toml
/// [security]
/// sri_algorithm = "sha384"   # "sha256" | "sha384" | "sha512"
/// ```
///
/// Absent section (the default) means SHA-384 SRI, matching the
/// documented posture in README/SECURITY.md.
///
/// # Examples
///
/// ```rust
/// use ssg::cmd::{SecurityConfig, SriAlgorithm};
///
/// // The default posture is SHA-384 SRI.
/// let cfg = SecurityConfig::default();
/// assert_eq!(cfg.sri_algorithm, SriAlgorithm::Sha384);
///
/// // `[security] sri_algorithm = "sha512"` in ssg.toml deserializes
/// // into the strongest digest the SRI spec admits.
/// let cfg: SecurityConfig =
///     toml::from_str("sri_algorithm = \"sha512\"").unwrap();
/// assert_eq!(cfg.sri_algorithm, SriAlgorithm::Sha512);
/// ```
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Digest algorithm for `integrity=` attributes on externalized
    /// assets. Defaults to [`SriAlgorithm::Sha384`].
    #[serde(default)]
    pub sri_algorithm: SriAlgorithm,
}

/// Core configuration for the static site generator.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SsgConfig {
    /// Name of the site.
    pub site_name: String,
    /// Directory containing content files.
    pub content_dir: PathBuf,
    /// Directory for generated output files.
    pub output_dir: PathBuf,
    /// Directory containing template files.
    pub template_dir: PathBuf,
    /// Optional directory for development server files.
    pub serve_dir: Option<PathBuf>,
    /// Base URL of the site.
    pub base_url: String,
    /// Title of the site.
    pub site_title: String,
    /// Description of the site.
    pub site_description: String,
    /// Language code for the site.
    pub language: String,
    /// Optional i18n configuration for multi-locale sites.
    #[serde(default)]
    pub i18n: Option<crate::i18n::I18nConfig>,
    /// Optional CDN prefix for markdown images.
    #[serde(default)]
    pub cdn_prefix: Option<String>,
    /// Optional site-wide fallback `og:image` (a URL or site-relative
    /// path). Used by generated pages that have no per-page image of
    /// their own — currently the taxonomy/tag pages emitted by
    /// [`crate::taxonomy::TaxonomyPlugin`], which bypass the
    /// `SeoPlugin` transform chain (#586) and so never see the
    /// front-matter-derived `og:image` that regular content pages get.
    /// Absent ⇒ no `og:image` tag on those pages.
    #[serde(default)]
    pub og_image: Option<String>,
    /// Optional image-pipeline tunables (issue #521).
    #[serde(default)]
    pub image: ImageConfig,
    /// Edge-runtime header emitter config (issue #550). Absent /
    /// empty `targets` disables the emitter.
    #[serde(default)]
    pub edge_headers: EdgeHeadersConfig,
    /// Agentic-discovery emitters: `agents.txt`, `ai-plugin.json`, and
    /// the MCP registry (issue #552). All three are opt-in per the
    /// `[agents]` section of `ssg.toml`. Absent ⇒ no files written.
    #[serde(default)]
    pub agents: Option<
        crate::plugins_group::postprocess::agentic_discovery::AgentsConfig,
    >,
    /// Opt-in View Transitions + lazy-nav client (issue #547).
    ///
    /// When `true`, the build emits `_transitions/ssg-transitions.js`
    /// and injects a small `<script>` + `<style>` block into every
    /// page so same-origin navigations animate via the View
    /// Transitions API (Chromium/Safari) or fall back to a plain
    /// reload in non-supporting browsers (Firefox stable as of
    /// 2026-06). Persistent `<header>` / `<footer>` roots get
    /// `view-transition-name` so they don't animate across boundaries.
    /// Defaults to `false` to keep zero-JS sites zero-JS.
    #[serde(default)]
    pub transitions: bool,
    /// Skip generating taxonomy (tag / category / topic) pages.
    ///
    /// Defaults to `false`, so a build that does not ask for this is
    /// unchanged. Sites that curate their own taxonomy — a canonical
    /// vocabulary, a minimum-article threshold, hand-translated slugs —
    /// need to own `/tags/` outright: emitting a page per raw
    /// front-matter term contradicts that curation, and on a
    /// multi-locale corpus it multiplies the URL surface with thin
    /// pages. Opting out is cheaper and more honest than deleting the
    /// output afterwards.
    #[serde(default)]
    pub no_taxonomy_pages: bool,
    /// Security tunables (v0.0.47 plan §3 item 2.3): the `[security]`
    /// section of `ssg.toml`. Currently holds the SRI digest
    /// algorithm; absent ⇒ SHA-384.
    #[serde(default)]
    pub security: SecurityConfig,
}

impl Default for SsgConfig {
    fn default() -> Self {
        default_config().as_ref().clone()
    }
}

impl SsgConfig {
    /// Applies command-line arguments to override defaults.
    fn override_with_cli(
        mut self,
        matches: &ArgMatches,
    ) -> Result<Self, CliError> {
        // If `-n/--new` was used
        if let Some(site_name) = matches.get_one::<String>("new") {
            self.site_name.clone_from(site_name);
        }

        // If `-c/--content` was used
        if let Some(content_dir) = matches.get_one::<PathBuf>("content") {
            self.content_dir.clone_from(content_dir);
        }

        // If `-o/--output` was used
        if let Some(output_dir) = matches.get_one::<PathBuf>("output") {
            self.output_dir.clone_from(output_dir);
        }

        // If `-t/--template` was used
        if let Some(template_dir) = matches.get_one::<PathBuf>("template") {
            self.template_dir.clone_from(template_dir);
        }

        // If `-s/--serve` was used
        if let Some(serve_dir) = matches.get_one::<PathBuf>("serve") {
            self.serve_dir = Some(serve_dir.clone());
        }

        // `--no-tag-pages` / SSG_NO_TAG_PAGES. Only ever turns generation
        // off — absent means the default, so no existing build changes
        // behaviour merely by upgrading.
        if matches.get_flag("no_tag_pages") {
            self.no_taxonomy_pages = true;
        }

        // `--watch` flag is handled by the caller (run() in lib.rs)

        // Re-validate after overriding
        self.validate()?;
        Ok(self)
    }
    /// Creates a configuration by merging the default values with any command-line arguments.
    ///
    /// # Arguments
    /// * `matches` - Parsed command-line arguments from Clap.
    ///
    /// # Errors
    /// Returns a [`CliError`] if:
    /// - A path fails validation (e.g., directory traversal or symlink).
    /// - A URL is malformed.
    /// - The language is incorrectly formatted.
    ///
    /// # Examples
    /// ```rust,ignore
    /// let matches = cli.build().get_matches();
    /// let config = SsgConfig::from_matches(&matches)?;
    /// ```
    pub fn from_matches(matches: &ArgMatches) -> Result<Self, CliError> {
        if let Some(config_path) = matches.get_one::<PathBuf>("config") {
            let loaded_config = Self::from_file(config_path)?;
            return Ok(loaded_config);
        }

        // 1) Start with defaults
        let config = Self::default();

        // 2) Override them with CLI flags
        let config = config.override_with_cli(matches)?;

        // 3) Return the result
        Ok(config)
    }

    /// Subcommand variant: subcommand parsers re-use the same
    /// `--config / --content / --output / --template / --serve` flag
    /// names but omit the legacy `--new` (project scaffolding is its
    /// own command). The override logic is identical otherwise, so we
    /// just delegate through a thin shim that skips the missing
    /// `--new` lookup.
    ///
    /// # Errors
    /// Returns [`CliError`] under the same conditions as
    /// [`Self::from_matches`].
    pub fn from_subcommand_matches(
        sub_m: &ArgMatches,
    ) -> Result<Self, CliError> {
        if let Some(config_path) = sub_m.get_one::<PathBuf>("config") {
            return Self::from_file(config_path);
        }

        let mut config = Self::default();
        if let Some(content_dir) = sub_m.get_one::<PathBuf>("content") {
            config.content_dir.clone_from(content_dir);
        }
        if let Some(output_dir) = sub_m.get_one::<PathBuf>("output") {
            config.output_dir.clone_from(output_dir);
        }
        if let Some(template_dir) = sub_m.get_one::<PathBuf>("template") {
            config.template_dir.clone_from(template_dir);
        }
        // `dev` exposes `--serve`; `build` / `check` / `deploy` do not.
        if sub_m.try_contains_id("serve").unwrap_or(false) {
            if let Some(serve_dir) = sub_m.get_one::<PathBuf>("serve") {
                config.serve_dir = Some(serve_dir.clone());
            }
        }
        config.validate()?;
        Ok(config)
    }
    /// Loads configuration from a TOML file, enforcing a maximum file size limit.
    ///
    /// # Arguments
    /// * `path` - The path of the TOML file to be read.
    ///
    /// # Errors
    /// Returns a [`CliError`] if:
    /// - The file cannot be read or exceeds `MAX_CONFIG_SIZE`.
    /// - The file is malformed TOML.
    /// - Any fields fail validation afterward.
    ///
    /// # Examples
    /// ```rust,ignore
    /// let config = SsgConfig::from_file(Path::new("config.toml"))?;
    /// ```
    pub fn from_file(path: &Path) -> Result<Self, CliError> {
        let metadata = fs::metadata(path)?;
        if metadata.len() > MAX_CONFIG_SIZE as u64 {
            return Err(CliError::ValidationError(format!(
                "Config file too large (max {MAX_CONFIG_SIZE} bytes)"
            )));
        }

        let content = fs::read_to_string(path)?;
        let config: Self = toml::from_str(&content)?;
        config.validate()?;
        Ok(config)
    }

    /// Validates the configuration's URLs and paths.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ssg::cmd::SsgConfig;
    ///
    /// let cfg = SsgConfig::default();
    /// assert!(cfg.validate().is_ok());
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`CliError::ValidationError`] when `site_name` is empty,
    /// or path/URL safety checks fail.
    pub fn validate(&self) -> Result<(), CliError> {
        debug!("Validating config: {self:?}");

        if self.site_name.trim().is_empty() {
            error!("site_name cannot be empty");
            return Err(CliError::ValidationError(
                "site_name cannot be empty".into(),
            ));
        }

        if !self.base_url.is_empty() {
            validate_url(&self.base_url)?;
        }

        validate_path_safety(&self.content_dir, "content_dir")?;
        validate_path_safety(&self.output_dir, "output_dir")?;
        validate_path_safety(&self.template_dir, "template_dir")?;
        if let Some(ref serve_dir) = self.serve_dir {
            validate_path_safety(serve_dir, "serve_dir")?;
        }

        info!("Config validation successful");
        Ok(())
    }

    /// Returns a fresh [`SsgConfigBuilder`] for fluent construction.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ssg::cmd::SsgConfig;
    ///
    /// let cfg = SsgConfig::builder()
    ///     .site_name("My Site".into())
    ///     .build()
    ///     .expect("valid config");
    /// assert_eq!(cfg.site_name, "My Site");
    /// ```
    #[must_use]
    pub fn builder() -> SsgConfigBuilder {
        SsgConfigBuilder::default()
    }
}

impl FromStr for SsgConfig {
    type Err = CliError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let config: Self = toml::from_str(s)?;
        config.validate()?;
        Ok(config)
    }
}

/// Builder for `SsgConfig`.
#[derive(Debug, Clone, Default)]
pub struct SsgConfigBuilder {
    config: SsgConfig,
}

/// # Examples
/// ```
/// use ssg::cmd::SsgConfig;
/// let config = SsgConfig::builder()
///     .site_name("My Site".to_string())
///     .base_url("http://example.com".to_string())
///     .build()
///     .unwrap();
/// ```
impl SsgConfigBuilder {
    /// Sets the site name for the configuration.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ssg::cmd::SsgConfig;
    ///
    /// let cfg = SsgConfig::builder().site_name("Hello".into()).build().unwrap();
    /// assert_eq!(cfg.site_name, "Hello");
    /// ```
    #[must_use]
    pub fn site_name(mut self, name: String) -> Self {
        self.config.site_name = name;
        self
    }
    /// Sets the base URL for the configuration.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ssg::cmd::SsgConfig;
    ///
    /// let cfg = SsgConfig::builder()
    ///     .base_url("https://example.com".into())
    ///     .build()
    ///     .unwrap();
    /// assert_eq!(cfg.base_url, "https://example.com");
    /// ```
    #[must_use]
    pub fn base_url(mut self, url: String) -> Self {
        self.config.base_url = url;
        self
    }
    /// Sets the content directory for the configuration.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ssg::cmd::SsgConfig;
    /// use std::path::PathBuf;
    ///
    /// let cfg = SsgConfig::builder()
    ///     .content_dir(PathBuf::from("docs"))
    ///     .build()
    ///     .unwrap();
    /// assert_eq!(cfg.content_dir, PathBuf::from("docs"));
    /// ```
    #[must_use]
    pub fn content_dir(mut self, dir: PathBuf) -> Self {
        self.config.content_dir = dir;
        self
    }
    /// Sets the output directory for the configuration.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ssg::cmd::SsgConfig;
    /// use std::path::PathBuf;
    ///
    /// let cfg = SsgConfig::builder()
    ///     .output_dir(PathBuf::from("dist"))
    ///     .build()
    ///     .unwrap();
    /// assert_eq!(cfg.output_dir, PathBuf::from("dist"));
    /// ```
    #[must_use]
    pub fn output_dir(mut self, dir: PathBuf) -> Self {
        self.config.output_dir = dir;
        self
    }
    /// Sets the template directory for the configuration.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ssg::cmd::SsgConfig;
    /// use std::path::PathBuf;
    ///
    /// let cfg = SsgConfig::builder()
    ///     .template_dir(PathBuf::from("tpl"))
    ///     .build()
    ///     .unwrap();
    /// assert_eq!(cfg.template_dir, PathBuf::from("tpl"));
    /// ```
    #[must_use]
    pub fn template_dir(mut self, dir: PathBuf) -> Self {
        self.config.template_dir = dir;
        self
    }
    /// Sets the optional development server directory for the configuration.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ssg::cmd::SsgConfig;
    /// use std::path::PathBuf;
    ///
    /// let cfg = SsgConfig::builder()
    ///     .serve_dir(Some(PathBuf::from("public")))
    ///     .build()
    ///     .unwrap();
    /// assert_eq!(cfg.serve_dir, Some(PathBuf::from("public")));
    /// ```
    #[must_use]
    pub fn serve_dir(mut self, dir: Option<PathBuf>) -> Self {
        self.config.serve_dir = dir;
        self
    }
    /// Sets the site title for the configuration.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ssg::cmd::SsgConfig;
    ///
    /// let cfg = SsgConfig::builder().site_title("Title".into()).build().unwrap();
    /// assert_eq!(cfg.site_title, "Title");
    /// ```
    #[must_use]
    pub fn site_title(mut self, title: String) -> Self {
        self.config.site_title = title;
        self
    }
    /// Sets the site description for the configuration.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ssg::cmd::SsgConfig;
    ///
    /// let cfg = SsgConfig::builder().site_description("Demo".into()).build().unwrap();
    /// assert_eq!(cfg.site_description, "Demo");
    /// ```
    #[must_use]
    pub fn site_description(mut self, desc: String) -> Self {
        self.config.site_description = desc;
        self
    }
    /// Sets the language code for the configuration.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ssg::cmd::SsgConfig;
    ///
    /// let cfg = SsgConfig::builder().language("fr-FR".into()).build().unwrap();
    /// assert_eq!(cfg.language, "fr-FR");
    /// ```
    #[must_use]
    pub fn language(mut self, lang: String) -> Self {
        self.config.language = lang;
        self
    }
    /// Sets the i18n configuration.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ssg::cmd::SsgConfig;
    ///
    /// let cfg = SsgConfig::builder().i18n(None).build().unwrap();
    /// assert!(cfg.i18n.is_none());
    /// ```
    #[must_use]
    pub fn i18n(mut self, i18n: Option<crate::i18n::I18nConfig>) -> Self {
        self.config.i18n = i18n;
        self
    }
    /// Sets the CDN prefix configuration.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ssg::cmd::SsgConfig;
    ///
    /// let cfg = SsgConfig::builder()
    ///     .cdn_prefix(Some("https://cdn.example.com".into()))
    ///     .build()
    ///     .unwrap();
    /// assert!(cfg.cdn_prefix.is_some());
    /// ```
    #[must_use]
    pub fn cdn_prefix(mut self, prefix: Option<String>) -> Self {
        self.config.cdn_prefix = prefix;
        self
    }
    /// Sets the site-wide fallback `og:image` used by generated
    /// taxonomy/tag pages that have no per-page image of their own.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ssg::cmd::SsgConfig;
    ///
    /// let cfg = SsgConfig::builder()
    ///     .og_image(Some("/social/default.png".into()))
    ///     .build()
    ///     .unwrap();
    /// assert_eq!(cfg.og_image.as_deref(), Some("/social/default.png"));
    /// ```
    #[must_use]
    pub fn og_image(mut self, og_image: Option<String>) -> Self {
        self.config.og_image = og_image;
        self
    }
    /// Sets the edge-headers emitter configuration (issue #550).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ssg::cmd::{EdgeHeadersConfig, SsgConfig};
    ///
    /// let cfg = SsgConfig::builder()
    ///     .edge_headers(EdgeHeadersConfig::default())
    ///     .build()
    ///     .unwrap();
    /// assert!(!cfg.edge_headers.is_enabled());
    /// ```
    #[must_use]
    pub fn edge_headers(mut self, edge: EdgeHeadersConfig) -> Self {
        self.config.edge_headers = edge;
        self
    }
    /// Sets the security tunables (v0.0.47 plan §3 item 2.3).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ssg::cmd::{SecurityConfig, SriAlgorithm, SsgConfig};
    ///
    /// let cfg = SsgConfig::builder()
    ///     .security(SecurityConfig {
    ///         sri_algorithm: SriAlgorithm::Sha512,
    ///     })
    ///     .build()
    ///     .unwrap();
    /// assert_eq!(cfg.security.sri_algorithm, SriAlgorithm::Sha512);
    /// ```
    #[must_use]
    pub const fn security(mut self, security: SecurityConfig) -> Self {
        self.config.security = security;
        self
    }
    /// Enables the View Transitions + lazy-nav client (issue #547).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ssg::cmd::SsgConfig;
    ///
    /// let cfg = SsgConfig::builder().transitions(true).build().unwrap();
    /// assert!(cfg.transitions);
    /// ```
    #[must_use]
    pub const fn transitions(mut self, enabled: bool) -> Self {
        self.config.transitions = enabled;
        self
    }
    /// Builds the final `SsgConfig` instance.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ssg::cmd::SsgConfig;
    ///
    /// let cfg = SsgConfig::builder().build().expect("default is valid");
    /// assert!(!cfg.site_name.is_empty());
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`CliError::ValidationError`] when [`SsgConfig::validate`] fails.
    pub fn build(self) -> Result<SsgConfig, CliError> {
        self.config.validate()?;
        Ok(self.config)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use crate::cmd::Cli;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    /// Region-free variant of `assert!(matches!(err, <Variant>))` —
    /// `matches!` would leave its never-taken false arm uncovered.
    fn assert_err_variant<T: std::fmt::Debug>(
        result: Result<T, CliError>,
        variant: &str,
    ) {
        let err = result.expect_err("expected an error");
        let repr = format!("{err:?}");
        assert!(repr.starts_with(variant), "expected {variant}, got {repr}");
    }

    #[test]
    fn test_config_validation() {
        let config = SsgConfig::builder().site_name(String::new()).build();
        assert_err_variant(config, "ValidationError");
    }

    #[test]
    fn test_config_file_size_limit() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("large.toml");
        let mut file = File::create(&config_path).unwrap();

        write!(file, "{}", "x".repeat(MAX_CONFIG_SIZE + 1)).unwrap();

        assert_err_variant(
            SsgConfig::from_file(&config_path),
            "ValidationError",
        );
    }

    #[test]
    fn test_config_from_str() {
        let config_str = r#"
    site_name = "test"
    content_dir = "./examples/content"
    output_dir = "./examples/public"
    template_dir = "./examples/templates"
    base_url = "http://example.com"
    site_title = "Test Site"
    site_description = "Test Description"
    language = "en-GB"
    "#;

        let config: Result<SsgConfig, _> = config_str.parse();
        assert!(config.is_ok());
    }

    #[test]
    fn test_config_builder_all_fields() {
        let temp_dir = tempdir().unwrap();
        let serve_dir = temp_dir.path().join("serve");

        fs::create_dir_all(&serve_dir).unwrap();

        let config = SsgConfig::builder()
            .site_name("test".to_string())
            .base_url("http://example.com".to_string())
            .content_dir(PathBuf::from("./examples/content"))
            .output_dir(PathBuf::from("./examples/public"))
            .template_dir(PathBuf::from("./examples/templates"))
            .serve_dir(Some(serve_dir))
            .site_title("Test Site".to_string())
            .site_description("Test Desc".to_string())
            .language("en-GB".to_string())
            .build();

        assert!(config.is_ok());
    }

    #[test]
    fn test_invalid_config_file() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("invalid.toml");
        let mut file = File::create(&config_path).unwrap();
        write!(file, "invalid toml content").unwrap();

        assert_err_variant(SsgConfig::from_file(&config_path), "TomlError");
    }

    #[test]
    fn test_from_matches() {
        let matches = Cli::build().get_matches_from(vec!["ssg"]);
        let config = SsgConfig::from_matches(&matches);
        assert!(config.is_ok());
    }

    #[test]
    fn test_config_builder_empty_required_fields() {
        let config = SsgConfig::builder()
            .site_name(String::new())
            .site_title(String::new())
            .build();
        assert_err_variant(config, "ValidationError");
    }

    #[test]
    fn test_config_file_not_found() {
        let non_existent = Path::new("non_existent.toml");
        assert_err_variant(SsgConfig::from_file(non_existent), "IoError");
    }

    #[test]
    fn test_from_matches_with_config_file() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        let config_content = r#"
site_name = "from-file"
content_dir = "./examples/content"
output_dir = "./examples/public"
template_dir = "./examples/templates"
base_url = "http://example.com"
site_title = "File Site"
site_description = "From file"
language = "en-GB"
"#;
        fs::write(&config_path, config_content).unwrap();

        let cmd = Cli::build();
        let matches = cmd.get_matches_from(vec![
            "ssg",
            "--config",
            config_path.to_str().unwrap(),
        ]);
        let config = SsgConfig::from_matches(&matches).unwrap();
        assert_eq!(config.site_name, "from-file");
    }

    #[test]
    fn test_override_with_cli_all_flags() {
        let cmd = Cli::build();
        let matches = cmd.get_matches_from(vec![
            "ssg",
            "--new",
            "cli-site",
            "--content",
            "./examples/content",
            "--output",
            "./examples/public",
            "--template",
            "./examples/templates",
            "--serve",
            "./examples/public",
        ]);
        let config = SsgConfig::from_matches(&matches).unwrap();
        assert_eq!(config.site_name, "cli-site");
        assert_eq!(config.content_dir, PathBuf::from("./examples/content"));
        assert_eq!(config.output_dir, PathBuf::from("./examples/public"));
        assert_eq!(config.template_dir, PathBuf::from("./examples/templates"));
        assert!(config.serve_dir.is_some());
    }

    #[test]
    fn test_override_with_watch_flag() {
        let cmd = Cli::build();
        let matches = cmd.get_matches_from(vec!["ssg", "--watch"]);
        let config = SsgConfig::from_matches(&matches).unwrap();
        assert!(!config.site_name.is_empty());
    }

    #[test]
    fn test_validate_empty_url() {
        let config = SsgConfig::builder()
            .site_name("test".to_string())
            .base_url(String::new())
            .build();
        assert!(config.is_ok());
    }

    // -----------------------------------------------------------------
    // SsgConfig::from_file -- valid TOML
    // -----------------------------------------------------------------

    #[test]
    fn test_config_from_file_valid_toml() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("valid.toml");
        let toml_content = r#"
site_name = "TestSite"
content_dir = "./examples/content"
output_dir = "./examples/public"
template_dir = "./examples/templates"
base_url = "http://test.example.com"
site_title = "Test Title"
site_description = "A test site"
language = "en-GB"
"#;
        fs::write(&config_path, toml_content).unwrap();

        let config = SsgConfig::from_file(&config_path).unwrap();
        assert_eq!(config.site_name, "TestSite");
        assert_eq!(config.site_title, "Test Title");
        assert_eq!(config.base_url, "http://test.example.com");
    }

    // -----------------------------------------------------------------
    // SsgConfigBuilder::i18n / cdn_prefix
    // -----------------------------------------------------------------

    #[test]
    fn builder_sets_i18n() {
        let i18n_cfg = crate::i18n::I18nConfig {
            default_locale: "en".into(),
            locales: vec!["en".into(), "fr".into()],
            url_prefix: crate::i18n::UrlPrefixStrategy::SubPath,
        };
        let cfg = SsgConfig::builder()
            .site_name("t".to_string())
            .base_url("http://example.com".to_string())
            .i18n(Some(i18n_cfg.clone()))
            .build()
            .unwrap();
        assert!(cfg.i18n.is_some());
        assert_eq!(cfg.i18n.as_ref().unwrap().default_locale, "en");
    }

    #[test]
    fn builder_sets_cdn_prefix() {
        let cfg = SsgConfig::builder()
            .site_name("t".to_string())
            .base_url("http://example.com".to_string())
            .cdn_prefix(Some("https://cdn.example.com".into()))
            .build()
            .unwrap();
        assert_eq!(cfg.cdn_prefix.as_deref(), Some("https://cdn.example.com"));
    }

    #[test]
    fn builder_cdn_prefix_none_is_default() {
        let cfg = SsgConfig::builder()
            .site_name("t".to_string())
            .base_url("http://example.com".to_string())
            .cdn_prefix(None)
            .build()
            .unwrap();
        assert!(cfg.cdn_prefix.is_none());
    }

    // -----------------------------------------------------------------
    // [security] sri_algorithm (v0.0.47 plan §3 item 2.3)
    // -----------------------------------------------------------------

    #[test]
    fn security_section_absent_defaults_to_sha384() {
        let config_str = r#"
    site_name = "test"
    content_dir = "./examples/content"
    output_dir = "./examples/public"
    template_dir = "./examples/templates"
    base_url = "http://example.com"
    site_title = "Test Site"
    site_description = "Test Description"
    language = "en-GB"
    "#;
        let cfg: SsgConfig = config_str.parse().unwrap();
        assert_eq!(cfg.security.sri_algorithm, SriAlgorithm::Sha384);
    }

    #[test]
    fn security_sri_algorithm_parses_all_enum_values() {
        for (raw, expected) in [
            ("sha256", SriAlgorithm::Sha256),
            ("sha384", SriAlgorithm::Sha384),
            ("sha512", SriAlgorithm::Sha512),
        ] {
            let config_str = format!(
                r#"
    site_name = "test"
    content_dir = "./examples/content"
    output_dir = "./examples/public"
    template_dir = "./examples/templates"
    base_url = "http://example.com"
    site_title = "Test Site"
    site_description = "Test Description"
    language = "en-GB"

    [security]
    sri_algorithm = "{raw}"
    "#
            );
            let cfg: SsgConfig = config_str.parse().unwrap();
            assert_eq!(cfg.security.sri_algorithm, expected, "raw = {raw}");
        }
    }

    #[test]
    fn security_sri_algorithm_rejects_unknown_value() {
        let config_str = r#"
    site_name = "test"
    content_dir = "./examples/content"
    output_dir = "./examples/public"
    template_dir = "./examples/templates"
    base_url = "http://example.com"
    site_title = "Test Site"
    site_description = "Test Description"
    language = "en-GB"

    [security]
    sri_algorithm = "md5"
    "#;
        let cfg: Result<SsgConfig, CliError> = config_str.parse();
        assert_err_variant(cfg, "TomlError");
    }

    // -----------------------------------------------------------------
    // Error-path propagation
    // -----------------------------------------------------------------

    #[test]
    fn from_matches_rejects_invalid_content_override() {
        // An invalid --content path fails override_with_cli's
        // re-validation, covering both `?` propagation sites.
        let matches =
            Cli::build().get_matches_from(vec!["ssg", "--content", "bad<dir"]);
        assert_err_variant(SsgConfig::from_matches(&matches), "InvalidPath");
    }

    #[test]
    fn from_matches_propagates_missing_config_file_error() {
        let matches = Cli::build().get_matches_from(vec![
            "ssg",
            "--config",
            "/nonexistent/ssg-test-config.toml",
        ]);
        assert_err_variant(SsgConfig::from_matches(&matches), "IoError");
    }

    #[test]
    fn from_subcommand_matches_rejects_invalid_content_override() {
        let (_inv, matches) =
            Cli::parse_and_dispatch(["ssg", "build", "--content", "bad<dir"])
                .unwrap();
        let sub = matches.subcommand_matches("build").unwrap();
        assert_err_variant(
            SsgConfig::from_subcommand_matches(sub),
            "InvalidPath",
        );
    }

    #[test]
    fn from_subcommand_matches_dev_without_serve_keeps_none() {
        // `dev` exposes --serve; leaving it unset covers the inner
        // `if let Some(serve_dir)` miss branch.
        let (_inv, matches) = Cli::parse_and_dispatch(["ssg", "dev"]).unwrap();
        let sub = matches.subcommand_matches("dev").unwrap();
        let cfg = SsgConfig::from_subcommand_matches(sub).unwrap();
        assert!(cfg.serve_dir.is_none());
    }

    #[test]
    fn from_file_fails_when_path_is_a_directory() {
        // metadata() succeeds but read_to_string() fails, covering the
        // read error propagation distinct from the not-found case.
        let dir = tempdir().unwrap();
        assert_err_variant(SsgConfig::from_file(dir.path()), "IoError");
    }

    #[test]
    fn from_file_propagates_validation_failure() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("invalid-fields.toml");
        fs::write(
            &path,
            r#"
site_name = ""
content_dir = "./examples/content"
output_dir = "./examples/public"
template_dir = "./examples/templates"
base_url = "http://example.com"
site_title = "T"
site_description = "D"
language = "en-GB"
"#,
        )
        .unwrap();
        assert_err_variant(SsgConfig::from_file(&path), "ValidationError");
    }

    #[test]
    fn from_str_propagates_validation_failure() {
        let config_str = r#"
    site_name = ""
    content_dir = "./examples/content"
    output_dir = "./examples/public"
    template_dir = "./examples/templates"
    base_url = "http://example.com"
    site_title = "T"
    site_description = "D"
    language = "en-GB"
    "#;
        let cfg: Result<SsgConfig, CliError> = config_str.parse();
        assert_err_variant(cfg, "ValidationError");
    }

    #[test]
    fn validate_rejects_invalid_base_url() {
        let cfg = SsgConfig::builder()
            .site_name("t".to_string())
            .base_url("ftp://example.com".to_string())
            .build();
        assert_err_variant(cfg, "InvalidUrl");
    }

    #[test]
    fn validate_rejects_invalid_content_dir() {
        let cfg = SsgConfig::builder()
            .site_name("t".to_string())
            .content_dir(PathBuf::from("bad<content"))
            .build();
        assert_err_variant(cfg, "InvalidPath");
    }

    #[test]
    fn validate_rejects_invalid_output_dir() {
        let cfg = SsgConfig::builder()
            .site_name("t".to_string())
            .output_dir(PathBuf::from("bad<output"))
            .build();
        assert_err_variant(cfg, "InvalidPath");
    }

    #[test]
    fn validate_rejects_invalid_template_dir() {
        let cfg = SsgConfig::builder()
            .site_name("t".to_string())
            .template_dir(PathBuf::from("bad<template"))
            .build();
        assert_err_variant(cfg, "InvalidPath");
    }

    #[test]
    fn validate_rejects_invalid_serve_dir() {
        let cfg = SsgConfig::builder()
            .site_name("t".to_string())
            .serve_dir(Some(PathBuf::from("bad<serve")))
            .build();
        assert_err_variant(cfg, "InvalidPath");
    }

    #[test]
    fn builder_transitions_flag_round_trips() {
        let on = SsgConfig::builder().transitions(true).build().unwrap();
        assert!(on.transitions);
        let off = SsgConfig::builder().transitions(false).build().unwrap();
        assert!(!off.transitions);
    }

    // -----------------------------------------------------------------
    // SsgConfig::from_subcommand_matches
    // -----------------------------------------------------------------

    #[test]
    fn from_subcommand_matches_returns_defaults_when_no_overrides() {
        let (_inv, matches) =
            Cli::parse_and_dispatch(["ssg", "build"]).unwrap();
        let sub = matches.subcommand_matches("build").unwrap();
        let cfg = SsgConfig::from_subcommand_matches(sub).unwrap();
        // Defaults preserved.
        assert_eq!(cfg.content_dir, PathBuf::from("content"));
        assert_eq!(cfg.output_dir, PathBuf::from("public"));
        assert_eq!(cfg.template_dir, PathBuf::from("templates"));
        assert!(cfg.serve_dir.is_none());
    }

    #[test]
    fn from_subcommand_matches_applies_content_output_template_overrides() {
        let (_inv, matches) = Cli::parse_and_dispatch([
            "ssg",
            "build",
            "--content",
            "/c",
            "--output",
            "/o",
            "--template",
            "/t",
        ])
        .unwrap();
        let sub = matches.subcommand_matches("build").unwrap();
        let cfg = SsgConfig::from_subcommand_matches(sub).unwrap();
        assert_eq!(cfg.content_dir, PathBuf::from("/c"));
        assert_eq!(cfg.output_dir, PathBuf::from("/o"));
        assert_eq!(cfg.template_dir, PathBuf::from("/t"));
    }

    #[test]
    fn from_subcommand_matches_picks_up_serve_for_dev_subcommand() {
        let (_inv, matches) =
            Cli::parse_and_dispatch(["ssg", "dev", "--serve", "/srv"]).unwrap();
        let sub = matches.subcommand_matches("dev").unwrap();
        let cfg = SsgConfig::from_subcommand_matches(sub).unwrap();
        assert_eq!(cfg.serve_dir, Some(PathBuf::from("/srv")));
    }

    #[test]
    fn from_subcommand_matches_check_subcommand_has_no_serve() {
        // `check` doesn't expose `--serve` — code path goes through
        // try_contains_id == false branch.
        let (_inv, matches) =
            Cli::parse_and_dispatch(["ssg", "check"]).unwrap();
        let sub = matches.subcommand_matches("check").unwrap();
        let cfg = SsgConfig::from_subcommand_matches(sub).unwrap();
        assert!(cfg.serve_dir.is_none());
    }

    #[test]
    fn from_subcommand_matches_loads_config_file_when_present() {
        let dir = tempdir().unwrap();
        let cfg_path = dir.path().join("c.toml");
        fs::write(
            &cfg_path,
            r#"
site_name = "FromSub"
content_dir = "./examples/content"
output_dir = "./examples/public"
template_dir = "./examples/templates"
base_url = "http://sub.example.com"
site_title = "Sub Title"
site_description = "Sub Desc"
language = "en-GB"
"#,
        )
        .unwrap();

        let (_inv, matches) = Cli::parse_and_dispatch([
            "ssg",
            "build",
            "--config",
            cfg_path.to_str().unwrap(),
        ])
        .unwrap();
        let sub = matches.subcommand_matches("build").unwrap();
        let cfg = SsgConfig::from_subcommand_matches(sub).unwrap();
        assert_eq!(cfg.site_name, "FromSub");
        assert_eq!(cfg.base_url, "http://sub.example.com");
    }
}

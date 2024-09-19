// engine.rs

use crate::cache::Cache;
use crate::{Context, TemplateError};
use std::collections::HashMap;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tempfile::tempdir;

#[derive(Debug, Default, PartialEq, Eq, Clone)]
/// ## Struct: `PageOptions` - Options for rendering a page template
///
/// This struct contains the options for rendering a page template.
/// These options are used to construct a context `HashMap` that is
/// passed to the `render_template` function.
pub struct PageOptions<'a> {
    /// Elements of the page
    pub elements: HashMap<&'a str, &'a str>,
}

impl<'a> PageOptions<'a> {
    /// Creates a new `PageOptions` instance.
    pub fn new() -> PageOptions<'a> {
        PageOptions {
            elements: HashMap::new(),
        }
    }

    /// Sets a page option in the `elements` map.
    pub fn set(&mut self, key: &'a str, value: &'a str) {
        self.elements.insert(key, value);
    }

    /// Retrieves a page option from the `elements` map.
    pub fn get(&self, key: &'a str) -> Option<&&'a str> {
        self.elements.get(key)
    }
}

/// The main template rendering engine.
pub struct Engine {
    template_path: String,
    render_cache: Cache<String, String>,
}

impl Engine {
    /// Creates a new `Engine` instance.
    ///
    /// # Arguments
    /// * `template_path` - The path to the template directory.
    /// * `cache_ttl` - Time-to-live for cached rendered templates.
    pub fn new(template_path: &str, cache_ttl: Duration) -> Self {
        Self {
            template_path: template_path.to_string(),
            render_cache: Cache::new(cache_ttl),
        }
    }

    /// Renders a page using the specified layout and context, with caching.
    ///
    /// # Arguments
    /// * `context` - The rendering context.
    /// * `layout` - The layout to use for rendering.
    ///
    /// # Returns
    /// A `Result` containing the rendered page or a `TemplateError`.
    pub fn render_page(
        &mut self,
        context: &Context,
        layout: &str,
    ) -> Result<String, TemplateError> {
        let cache_key = format!("{}:{}", layout, context.hash());

        if let Some(cached) = self.render_cache.get(&cache_key) {
            return Ok(cached.to_string());
        }

        let template_path = Path::new(&self.template_path)
            .join(format!("{}.html", layout));
        let template_content = fs::read_to_string(&template_path)
            .map_err(TemplateError::Io)?;

        let rendered =
            self.render_template(&template_content, &context.elements)?;
        self.render_cache.insert(cache_key, rendered.clone());

        Ok(rendered)
    }

    /// Renders a template string with the given context.
    ///
    /// # Arguments
    /// * `template` - The template string.
    /// * `context` - The context for rendering.
    ///
    /// # Returns
    /// A `Result` containing the rendered string or a `TemplateError`.
    pub fn render_template(
        &self,
        template: &str,
        context: &HashMap<String, String>,
    ) -> Result<String, TemplateError> {
        if template.trim().is_empty() {
            return Err(TemplateError::RenderError(
                "Template is empty".to_string(),
            ));
        }

        // Detect and flag single curly braces as invalid
        if template.contains('{') && !template.contains("{{") {
            return Err(TemplateError::RenderError(
                "Invalid template format: single curly braces detected"
                    .to_string(),
            ));
        }

        let mut output = template.to_owned();
        for (key, value) in context {
            output = output.replace(&format!("{{{{{}}}}}", key), value);
        }

        // Check for any unresolved template tags (double curly braces)
        if output.contains("{{") {
            return Err(TemplateError::RenderError(format!(
                "Failed to render template, unresolved or invalid template tags: {}",
                output
            )));
        }

        Ok(output)
    }

    /// Creates or uses an existing template folder.
    ///
    /// # Arguments
    /// * `template_path` - An optional path to the template folder.
    ///
    /// # Returns
    /// A `Result` containing the template folder path or a `TemplateError`.
    pub fn create_template_folder(
        &self,
        template_path: Option<&str>,
    ) -> Result<String, TemplateError> {
        let current_dir =
            std::env::current_dir().map_err(TemplateError::Io)?;

        let template_dir_path = match template_path {
            Some(path) if is_url(path) => {
                self.download_files_from_url(path)?
            }
            Some(path) => {
                let local_path = current_dir.join(path);
                if local_path.exists() && local_path.is_dir() {
                    local_path
                } else {
                    return Err(TemplateError::Io(
                        std::io::Error::new(
                            std::io::ErrorKind::NotFound,
                            format!(
                                "Template directory not found: {}",
                                path
                            ),
                        ),
                    ));
                }
            }
            None => {
                let default_url = "https://raw.githubusercontent.com/sebastienrousseau/shokunin/main/template/";
                self.download_files_from_url(default_url)?
            }
        };

        Ok(template_dir_path.to_str().unwrap().to_string())
    }

    /// Helper function to download files from a URL and save to a directory.
    ///
    /// # Arguments
    /// * `url` - The URL to download files from.
    ///
    /// # Returns
    /// A `Result` containing the path to the directory or a `TemplateError`.
    fn download_files_from_url(
        &self,
        url: &str,
    ) -> Result<PathBuf, TemplateError> {
        let template_dir_path =
            tempdir().map_err(TemplateError::Io)?.into_path();

        let files = [
            "contact.html",
            "index.html",
            "page.html",
            "post.html",
            "main.js",
            "sw.js",
        ];

        for file in files.iter() {
            self.download_file(url, file, &template_dir_path)?;
        }

        Ok(template_dir_path)
    }

    /// Downloads a single file from a URL to the given directory.
    ///
    /// # Arguments
    /// * `url` - The base URL.
    /// * `file` - The file to download.
    /// * `dir` - The directory to save the file.
    pub fn download_file(
        &self,
        url: &str,
        file: &str,
        dir: &Path,
    ) -> Result<(), TemplateError> {
        let file_url = format!("{}/{}", url, file);
        let file_path = dir.join(file);
        let mut response = reqwest::blocking::get(&file_url)
            .map_err(TemplateError::Reqwest)?;
        let mut file =
            File::create(&file_path).map_err(TemplateError::Io)?;
        response
            .copy_to(&mut file)
            .map_err(TemplateError::Reqwest)?;

        Ok(())
    }
}

/// Utility function to check if a given path is a URL.
fn is_url(path: &str) -> bool {
    path.starts_with("http://") || path.starts_with("https://")
}

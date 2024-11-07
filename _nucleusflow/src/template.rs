//! # Template Rendering Module
//!
//! This module provides functionality for rendering templates using various engines.

use crate::{NucleusFlowError, Result, TemplateRenderer};
use handlebars::Handlebars;
use std::fs;
use std::path::Path;

/// Handlebars template renderer
#[derive(Debug)]
pub struct HandlebarsRenderer {
    handlebars: Handlebars<'static>,
}

impl HandlebarsRenderer {
    /// Create a new HandlebarsRenderer instance
    ///
    /// This function initializes a new Handlebars renderer and registers all templates
    /// found in the specified template directory.
    ///
    /// # Arguments
    ///
    /// * `template_dir` - A Path reference to the directory containing Handlebars templates
    ///
    /// # Returns
    ///
    /// * `Result<Self>` - A Result containing the HandlebarsRenderer instance if successful,
    ///   or an error if template registration fails
    pub fn new(template_dir: &Path) -> Result<Self> {
        let mut handlebars = Handlebars::new();

        // Register all .hbs files in the template directory
        Self::register_templates(&mut handlebars, template_dir)?;

        Ok(Self { handlebars })
    }

    /// Helper function to register all .hbs files in a directory
    fn register_templates(
        handlebars: &mut Handlebars<'static>,
        dir: &Path,
    ) -> Result<()> {
        for entry in fs::read_dir(dir).map_err(|e| {
            NucleusFlowError::TemplateRendering(e.to_string())
        })? {
            let entry = entry.map_err(|e| {
                NucleusFlowError::TemplateRendering(e.to_string())
            })?;
            let path = entry.path();
            if path.is_file()
                && path.extension().and_then(|s| s.to_str())
                    == Some("hbs")
            {
                let template_name = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .ok_or_else(|| {
                        NucleusFlowError::TemplateRendering(
                            "Invalid template filename".to_string(),
                        )
                    })?;
                let template_content = fs::read_to_string(&path)
                    .map_err(|e| {
                        NucleusFlowError::TemplateRendering(
                            e.to_string(),
                        )
                    })?;
                handlebars
                    .register_template_string(
                        template_name,
                        template_content,
                    )
                    .map_err(|e| {
                        NucleusFlowError::TemplateRendering(
                            e.to_string(),
                        )
                    })?;
            }
        }
        Ok(())
    }
}

impl TemplateRenderer for HandlebarsRenderer {
    /// Render a template with the given context
    ///
    /// This function renders the specified template using the provided context data.
    ///
    /// # Arguments
    ///
    /// * `template` - A string slice containing the name of the template to render
    /// * `context` - A reference to a serde_json::Value containing the context data for rendering
    ///
    /// # Returns
    ///
    /// * `Result<String>` - A Result containing the rendered template as a string if successful,
    ///   or an error if rendering fails
    fn render(
        &self,
        template: &str,
        context: &serde_json::Value,
    ) -> Result<String> {
        self.handlebars.render(template, context).map_err(|e| {
            NucleusFlowError::TemplateRendering(e.to_string())
        })
    }
}

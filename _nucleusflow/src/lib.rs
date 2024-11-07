// Copyright © 2024 NucleusFlow. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! # NucleusFlow
//!
//! `nucleusflow` is a library for building static site generators and content processing pipelines. It provides a flexible architecture for content processing, templating, and output generation.

use std::fs;
use std::{fmt, path::Path};
use thiserror::Error;

/// The cli module provides functionality for building and handling the command-line interface.
pub mod cli;
/// The config module provides functionality for handling configuration loading and parsing.
pub mod config;
/// The content module provides functionality for processing content in various formats.
pub mod content;
/// The output module provides functionality for generating output in various formats.
pub mod output;
/// The process module provides functionality for processing parsed arguments and executing core actions.
pub mod process;
/// The template module provides functionality for rendering templates.
pub mod template;

/// Custom error type for NucleusFlow operations
#[derive(Error, Debug)]
pub enum NucleusFlowError {
    /// IO errors
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    /// Configuration errors
    #[error("Configuration error: {0}")]
    Config(String),
    /// Content processing errors
    #[error("Content processing error: {0}")]
    ContentProcessing(String),
    /// Template rendering errors
    #[error("Template rendering error: {0}")]
    TemplateRendering(String),
    /// Output generation errors
    #[error("Output generation error: {0}")]
    OutputGeneration(String),
    /// Process errors
    #[error("Process error: {0}")]
    Process(String),
}

/// Result type alias for NucleusFlow operations
pub type Result<T> = std::result::Result<T, NucleusFlowError>;

/// Trait for content processors
pub trait ContentProcessor {
    /// Process content and return the processed output
    ///
    /// # Arguments
    ///
    /// * `content` - A string slice containing the content to be processed
    ///
    /// # Returns
    ///
    /// A `Result` containing the processed content as a `String` if successful,
    /// or a `NucleusFlowError` if processing fails
    fn process(&self, content: &str) -> Result<String>;
}

/// Trait for template renderers
pub trait TemplateRenderer {
    /// Render a template with the given context
    ///
    /// # Arguments
    ///
    /// * `template` - A string slice containing the name of the template to render
    /// * `context` - A reference to a `serde_json::Value` containing the context data for rendering
    ///
    /// # Returns
    ///
    /// A `Result` containing the rendered content as a `String` if successful,
    /// or a `NucleusFlowError` if rendering fails
    fn render(
        &self,
        template: &str,
        context: &serde_json::Value,
    ) -> Result<String>;
}

/// Trait for output generators
pub trait OutputGenerator {
    /// Generate output and write it to the specified path
    ///
    /// # Arguments
    ///
    /// * `content` - A string slice containing the content to be written
    /// * `output_path` - A `Path` reference specifying where to write the output
    ///
    /// # Returns
    ///
    /// A `Result` indicating success (`Ok((()))`) or containing a `NucleusFlowError` if the operation fails
    fn generate(&self, content: &str, output_path: &Path)
        -> Result<()>;
}

/// Configuration for the static site generator
#[derive(Debug, Clone)]
pub struct NucleusFlowConfig {
    /// Path to the content directory
    pub content_dir: std::path::PathBuf,
    /// Path to the output directory
    pub output_dir: std::path::PathBuf,
    /// Path to the template directory
    pub template_dir: std::path::PathBuf,
}

/// The main struct
pub struct NucleusFlow {
    config: NucleusFlowConfig,
    content_processor: Box<dyn ContentProcessor>,
    template_renderer: Box<dyn TemplateRenderer>,
    output_generator: Box<dyn OutputGenerator>,
}

impl fmt::Debug for NucleusFlow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NucleusFlow")
            .field("config", &self.config)
            .field("content_processor", &"<dyn ContentProcessor>")
            .field("template_renderer", &"<dyn TemplateRenderer>")
            .field("output_generator", &"<dyn OutputGenerator>")
            .finish()
    }
}

impl NucleusFlow {
    /// Create a new NucleusFlow instance with the given configuration and components
    pub fn new(
        config: NucleusFlowConfig,
        content_processor: Box<dyn ContentProcessor>,
        template_renderer: Box<dyn TemplateRenderer>,
        output_generator: Box<dyn OutputGenerator>,
    ) -> Self {
        Self {
            config,
            content_processor,
            template_renderer,
            output_generator,
        }
    }

    /// Generate the static site
    pub fn generate(&self) -> Result<()> {
        // Read content files
        let content_dir = &self.config.content_dir;
        for entry in fs::read_dir(content_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                // Read the content
                let content = fs::read_to_string(&path)?;

                // Process the content
                let processed_content =
                    self.content_processor.process(&content)?;

                // Render the template
                let context = serde_json::json!({
                    "content": processed_content,
                    // Add more context variables as needed
                });
                let template_name = "default";
                let rendered_content = self
                    .template_renderer
                    .render(template_name, &context)?;

                // Generate the output
                let relative_path =
                    path.strip_prefix(content_dir).unwrap();
                let output_path = self
                    .config
                    .output_dir
                    .join(relative_path)
                    .with_extension("html");
                self.output_generator
                    .generate(&rendered_content, &output_path)?;
            }
        }

        Ok(())
    }
}

/// Runs the NucleusFlow static site generator.
///
/// This function initializes the environment logger, prints the banner, builds the command-line interface,
/// processes the arguments, and returns a `Result` indicating success or an error.
///
/// # Parameters
///
/// None.
///
/// # Returns
///
/// * `Result<()>`: A `Result` indicating success (`Ok(())`) or an error (`Err(NucleusFlowError::Process)`)
///   if processing the arguments fails.
pub fn run() -> Result<()> {
    env_logger::init();
    cli::print_banner();
    let cli = cli::build().get_matches();
    process::args(&cli)
        .map_err(|e| NucleusFlowError::Process(e.to_string()))
}

#[cfg(test)]
mod integration_tests {
    use std::process::Command;

    #[test]
    fn test_banner_display() {
        let output = Command::new("cargo")
            .arg("run")
            .output()
            .expect("Failed to execute command");

        let stdout = String::from_utf8_lossy(&output.stdout);
        println!("{}", stdout);

        assert!(
            stdout.contains("NucleusFlow 🦀 v0.0.1"),
            "Banner did not print expected text"
        );
        assert!(stdout.contains(
            "A powerful Rust library for content processing, enabling static site generation, document conversion, and templating."
        ));
    }
}

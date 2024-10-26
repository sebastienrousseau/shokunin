//! # Output Generation Module
//!
//! This module provides functionality for generating output in various formats.

use crate::{NucleusFlowError, OutputGenerator, Result};
use std::fs;
use std::path::Path;

/// HTML output generator
#[derive(Debug, Copy, Clone)]
pub struct HtmlGenerator;

impl HtmlGenerator {
    /// Create a new HtmlGenerator instance
    pub fn new() -> Self {
        Self
    }
}

impl Default for HtmlGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl OutputGenerator for HtmlGenerator {
    /// Generate HTML output and write it to a file
    ///
    /// This function takes the processed content and writes it to the specified output path.
    ///
    /// # Arguments
    ///
    /// * `content` - A string slice containing the HTML content to be written
    /// * `output_path` - A Path reference specifying where to write the output file
    ///
    /// # Returns
    ///
    /// * `Result<()>` - A Result indicating success (Ok) or containing an error if the write operation fails
    fn generate(
        &self,
        content: &str,
        output_path: &Path,
    ) -> Result<()> {
        fs::write(output_path, content).map_err(|e| {
            NucleusFlowError::OutputGeneration(e.to_string())
        })
    }
}

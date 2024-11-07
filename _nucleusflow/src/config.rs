//! # Configuration Module
//!
//! This module handles loading and parsing of the Shokunin configuration.

use serde::Deserialize;
use std::fs;
use std::path::Path;
use toml;

use crate::{NucleusFlowConfig, NucleusFlowError, Result};

/// Configuration structure for Shokunin
#[derive(Debug, Deserialize)]
struct Config {
    content_dir: String,
    output_dir: String,
    template_dir: String,
}

/// Load the configuration from a TOML file
///
/// This function reads the `shokunin.toml` file from the current directory,
/// parses its contents, and returns a `NucleusFlowConfig` struct.
///
/// # Returns
///
/// - `Result<NucleusFlowConfig>`: A result containing the parsed configuration if successful,
///   or an error if the file couldn't be read or parsed.
///
/// # Errors
///
/// This function will return an error if:
/// - The `shokunin.toml` file cannot be read
/// - The file contents cannot be parsed as valid TOML
/// - The parsed TOML doesn't match the expected `Config` structure
pub fn load_config() -> Result<NucleusFlowConfig> {
    let config_path = Path::new("shokunin.toml");
    let config_str = fs::read_to_string(config_path).map_err(|e| {
        NucleusFlowError::Config(format!(
            "Failed to read config file: {}",
            e
        ))
    })?;

    let config: Config = toml::from_str(&config_str).map_err(|e| {
        NucleusFlowError::Config(format!(
            "Failed to parse config file: {}",
            e
        ))
    })?;

    Ok(NucleusFlowConfig {
        content_dir: config.content_dir.into(),
        output_dir: config.output_dir.into(),
        template_dir: config.template_dir.into(),
    })
}

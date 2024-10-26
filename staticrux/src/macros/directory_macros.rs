// Copyright © 2024 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Directory operation macros for the static site generator
//!
//! This module provides macros for common directory operations, including:
//! - Checking and creating directories
//! - Cleaning up directories
//! - Creating multiple directories at once
//!
//! The macros in this module are designed to provide a convenient and safe way
//! to perform common directory operations while maintaining proper error handling
//! and logging.

/// Checks if a directory exists and creates it if necessary.
///
/// # Arguments
///
/// * `_dir` - The path to check/create (as a `std::path::Path`)
/// * `_name` - A string literal representing the directory name for error messages
///
/// # Example
///
/// ```rust
/// use staticrux::macro_check_directory;
/// use std::path::Path;
/// use std::fs;
///
/// let path = Path::new("logs");
/// macro_check_directory!(path, "logs");
///
/// // Ensure the directory is removed after the test
/// if path.exists() {
///     fs::remove_dir_all(path).expect("Failed to remove logs directory");
/// }
/// ```
///
/// # Panics
///
/// This macro will panic if:
/// - The path exists but is not a directory
/// - The directory cannot be created
#[macro_export]
macro_rules! macro_check_directory {
    ($_dir:expr, $_name:expr) => {{
        use std::path::Path;
        let directory: &Path = $_dir;
        let name = $_name;

        if directory.exists() {
            if !directory.is_dir() {
                log::error!("❌ '{}' is not a directory.", name);
                panic!("❌ '{}' is not a directory.", name);
            }
        } else {
            match std::fs::create_dir_all(directory) {
                Ok(_) => {
                    log::info!("✓ Created directory: {}", name);
                }
                Err(e) => {
                    log::error!(
                        "❌ Cannot create '{}' directory: {}",
                        name,
                        e
                    );
                    panic!(
                        "❌ Cannot create '{}' directory: {}",
                        name, e
                    );
                }
            }
        }
    }};
}

/// Cleans up (removes) multiple directories.
///
/// # Arguments
///
/// * `$path` - The path to the directory to clean up
///
/// # Returns
///
/// Returns a `Result<(), anyhow::Error>` indicating success or failure.
///
/// # Example
///
/// ```rust
/// use staticrux::macro_cleanup_directories;
/// use std::path::Path;
///
/// let path = Path::new("temp_dir");
/// if let Err(e) = macro_cleanup_directories!(path) {
///     eprintln!("Failed to clean up directory: {}", e);
/// }
/// ```
#[macro_export]
macro_rules! macro_cleanup_directories {
    ($path:expr) => {{
        use anyhow::Context;
        std::fs::remove_dir_all($path).with_context(|| {
            format!("Failed to clean up directory: {:?}", $path)
        })
    }};
}

/// Creates multiple directories at once.
///
/// # Arguments
///
/// * `$($path:expr),+` - One or more directory paths to create
///
/// # Returns
///
/// Returns a `Result<(), anyhow::Error>` indicating success or failure.
///
/// # Example
///
/// ```rust
/// use staticrux::macro_create_directories;
/// use std::path::Path;
/// use std::fs;
///
/// let path1 = Path::new("dir1");
/// let path2 = Path::new("dir2");
///
/// // Attempt to create directories
/// if let Err(e) = macro_create_directories!(path1, path2) {
///     eprintln!("Failed to create directories: {}", e);
/// }
///
/// // Ensure the directories are removed after the test
/// if path1.exists() {
///     fs::remove_dir_all(path1).expect("Failed to remove dir1");
/// }
/// if path2.exists() {
///     fs::remove_dir_all(path2).expect("Failed to remove dir2");
/// }
/// ```
#[macro_export]
macro_rules! macro_create_directories {
    ($($path:expr),+) => {
        {
            use anyhow::{Result, Context};
            (|| -> Result<()> {
                $(
                    std::fs::create_dir_all($path)
                        .with_context(|| format!("Failed to create directory: {:?}", $path))?;
                    log::info!("✓ Created directory: {:?}", $path);
                )+
                Ok(())
            })()
        }
    };
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    #[test]
    fn test_macro_check_directory() {
        let temp_dir = TempDir::new().unwrap();
        let test_path = temp_dir.path().join("test_dir");

        macro_check_directory!(&test_path, "test_dir");
        assert!(test_path.exists());
        assert!(test_path.is_dir());
    }

    #[test]
    fn test_macro_cleanup_directories() {
        let temp_dir = TempDir::new().unwrap();
        let test_path = temp_dir.path().join("test_dir");
        std::fs::create_dir(&test_path).unwrap();

        assert!(macro_cleanup_directories!(&test_path).is_ok());
        assert!(!test_path.exists());
    }

    #[test]
    fn test_macro_create_directories() {
        let temp_dir = TempDir::new().unwrap();
        let test_path1 = temp_dir.path().join("dir1");
        let test_path2 = temp_dir.path().join("dir2");

        assert!(
            macro_create_directories!(&test_path1, &test_path2).is_ok()
        );
        assert!(test_path1.exists());
        assert!(test_path2.exists());
    }
}

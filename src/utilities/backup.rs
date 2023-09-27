// Copyright © 2023 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

extern crate regex;
use std::{
    fs::{self},
    io::{self},
    path::{Path, PathBuf},
};

/// Creates a backup of a file.
///
/// This function takes a reference to a `Path` object for a file and creates a
/// backup of the file with the extension ".src.html".
///
/// # Arguments
///
/// * `file_path` - A reference to a `Path` object for the file.
///
/// # Returns
///
/// * `Result<PathBuf, std::io::Error>` - A result containing a `PathBuf`
///    object for the backup file.
///     - `Ok(PathBuf)` if the backup file was created successfully.
///     - `Err(std::io::Error)` if the backup file could not be created.
///
pub fn backup_file(file_path: &Path) -> io::Result<PathBuf> {
    let backup_path = file_path.with_extension("src.html");
    fs::copy(file_path, &backup_path)?;
    Ok(backup_path)
}
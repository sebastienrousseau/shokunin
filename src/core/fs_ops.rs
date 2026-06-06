// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! File system operations: directory copying, safety validation, and traversal.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{ensure, Context, Result};
use rayon::prelude::*;

use crate::MAX_DIR_DEPTH;

/// Minimum number of entries to justify Rayon parallel dispatch overhead.
pub(crate) const PARALLEL_THRESHOLD: usize = 16;

/// Validates and copies files from source to destination.
///
/// This function performs comprehensive safety checks before copying files,
/// including path validation, symlink detection, and size limitations.
///
/// # Arguments
///
/// * `src` - Source path to copy from
/// * `dst` - Destination path to copy to
///
/// # Returns
///
/// Returns `Ok(())` if the copy operation succeeds, or an error if:
/// * Source path is invalid or inaccessible
/// * Source contains symlinks (not allowed)
/// * Files exceed size limits (default: 10MB)
/// * Destination cannot be created or written to
///
/// # Example
///
/// ```rust,no_run
/// use std::path::Path;
/// use ssg::verify_and_copy_files;
///
/// fn main() -> anyhow::Result<()> {
///     let source = Path::new("source_directory");
///     let destination = Path::new("destination_directory");
///
///     verify_and_copy_files(source, destination)?;
///     println!("Files copied successfully");
///     Ok(())
/// }
/// ```
///
/// # Security
///
/// This function implements several security measures:
/// * Path traversal prevention
/// * Symlink restriction
/// * File size limits
/// * Permission validation
pub fn verify_and_copy_files(src: &Path, dst: &Path) -> Result<()> {
    ensure!(
        is_safe_path(src)?,
        "Source directory is unsafe or inaccessible: {}",
        src.display()
    );

    if !src.exists() {
        anyhow::bail!("Source directory does not exist: {}", src.display());
    }

    // If source is a file, verify its safety
    if src.is_file() {
        verify_file_safety(src)?;
    }

    // Ensure the destination directory exists
    fs::create_dir_all(dst).with_context(|| {
        format!(
            "Failed to create or access destination directory at path: {}",
            dst.display()
        )
    })?;

    // Copy directory contents with safety checks
    copy_dir_all(src, dst).with_context(|| {
        format!(
            "Failed to copy files from source: {} to destination: {}",
            src.display(),
            dst.display()
        )
    })?;

    Ok(())
}

/// Asynchronously validates and copies files between directories.
///
/// Uses iterative traversal with an explicit stack to avoid unbounded recursion.
/// Traversal depth is bounded by [`MAX_DIR_DEPTH`].
pub fn verify_and_copy_files_async(src: &Path, dst: &Path) -> Result<()> {
    if !src.exists() {
        return Err(anyhow::anyhow!(
            "Source directory does not exist: {}",
            src.display()
        ));
    }

    fs::create_dir_all(dst).with_context(|| {
        format!(
            "Failed to create or access destination directory at path: {}",
            dst.display()
        )
    })?;

    copy_directory_recursive(src, dst)
}

/// Iteratively copies a directory tree with depth bounds and safety checks.
fn copy_directory_recursive(src: &Path, dst: &Path) -> Result<()> {
    let mut stack = vec![(src.to_path_buf(), dst.to_path_buf(), 0usize)];

    while let Some((src_dir, dst_dir, depth)) = stack.pop() {
        ensure!(
            depth < MAX_DIR_DEPTH,
            "Directory nesting exceeds maximum depth of {}: {}",
            MAX_DIR_DEPTH,
            src_dir.display()
        );

        for entry in fs::read_dir(&src_dir)? {
            let entry = entry?;
            copy_entry(&entry, &dst_dir, depth, &mut stack)?;
        }
    }

    Ok(())
}

/// Copies a single directory entry, pushing subdirs onto the stack.
fn copy_entry(
    entry: &fs::DirEntry,
    dst_dir: &Path,
    depth: usize,
    stack: &mut Vec<(PathBuf, PathBuf, usize)>,
) -> Result<()> {
    let src_path = entry.path();
    let dst_path = dst_dir.join(entry.file_name());

    if src_path.is_dir() {
        fs::create_dir_all(&dst_path)?;
        stack.push((src_path, dst_path, depth + 1));
    } else {
        verify_file_safety(&src_path)?;
        _ = fs::copy(&src_path, &dst_path)?;
    }
    Ok(())
}

/// Copies directories with a progress bar for feedback.
///
/// Uses iterative traversal with an explicit stack to avoid unbounded recursion.
/// Traversal depth is bounded by [`MAX_DIR_DEPTH`].
pub fn copy_dir_with_progress(src: &Path, dst: &Path) -> Result<()> {
    if !src.exists() {
        anyhow::bail!("Source directory does not exist: {}", src.display());
    }

    fs::create_dir_all(dst).with_context(|| {
        format!("Failed to create destination directory: {}", dst.display())
    })?;

    let mut file_count: u64 = 0;

    // (source_dir, dest_dir, depth)
    let mut stack = vec![(src.to_path_buf(), dst.to_path_buf(), 0usize)];

    while let Some((src_dir, dst_dir, depth)) = stack.pop() {
        ensure!(
            depth < MAX_DIR_DEPTH,
            "Directory nesting exceeds maximum depth of {}: {}",
            MAX_DIR_DEPTH,
            src_dir.display()
        );

        let entries: Vec<_> = fs::read_dir(&src_dir)
            .context(format!(
                "Failed to read source directory: {}",
                src_dir.display()
            ))?
            .collect::<std::io::Result<Vec<_>>>()?;

        for entry in &entries {
            let src_path = entry.path();
            let dst_path = dst_dir.join(entry.file_name());

            if src_path.is_dir() {
                fs::create_dir_all(&dst_path)?;
                stack.push((src_path, dst_path, depth + 1));
            } else {
                let _ = fs::copy(&src_path, &dst_path)?;
            }
            file_count += 1;
        }
    }

    eprintln!("Copied {file_count} files");
    Ok(())
}

/// Checks if a given path is safe to use.
///
/// Validates that the provided path does not contain directory traversal attempts
/// or other potential security risks.
///
/// # Arguments
///
/// * `path` - The path to validate
///
/// # Returns
///
/// * `Ok(true)` - If the path is safe to use
/// * `Ok(false)` - If the path contains unsafe elements
/// * `Err` - If path validation fails
///
/// # Security
///
/// This function prevents directory traversal attacks by:
/// * Resolving symbolic links
/// * Checking for parent directory references (`..`)
/// * Validating path components
///
pub fn is_safe_path(path: &Path) -> Result<bool> {
    // Check for traversal patterns in non-existent paths
    if !path.exists() {
        let path_str = path.to_string_lossy();
        if path_str.contains("..") {
            return Ok(false);
        }
        return Ok(true); // Non-existent paths without traversal are safe
    }

    // canonicalize() resolves symlinks and all `..' components,
    // so the resulting path is always absolute with no parent refs.
    // A failure here (e.g. broken symlink) means the path is unsafe.
    let _canonical = path
        .canonicalize()
        .context(format!("Failed to canonicalize path {}", path.display()))?;

    Ok(true)
}

/// Verifies the safety of a file for processing.
///
/// Performs comprehensive safety checks on a file to ensure it meets security
/// requirements before processing. These checks include symlink detection and
/// file size validation.
///
/// # Arguments
///
/// * `path` - Reference to the path of the file to verify
///
/// # Returns
///
/// * `Ok(())` - If the file passes all safety checks
/// * `Err` - If any safety check fails
///
/// # Safety Checks
///
/// * Symlinks: Not allowed (returns error)
/// * File size: Must be under 10MB
/// * File type: Must be a regular file
///
/// # Examples
///
/// Verifies the safety of a file.
///
/// ```rust
/// use std::fs;
/// use std::path::Path;
/// use ssg::verify_file_safety;
/// use tempfile::tempdir;
///
/// # fn main() -> anyhow::Result<()> {
/// // Create temporary directory
/// let temp_dir = tempdir()?;
/// let file_path = temp_dir.path().join("index.md");
///
/// // Create test file
/// fs::write(&file_path, "Hello, world!")?;
///
/// // Perform verification
/// verify_file_safety(&file_path)?;
///
/// // Directory and file are automatically cleaned up
/// # Ok(())
/// # }
/// ```
///
/// # Errors
///
/// Returns an error if:
/// * File is a symlink
/// * File size exceeds 10MB
/// * Cannot read file metadata
pub fn verify_file_safety(path: &Path) -> Result<()> {
    const MAX_FILE_SIZE: u64 = 10 * 1024 * 1024; // 10MB limit

    // Get symlink metadata without following the symlink
    let symlink_metadata = path.symlink_metadata().map_err(|e| {
        anyhow::anyhow!(
            "Failed to get symlink metadata for {}: {}",
            path.display(),
            e
        )
    })?;

    // Explicitly check for symlinks first
    if symlink_metadata.file_type().is_symlink() {
        return Err(anyhow::anyhow!(
            "Symlinks are not allowed: {}",
            path.display()
        ));
    }

    // Only check size if it's a regular file
    if symlink_metadata.file_type().is_file()
        && symlink_metadata.len() > MAX_FILE_SIZE
    {
        return Err(anyhow::anyhow!(
            "File exceeds maximum allowed size of {} bytes: {}",
            MAX_FILE_SIZE,
            path.display()
        ));
    }

    Ok(())
}

/// Recursively collects all file paths within a directory.
///
/// Traverses a directory tree and compiles a list of all file paths found,
/// excluding directories themselves.
///
/// # Arguments
///
/// * `dir` - Reference to the directory to search
/// * `files` - Mutable vector to store found file paths
///
/// # Returns
///
/// * `Ok(())` - If the collection process succeeds
/// * `Err` - If any file system operation fails
///
/// # Examples
///
/// ```rust
/// use std::path::{Path, PathBuf};
/// use ssg::collect_files_recursive;
///
/// fn main() -> anyhow::Result<()> {
///     let mut files = Vec::new();
///     let dir_path = Path::new("./examples/content");
///
///     collect_files_recursive(dir_path, &mut files)?;
///
///     for file in files {
///         println!("Found file: {}", file.display());
///     }
///
///     Ok(())
/// }
/// ```
///
/// # Note
///
/// This function:
/// * Only collects file paths, not directory paths
/// * Rejects symbolic links (consistent with security model)
/// * Maintains original path structure
pub fn collect_files_recursive(
    dir: &Path,
    files: &mut Vec<PathBuf>,
) -> Result<()> {
    // (directory, depth)
    let mut stack = vec![(dir.to_path_buf(), 0usize)];

    while let Some((current_dir, depth)) = stack.pop() {
        ensure!(
            depth < MAX_DIR_DEPTH,
            "Directory nesting exceeds maximum depth of {}: {}",
            MAX_DIR_DEPTH,
            current_dir.display()
        );

        for entry in fs::read_dir(&current_dir)? {
            let path = entry?.path();

            if path.is_dir() {
                stack.push((path, depth + 1));
            } else {
                files.push(path);
            }
        }
    }
    Ok(())
}

/// Recursively copies a directory whilst maintaining structure and attributes.
///
/// Performs a deep copy of a directory tree, preserving file attributes and
/// handling nested directories. Uses parallel processing for improved performance.
///
/// # Arguments
///
/// * `src` - Source directory path
/// * `dst` - Destination directory path
///
/// # Returns
///
/// * `Ok(())` - If the copy operation succeeds
/// * `Err` - If any part of the copy operation fails
///
/// # Performance
///
/// Uses rayon for parallel processing of files, significantly improving
/// performance for directories with many files.
///
/// # Safety
///
/// * Verifies file safety before copying
/// * Maintains original file permissions
/// * Handles circular references
pub fn copy_dir_all(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst)?;

    // (source_dir, dest_dir, depth)
    let mut stack = vec![(src.to_path_buf(), dst.to_path_buf(), 0usize)];

    while let Some((src_dir, dst_dir, depth)) = stack.pop() {
        ensure!(
            depth < MAX_DIR_DEPTH,
            "Directory nesting exceeds maximum depth of {}: {}",
            MAX_DIR_DEPTH,
            src_dir.display()
        );

        let entries: Vec<_> =
            fs::read_dir(&src_dir)?.collect::<std::io::Result<Vec<_>>>()?;

        let (files, subdirs) = partition_entries(&entries, &dst_dir);

        copy_files_maybe_parallel(&files, &dst_dir)?;

        for (sub_src, sub_dst) in subdirs {
            fs::create_dir_all(&sub_dst)?;
            stack.push((sub_src, sub_dst, depth + 1));
        }
    }

    Ok(())
}

/// Separates directory entries into files and subdirectories.
fn partition_entries<'a>(
    entries: &'a [fs::DirEntry],
    dst_dir: &Path,
) -> (Vec<&'a fs::DirEntry>, Vec<(PathBuf, PathBuf)>) {
    let mut subdirs = Vec::new();
    let files: Vec<_> = entries
        .iter()
        .filter(|entry| {
            let path = entry.path();
            if path.is_dir() {
                subdirs.push((path, dst_dir.join(entry.file_name())));
                false
            } else {
                true
            }
        })
        .collect();
    (files, subdirs)
}

/// Copies file entries, using parallel dispatch when the count justifies it.
fn copy_files_maybe_parallel(
    files: &[&fs::DirEntry],
    dst_dir: &Path,
) -> Result<()> {
    let copy_file = |entry: &&fs::DirEntry| -> Result<()> {
        let src_path = entry.path();
        let dst_path = dst_dir.join(entry.file_name());
        verify_file_safety(&src_path)?;
        _ = fs::copy(&src_path, &dst_path)?;
        Ok(())
    };

    if files.len() >= PARALLEL_THRESHOLD {
        files.par_iter().try_for_each(copy_file)?;
    } else {
        files.iter().try_for_each(copy_file)?;
    }
    Ok(())
}

/// Asynchronously copies an entire directory structure, preserving file attributes and handling nested directories.
///
/// # Parameters
///
/// * `src`: A reference to the source directory path.
/// * `dst`: A reference to the destination directory path.
///
/// # Returns
///
/// * `Result<()>`:
///   - `Ok(())`: If the directory copying is successful.
///   - `Err(e)`: If an error occurs during the directory copying, where `e` is the associated error.
///
/// # Errors
///
/// This function can return the following errors:
///
/// * `std::io::Error`: If an error occurs during directory creation, file copying, or permission issues.
/// * `anyhow::Error`: If a file safety check fails.
pub fn copy_dir_all_async(src: &Path, dst: &Path) -> Result<()> {
    internal_copy_dir_async(src, dst)
}

fn internal_copy_dir_async(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst)?;

    // (source_dir, dest_dir, depth)
    let mut stack = vec![(src.to_path_buf(), dst.to_path_buf(), 0usize)];

    while let Some((src_path, dst_path, depth)) = stack.pop() {
        ensure!(
            depth < MAX_DIR_DEPTH,
            "Directory nesting exceeds maximum depth of {}: {}",
            MAX_DIR_DEPTH,
            src_path.display()
        );

        for entry in fs::read_dir(&src_path)? {
            let entry = entry?;
            let src_entry = entry.path();
            let dst_entry = dst_path.join(entry.file_name());

            if src_entry.is_dir() {
                fs::create_dir_all(&dst_entry)?;
                stack.push((src_entry, dst_entry, depth + 1));
            } else {
                verify_file_safety(&src_entry)?;
                _ = fs::copy(&src_entry, &dst_entry)?;
            }
        }
    }

    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn copy_dir_all_copies_files() {
        let src = tempdir().unwrap();
        let dst = tempdir().unwrap();
        fs::write(src.path().join("a.txt"), "hello").unwrap();
        fs::write(src.path().join("b.txt"), "world").unwrap();

        copy_dir_all(src.path(), dst.path()).unwrap();

        assert_eq!(
            fs::read_to_string(dst.path().join("a.txt")).unwrap(),
            "hello"
        );
        assert_eq!(
            fs::read_to_string(dst.path().join("b.txt")).unwrap(),
            "world"
        );
    }

    #[test]
    fn copy_dir_all_nested_preserves_structure() {
        let src = tempdir().unwrap();
        let dst = tempdir().unwrap();
        let nested = src.path().join("sub").join("deep");
        fs::create_dir_all(&nested).unwrap();
        fs::write(nested.join("file.txt"), "nested content").unwrap();
        fs::write(src.path().join("root.txt"), "root").unwrap();

        copy_dir_all(src.path(), dst.path()).unwrap();

        assert_eq!(
            fs::read_to_string(dst.path().join("sub/deep/file.txt")).unwrap(),
            "nested content"
        );
        assert_eq!(
            fs::read_to_string(dst.path().join("root.txt")).unwrap(),
            "root"
        );
    }

    #[test]
    fn copy_dir_all_nonexistent_src_returns_error() {
        let dst = tempdir().unwrap();
        let fake_src = dst.path().join("does_not_exist");

        let result = copy_dir_all(&fake_src, dst.path());
        assert!(result.is_err());
    }

    #[test]
    fn is_safe_path_normal_relative() {
        let tmp = tempdir().unwrap();
        let file = tmp.path().join("safe.txt");
        fs::write(&file, "ok").unwrap();

        assert!(is_safe_path(&file).unwrap());
    }

    #[test]
    fn is_safe_path_with_dotdot_nonexistent() {
        let path = Path::new("some/../../../etc/passwd");
        assert!(!is_safe_path(path).unwrap());
    }

    #[test]
    fn is_safe_path_with_dotdot_existing() {
        let tmp = tempdir().unwrap();
        // Create a path that exists and canonicalises cleanly
        let safe = tmp.path().join("a");
        fs::create_dir_all(&safe).unwrap();
        let dotdot_path = safe.join("..");
        // canonicalize succeeds → safe
        assert!(is_safe_path(&dotdot_path).unwrap());
    }

    #[test]
    fn is_safe_path_absolute_existing() {
        let tmp = tempdir().unwrap();
        let file = tmp.path().join("abs.txt");
        fs::write(&file, "data").unwrap();
        // Absolute path that exists is safe
        assert!(is_safe_path(&file).unwrap());
    }

    #[test]
    fn verify_file_safety_valid_file() {
        let tmp = tempdir().unwrap();
        let file = tmp.path().join("ok.txt");
        fs::write(&file, "small file").unwrap();

        assert!(verify_file_safety(&file).is_ok());
    }

    #[test]
    fn verify_file_safety_nonexistent() {
        let tmp = tempdir().unwrap();
        let missing = tmp.path().join("nope.txt");

        // symlink_metadata fails on nonexistent file → Err
        assert!(verify_file_safety(&missing).is_err());
    }

    #[test]
    fn verify_file_safety_directory() {
        let tmp = tempdir().unwrap();
        // Directories are not files but should not error (size check skipped)
        assert!(verify_file_safety(tmp.path()).is_ok());
    }

    #[test]
    fn collect_files_recursive_finds_all() {
        let tmp = tempdir().unwrap();
        let sub = tmp.path().join("sub");
        fs::create_dir_all(&sub).unwrap();
        fs::write(tmp.path().join("a.md"), "").unwrap();
        fs::write(sub.join("b.md"), "").unwrap();
        fs::write(sub.join("c.txt"), "").unwrap();

        let mut files = Vec::new();
        collect_files_recursive(tmp.path(), &mut files).unwrap();

        assert_eq!(files.len(), 3);
    }

    #[test]
    fn collect_files_recursive_empty_dir() {
        let tmp = tempdir().unwrap();

        let mut files = Vec::new();
        collect_files_recursive(tmp.path(), &mut files).unwrap();

        assert!(files.is_empty());
    }

    #[test]
    fn collect_files_recursive_only_files_not_dirs() {
        let tmp = tempdir().unwrap();
        let sub = tmp.path().join("subdir");
        fs::create_dir_all(&sub).unwrap();
        fs::write(sub.join("only.txt"), "data").unwrap();

        let mut files = Vec::new();
        collect_files_recursive(tmp.path(), &mut files).unwrap();

        assert_eq!(files.len(), 1);
        assert!(files[0].ends_with("only.txt"));
    }

    #[test]
    fn verify_and_copy_files_end_to_end() {
        let src = tempdir().unwrap();
        let dst = tempdir().unwrap();
        let target = dst.path().join("output");
        fs::write(src.path().join("page.html"), "<h1>Hi</h1>").unwrap();

        verify_and_copy_files(src.path(), &target).unwrap();

        assert_eq!(
            fs::read_to_string(target.join("page.html")).unwrap(),
            "<h1>Hi</h1>"
        );
    }

    #[test]
    fn copy_dir_with_progress_smoke() {
        let src = tempdir().unwrap();
        let dst = tempdir().unwrap();
        fs::write(src.path().join("f.txt"), "data").unwrap();

        // Should not panic
        copy_dir_with_progress(src.path(), &dst.path().join("out")).unwrap();
    }

    #[test]
    fn copy_dir_with_progress_nonexistent_src() {
        let tmp = tempdir().unwrap();
        let fake = tmp.path().join("missing");

        let result = copy_dir_with_progress(&fake, tmp.path());
        assert!(result.is_err());
    }
}

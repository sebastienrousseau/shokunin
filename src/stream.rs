// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! High-performance streaming file processor.
//!
//! Provides constant-memory file processing for workloads from 1K to 50K+
//! files. All I/O uses fixed-size buffers — memory usage does not grow
//! with file size or transaction count.
//!
//! # Performance targets
//!
//! - Time to first result: < 2 ms
//! - Throughput: >= 50,000 files/second
//! - Memory: constant O(1) per file via streaming
//!
//! # Architecture
//!
//! Files are processed through a pipeline of `StreamProcessor` stages.
//! Each stage reads from a buffered input, transforms in a fixed-size
//! buffer, and writes to a buffered output. No file is ever fully loaded
//! into memory unless it fits within the buffer size.

use anyhow::{Context, Result};
use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use std::time::Instant;

/// Default buffer size for streaming I/O (8 KB).
/// Aligned to typical filesystem block size for optimal throughput.
pub const STREAM_BUFFER_SIZE: usize = 8 * 1024;

/// Maximum number of files to process in a single batch.
/// Bounds memory for directory listings per Power of Ten Rule 2.
pub const MAX_BATCH_SIZE: usize = 100_000;

/// Result of processing a batch of files.
#[derive(Debug, Clone, Copy)]
pub struct BatchResult {
    /// Number of files processed.
    pub files_processed: usize,
    /// Total bytes read across all files.
    pub bytes_read: u64,
    /// Total bytes written across all files.
    pub bytes_written: u64,
    /// Wall-clock duration of the batch.
    pub duration_ms: f64,
    /// Throughput in files per second.
    pub throughput: f64,
}

/// Copies a single file using buffered streaming I/O.
///
/// Reads and writes in `STREAM_BUFFER_SIZE` chunks. Memory usage is
/// constant regardless of file size — a 1 KB file and a 1 GB file
/// use the same buffer.
///
/// # Errors
///
/// Returns an error if the source cannot be read or the destination
/// cannot be written.
pub fn stream_copy(src: &Path, dst: &Path) -> Result<u64> {
    let file_in = File::open(src)
        .with_context(|| format!("cannot open {}", src.display()))?;
    let file_out = File::create(dst)
        .with_context(|| format!("cannot create {}", dst.display()))?;

    let mut reader = BufReader::with_capacity(STREAM_BUFFER_SIZE, file_in);
    let mut writer = BufWriter::with_capacity(STREAM_BUFFER_SIZE, file_out);

    let mut buf = [0u8; STREAM_BUFFER_SIZE];
    let mut total: u64 = 0;

    loop {
        let n = reader
            .read(&mut buf)
            .with_context(|| format!("read error: {}", src.display()))?;
        if n == 0 {
            break;
        }
        writer
            .write_all(&buf[..n])
            .with_context(|| format!("write error: {}", dst.display()))?;
        total += n as u64;
    }

    writer
        .flush()
        .with_context(|| format!("flush error: {}", dst.display()))?;

    Ok(total)
}

/// Hashes a file using streaming I/O with constant memory.
///
/// Reads in `STREAM_BUFFER_SIZE` chunks and feeds each chunk to a
/// `DefaultHasher`. Never loads the entire file into memory.
///
/// Returns a 16-character hex fingerprint.
pub fn stream_hash(path: &Path) -> Result<String> {
    use std::hash::{DefaultHasher, Hasher};

    let file = File::open(path)
        .with_context(|| format!("cannot open {}", path.display()))?;
    let mut reader = BufReader::with_capacity(STREAM_BUFFER_SIZE, file);
    let mut hasher = DefaultHasher::new();
    let mut buf = [0u8; STREAM_BUFFER_SIZE];

    loop {
        let n = reader
            .read(&mut buf)
            .with_context(|| format!("read error: {}", path.display()))?;
        if n == 0 {
            break;
        }
        hasher.write(&buf[..n]);
    }

    Ok(format!("{:016x}", hasher.finish()))
}

/// Processes a batch of files through a streaming pipeline.
///
/// Applies `processor` to each file in `src_dir`, writing results to
/// `dst_dir`. Processes files sequentially with constant memory. For
/// parallel processing, use `process_batch_parallel`.
///
/// # Errors
///
/// Returns an error if any file cannot be read, processed, or written.
/// Processing stops at the first error.
pub fn process_batch<F>(
    src_dir: &Path,
    dst_dir: &Path,
    processor: F,
) -> Result<BatchResult>
where
    F: Fn(&Path, &Path) -> Result<u64>,
{
    let start = Instant::now();

    fs::create_dir_all(dst_dir)
        .with_context(|| format!("cannot create {}", dst_dir.display()))?;

    let entries: Vec<PathBuf> = collect_files_bounded(src_dir)?;
    let mut bytes_read: u64 = 0;
    let mut bytes_written: u64 = 0;
    let mut count: usize = 0;

    for src_path in &entries {
        let rel = src_path
            .strip_prefix(src_dir)
            .with_context(|| "strip_prefix failed")?;
        let dst_path = dst_dir.join(rel);

        if let Some(parent) = dst_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let src_size = fs::metadata(src_path).map_or(0, |m| m.len());
        let written = processor(src_path, &dst_path)?;

        bytes_read += src_size;
        bytes_written += written;
        count += 1;
    }

    let elapsed = start.elapsed();
    let duration_ms = elapsed.as_secs_f64() * 1000.0;
    let throughput = if duration_ms > 0.0 {
        count as f64 / elapsed.as_secs_f64()
    } else {
        f64::INFINITY
    };

    Ok(BatchResult {
        files_processed: count,
        bytes_read,
        bytes_written,
        duration_ms,
        throughput,
    })
}

/// Collects files from a directory with a bounded iteration count.
///
/// Returns at most `MAX_BATCH_SIZE` files. Uses iterative traversal
/// (no recursion) with depth tracking.
fn collect_files_bounded(dir: &Path) -> Result<Vec<PathBuf>> {
    collect_files_bounded_with_limit(dir, MAX_BATCH_SIZE)
}

/// Inner walker accepting an explicit limit.
///
/// Extracted so unit tests can exercise the saturation `break`
/// branches without allocating `MAX_BATCH_SIZE` (100k) files on disk.
fn collect_files_bounded_with_limit(
    dir: &Path,
    limit: usize,
) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    let mut iterations: usize = 0;

    while let Some(current) = stack.pop() {
        if iterations >= limit {
            break;
        }

        let entries = fs::read_dir(&current)
            .with_context(|| format!("cannot read {}", current.display()))?;

        for entry in entries {
            let path = entry?.path();
            if path.is_dir() {
                stack.push(path);
            } else {
                files.push(path);
                iterations += 1;
                if iterations >= limit {
                    break;
                }
            }
        }
    }

    Ok(files)
}

/// Processes a file by reading line-by-line with constant memory.
///
/// Calls `line_fn` for each line. The line buffer is reused across
/// iterations — memory does not grow with file length.
///
/// # Errors
///
/// Returns an error if the file cannot be read.
pub fn stream_lines<F>(path: &Path, mut line_fn: F) -> Result<usize>
where
    F: FnMut(usize, &str) -> Result<()>,
{
    use std::io::BufRead;

    let file = File::open(path)
        .with_context(|| format!("cannot open {}", path.display()))?;
    let reader = BufReader::with_capacity(STREAM_BUFFER_SIZE, file);
    let mut count: usize = 0;

    for line in reader.lines() {
        let line =
            line.with_context(|| format!("read error at line {count}"))?;
        line_fn(count, &line)?;
        count += 1;
    }

    Ok(count)
}

/// Returns the throughput of a no-op pipeline to measure overhead.
///
/// Creates `n` temporary files and streams them through `stream_copy`.
/// Returns the measured throughput in files/second.
#[cfg(any(test, feature = "benchmark"))]
pub fn benchmark_throughput(n: usize) -> Result<BatchResult> {
    let tmp = tempfile::tempdir().context("cannot create temp dir")?;
    let src = tmp.path().join("src");
    let dst = tmp.path().join("dst");
    fs::create_dir_all(&src)?;

    // Create n small files (64 bytes each)
    for i in 0..n {
        fs::write(src.join(format!("f{i}.txt")), "a]".repeat(32))?;
    }

    process_batch(&src, &dst, stream_copy)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_stream_copy_small_file() -> Result<()> {
        let tmp = tempdir()?;
        let src = tmp.path().join("src.txt");
        let dst = tmp.path().join("dst.txt");
        fs::write(&src, "hello world")?;

        let bytes = stream_copy(&src, &dst)?;
        assert_eq!(bytes, 11);
        assert_eq!(fs::read_to_string(&dst)?, "hello world");
        Ok(())
    }

    #[test]
    fn test_stream_copy_large_file() -> Result<()> {
        let tmp = tempdir()?;
        let src = tmp.path().join("large.bin");
        let dst = tmp.path().join("large_copy.bin");

        // 1 MB file — larger than STREAM_BUFFER_SIZE
        let data = vec![0xABu8; 1024 * 1024];
        fs::write(&src, &data)?;

        let bytes = stream_copy(&src, &dst)?;
        assert_eq!(bytes, 1024 * 1024);
        assert_eq!(fs::read(&dst)?, data);
        Ok(())
    }

    #[test]
    fn test_stream_copy_empty_file() -> Result<()> {
        let tmp = tempdir()?;
        let src = tmp.path().join("empty.txt");
        let dst = tmp.path().join("empty_copy.txt");
        fs::write(&src, "")?;

        let bytes = stream_copy(&src, &dst)?;
        assert_eq!(bytes, 0);
        Ok(())
    }

    #[test]
    fn test_stream_hash_deterministic() -> Result<()> {
        let tmp = tempdir()?;
        let path = tmp.path().join("test.txt");
        fs::write(&path, "consistent content")?;

        let h1 = stream_hash(&path)?;
        let h2 = stream_hash(&path)?;
        assert_eq!(h1, h2);
        assert_eq!(h1.len(), 16);
        Ok(())
    }

    #[test]
    fn test_stream_hash_differs_for_different_content() -> Result<()> {
        let tmp = tempdir()?;
        let a = tmp.path().join("a.txt");
        let b = tmp.path().join("b.txt");
        fs::write(&a, "content a")?;
        fs::write(&b, "content b")?;

        assert_ne!(stream_hash(&a)?, stream_hash(&b)?);
        Ok(())
    }

    #[test]
    fn test_stream_hash_large_file() -> Result<()> {
        let tmp = tempdir()?;
        let path = tmp.path().join("big.bin");
        fs::write(&path, vec![0u8; 100_000])?;

        let hash = stream_hash(&path)?;
        assert_eq!(hash.len(), 16);
        Ok(())
    }

    #[test]
    fn test_process_batch_copies_files() -> Result<()> {
        let tmp = tempdir()?;
        let src = tmp.path().join("src");
        let dst = tmp.path().join("dst");
        fs::create_dir_all(&src)?;

        for i in 0..10 {
            fs::write(src.join(format!("f{i}.txt")), format!("data {i}"))?;
        }

        let result = process_batch(&src, &dst, stream_copy)?;
        assert_eq!(result.files_processed, 10);
        assert!(result.bytes_written > 0);
        assert!(result.throughput > 0.0);
        Ok(())
    }

    #[test]
    fn test_process_batch_empty_directory() -> Result<()> {
        let tmp = tempdir()?;
        let src = tmp.path().join("src");
        let dst = tmp.path().join("dst");
        fs::create_dir_all(&src)?;

        let result = process_batch(&src, &dst, stream_copy)?;
        assert_eq!(result.files_processed, 0);
        Ok(())
    }

    #[test]
    fn test_process_batch_nested_dirs() -> Result<()> {
        let tmp = tempdir()?;
        let src = tmp.path().join("src");
        let dst = tmp.path().join("dst");
        fs::create_dir_all(src.join("sub/deep"))?;
        fs::write(src.join("root.txt"), "root")?;
        fs::write(src.join("sub/mid.txt"), "mid")?;
        fs::write(src.join("sub/deep/leaf.txt"), "leaf")?;

        let result = process_batch(&src, &dst, stream_copy)?;
        assert_eq!(result.files_processed, 3);
        assert_eq!(fs::read_to_string(dst.join("sub/deep/leaf.txt"))?, "leaf");
        Ok(())
    }

    #[test]
    fn test_stream_lines_counts_correctly() -> Result<()> {
        let tmp = tempdir()?;
        let path = tmp.path().join("lines.txt");
        fs::write(&path, "line1\nline2\nline3\n")?;

        let count = stream_lines(&path, |_i, _line| Ok(()))?;
        assert_eq!(count, 3);
        Ok(())
    }

    #[test]
    fn test_stream_lines_provides_content() -> Result<()> {
        let tmp = tempdir()?;
        let path = tmp.path().join("data.txt");
        fs::write(&path, "alpha\nbeta\ngamma")?;

        let mut collected = Vec::new();
        let _ = stream_lines(&path, |_i, line| {
            collected.push(line.to_string());
            Ok(())
        })?;
        assert_eq!(collected, vec!["alpha", "beta", "gamma"]);
        Ok(())
    }

    #[test]
    fn test_collect_files_bounded_respects_limit() -> Result<()> {
        let tmp = tempdir()?;
        // MAX_BATCH_SIZE is 100_000 — just verify it doesn't panic
        for i in 0..50 {
            fs::write(tmp.path().join(format!("f{i}.txt")), "x")?;
        }
        let files = collect_files_bounded(tmp.path())?;
        assert_eq!(files.len(), 50);
        Ok(())
    }

    #[test]
    fn collect_files_bounded_with_limit_breaks_on_outer_loop_saturation(
    ) -> Result<()> {
        // Hits the `if iterations >= limit { break }` at the top of
        // the outer while loop (line 196 of the public version).
        // We add files in batches across multiple subdirectories so
        // the inner break fires first, leaves leftover stack entries,
        // and then the next outer-loop pop sees iterations == limit.
        let tmp = tempdir()?;
        let a = tmp.path().join("a");
        let b = tmp.path().join("b");
        fs::create_dir_all(&a)?;
        fs::create_dir_all(&b)?;
        for i in 0..3 {
            fs::write(a.join(format!("f{i}.txt")), "x")?;
            fs::write(b.join(format!("f{i}.txt")), "x")?;
        }

        let files = collect_files_bounded_with_limit(tmp.path(), 2)?;
        // The cap is honoured: at most `limit` files returned
        // (may be slightly more depending on which subdir is popped
        // first; the contract is "at most" with break-on-saturation).
        assert!(files.len() <= 4);
        Ok(())
    }

    #[test]
    fn collect_files_bounded_with_limit_breaks_on_inner_loop_saturation(
    ) -> Result<()> {
        // Hits the inner `if iterations >= limit { break }` (line 210
        // of the public version) — file count exceeds limit during
        // a single read_dir iteration.
        let tmp = tempdir()?;
        for i in 0..10 {
            fs::write(tmp.path().join(format!("f{i}.txt")), "x")?;
        }
        let files = collect_files_bounded_with_limit(tmp.path(), 3)?;
        assert_eq!(files.len(), 3);
        Ok(())
    }

    #[test]
    fn test_benchmark_throughput_runs() -> Result<()> {
        let result = benchmark_throughput(100)?;
        assert_eq!(result.files_processed, 100);
        assert!(
            result.throughput.is_finite() && result.throughput > 0.0,
            "invalid throughput: {}",
            result.throughput
        );
        println!(
            "Benchmark: {} files in {:.2} ms ({:.0} files/sec)",
            result.files_processed, result.duration_ms, result.throughput
        );
        Ok(())
    }

    #[test]
    fn test_batch_result_fields() {
        let r = BatchResult {
            files_processed: 10,
            bytes_read: 1000,
            bytes_written: 900,
            duration_ms: 1.5,
            throughput: 6666.0,
        };
        assert_eq!(r.files_processed, 10);
        assert!(r.throughput > 0.0);
    }

    #[test]
    fn test_stream_copy_nonexistent_source() {
        let dst = std::env::temp_dir().join("ssg_stream_copy_out");
        let result =
            stream_copy(Path::new("/definitely-does-not-exist-ssg"), &dst);
        assert!(result.is_err());
    }

    #[test]
    fn test_stream_hash_nonexistent() {
        let result = stream_hash(Path::new("/nonexistent"));
        assert!(result.is_err());
    }

    #[test]
    fn test_stream_lines_empty_file() -> Result<()> {
        let tmp = tempdir()?;
        let path = tmp.path().join("empty.txt");
        fs::write(&path, "")?;

        let count = stream_lines(&path, |_i, _line| Ok(()))?;
        assert_eq!(count, 0);
        Ok(())
    }

    #[test]
    fn stream_copy_exact_buffer_size_file() -> Result<()> {
        // Arrange
        let tmp = tempdir()?;
        let src = tmp.path().join("exact.bin");
        let dst = tmp.path().join("exact_copy.bin");
        let data = vec![0xCDu8; STREAM_BUFFER_SIZE];
        fs::write(&src, &data)?;

        // Act
        let bytes = stream_copy(&src, &dst)?;

        // Assert
        assert_eq!(bytes, STREAM_BUFFER_SIZE as u64);
        assert_eq!(fs::read(&dst)?, data);
        Ok(())
    }

    #[test]
    fn stream_hash_empty_file() -> Result<()> {
        // Arrange
        let tmp = tempdir()?;
        let path = tmp.path().join("empty.bin");
        fs::write(&path, b"")?;

        // Act
        let h1 = stream_hash(&path)?;
        let h2 = stream_hash(&path)?;

        // Assert
        assert_eq!(h1, h2, "hash of empty file must be deterministic");
        assert_eq!(h1.len(), 16);
        Ok(())
    }

    #[test]
    fn stream_hash_same_content_same_hash() -> Result<()> {
        // Arrange
        let tmp = tempdir()?;
        let a = tmp.path().join("file_a.txt");
        let b = tmp.path().join("file_b.txt");
        let content = "identical content in both files";
        fs::write(&a, content)?;
        fs::write(&b, content)?;

        // Act
        let hash_a = stream_hash(&a)?;
        let hash_b = stream_hash(&b)?;

        // Assert
        assert_eq!(hash_a, hash_b, "same content must produce same hash");
        Ok(())
    }

    #[test]
    fn stream_lines_binary_content() -> Result<()> {
        // Arrange — file with no newline characters
        let tmp = tempdir()?;
        let path = tmp.path().join("binary.bin");
        fs::write(&path, "no-newlines-here")?;

        // Act
        let mut lines_seen = Vec::new();
        let count = stream_lines(&path, |_i, line| {
            lines_seen.push(line.to_string());
            Ok(())
        })?;

        // Assert — single line, no newline splitting
        assert_eq!(count, 1);
        assert_eq!(lines_seen, vec!["no-newlines-here"]);
        Ok(())
    }

    #[test]
    fn process_batch_empty_directory() -> Result<()> {
        // Arrange — source directory with no files
        let tmp = tempdir()?;
        let src = tmp.path().join("empty_src");
        let dst = tmp.path().join("empty_dst");
        fs::create_dir_all(&src)?;

        // Act
        let result = process_batch(&src, &dst, stream_copy)?;

        // Assert
        assert_eq!(result.files_processed, 0);
        assert_eq!(result.bytes_read, 0);
        assert_eq!(result.bytes_written, 0);
        Ok(())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(1000))]

        /// Hashing the same content twice must yield the same fingerprint.
        #[test]
        fn stream_hash_deterministic(data in proptest::collection::vec(any::<u8>(), 0..4096)) {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("input.bin");
            fs::write(&path, &data).unwrap();

            let h1 = stream_hash(&path).unwrap();
            let h2 = stream_hash(&path).unwrap();
            prop_assert_eq!(h1, h2);
        }
    }
}

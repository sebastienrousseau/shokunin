// Copyright © 2025 Shokunin Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! This crate tests file generation functionality using `FileGenerator`.

#[cfg(test)]
mod tests {
    use staticdatagen::utilities::file::add;
    use std::{fs::File, io::Write, path::Path};

    #[test]
    fn test_add() {
        // Create a temporary directory for testing
        let temp_dir = tempfile::tempdir()
            .expect("Failed to create temporary directory");
        let temp_path = temp_dir.path();

        // Create test files
        create_test_file(temp_path, "file1.txt", "File 1 content");
        create_test_file(temp_path, "file2.txt", "File 2 content");
        create_test_file(
            temp_path,
            ".DS_Store",
            "DS_Store file content",
        ); // Should be skipped

        // Call add function
        let result = add(temp_path);

        // Clean up the temporary directory
        temp_dir
            .close()
            .expect("Failed to clean up temporary directory");

        // Assert the result
        assert!(result.is_ok());
        let files = result.unwrap();
        assert_eq!(files.len(), 2);

        let file1 =
            files.iter().find(|f| f.name == "file1.txt").unwrap();
        assert_eq!(file1.content, "File 1 content");
        assert_eq!(file1.rss, "File 1 content");
        assert_eq!(file1.txt, "File 1 content");
        assert_eq!(file1.cname, "File 1 content");
        assert_eq!(file1.sitemap, "File 1 content");

        let file2 =
            files.iter().find(|f| f.name == "file2.txt").unwrap();
        assert_eq!(file2.content, "File 2 content");
        assert_eq!(file2.rss, "File 2 content");
        assert_eq!(file2.txt, "File 2 content");
        assert_eq!(file2.cname, "File 2 content");
        assert_eq!(file2.sitemap, "File 2 content");
    }

    fn create_test_file<P: AsRef<Path>>(
        dir: P,
        name: &str,
        content: &str,
    ) {
        let file_path = dir.as_ref().join(name);
        let mut file = File::create(file_path)
            .expect("Failed to create test file");
        write!(file, "{}", content)
            .expect("Failed to write to test file");
    }
}

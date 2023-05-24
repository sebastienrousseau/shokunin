#[cfg(test)]
mod tests {
    use ssg::file::add;
    use std::{fs::File, io::Write};

    #[test]
    fn test_add() {
        // Create temporary directory with some files
        let temp_dir = tempfile::tempdir().unwrap();
        let file1_path = temp_dir.path().join("file1.txt");
        let mut file1 = File::create(file1_path).unwrap();
        writeln!(file1, "This is file1.").unwrap();

        // Test the function
        let files = add(temp_dir.path()).unwrap();

        // Verify the result
        assert_eq!(
            files,
            vec![ssg::file::File {
                name: "file1.txt".to_string(),
                content: "This is file1.\n".to_string(),
                rss: "This is file1.\n".to_string(),
                json: "\"This is file1.\\n\"".to_string(),
                txt: "This is file1.\n".to_string(),
                cname: "This is file1.\n".to_string(),
            }]
        );
    }
}

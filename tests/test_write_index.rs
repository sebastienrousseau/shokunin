#[cfg(test)]
mod tests {
    use std::{error::Error, fs};

    use ssg::{file::File, write_index};
    use tempfile::TempDir;

    #[test]
    fn test_write_index() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let out_dir = temp_dir.path().join("out");
        fs::create_dir(&out_dir)?;

        let files = vec![
            File::new(
                "file1.md",
                "Content1".to_string(),
                "Content1".to_string(),
            ),
            File::new(
                "file2.md",
                "Content2".to_string(),
                "Content2".to_string(),
            ),
        ];

        write_index(&files, &out_dir)?;

        let index_file = out_dir.join("index.html");
        let index_contents = fs::read_to_string(&index_file)?;

        let expected_output =
            "<ul class=\"nav\">\n<li><a href=\"file1.html\">file1</a></li>\n<li><a href=\"file2.html\">file2</a></li>\n</ul>";
        assert_eq!(index_contents.trim(), expected_output);

        Ok(())
    }
}

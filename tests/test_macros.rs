#[cfg(test)]
mod tests {
    use quick_xml::Writer;
    use ssg::{
        macro_check_directory, macro_cleanup_directories,
        macro_create_directories, macro_metadata_option,
        macro_write_element,
    };
    use std::path::Path;
    use std::{collections::HashMap, io::Cursor};

    #[test]
    fn test_macro_check_directory_existing_directory() {
        let temp_dir = tempfile::tempdir().unwrap();
        macro_check_directory!(temp_dir.path(), "temp_dir");
        assert!(temp_dir.path().is_dir());
    }

    #[test]
    fn test_macro_check_directory_nonexistent_directory() {
        let temp_dir = tempfile::tempdir().unwrap();
        let new_dir = temp_dir.path().join("new_dir");
        macro_check_directory!(&new_dir, "new_dir");
        assert!(new_dir.is_dir());
    }

    #[test]
    fn test_macro_cleanup_directories() {
        let dir1 = Path::new("dir1");
        let dir2 = Path::new("dir2");
        macro_cleanup_directories!(dir1, dir2);
    }

    #[test]
    fn test_macro_create_directories() {
        let temp_dir = tempfile::tempdir().unwrap();
        let dir1 = temp_dir.path().join("dir1");
        let dir2 = temp_dir.path().join("dir2");

        macro_create_directories!(&dir1, &dir2).unwrap();

        assert!(dir1.exists());
        assert!(dir2.exists());

        std::fs::remove_dir(&dir1).unwrap();
        std::fs::remove_dir(&dir2).unwrap();
    }

    #[test]
    fn test_macro_metadata_option_existing_key() {
        let mut metadata = HashMap::new();
        metadata.insert("key", "value");

        let value = macro_metadata_option!(metadata, "key");
        assert_eq!(value, "value");
    }

    #[test]
    fn test_macro_write_element_empty_value() {
        let mut writer = Writer::new(Cursor::new(Vec::new()));
        macro_write_element!(writer, "tag", "").unwrap();

        let result = writer.into_inner().into_inner();
        let expected = Vec::<u8>::new();

        assert_eq!(result, expected);
    }

    #[test]
    fn test_macro_write_element_nonempty_value() {
        let mut writer = Writer::new(Cursor::new(Vec::new()));
        macro_write_element!(writer, "tag", "value").unwrap();

        let result = writer.into_inner().into_inner();
        let expected = b"<tag>value</tag>".to_vec();

        assert_eq!(result, expected);
    }
}

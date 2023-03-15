#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;

    use clap::{Arg, Command};
    use ssg::args::process_arguments;

    #[test]
    fn test_process_arguments_valid_directories() {
        let content = "tests/test_content";
        let output = "tests/test_output";
        let _ = fs::create_dir_all(content);
        let _ = fs::create_dir_all(output);

        let matches = Command::new("test_shokunin")
            .arg(
                Arg::new("new")
                    .help("Create a new project.")
                    .long("new")
                    .short('n')
                    .value_name("NEW"),
            )
            .arg(
                Arg::new("content")
                    .help("Location of the content directory.")
                    .long("content")
                    .short('c')
                    .value_name("CONTENT"),
            )
            .arg(
                Arg::new("output")
                    .help("Location of the output directory.")
                    .long("output")
                    .short('o')
                    .value_name("OUTPUT"),
            )
            .get_matches();

        let result = process_arguments(&matches);
        assert_eq!(
            result,
            Err("❌ Error: Argument \"content\" is required but missing.".to_owned())
        );

        let content_path = Path::new(content);
        assert!(content_path.exists() && content_path.is_dir());

        let output_path = Path::new(output);
        assert!(output_path.exists() && output_path.is_dir());

        let _ = fs::remove_dir_all(content);
        assert!(!content_path.exists());

        let _ = fs::remove_dir_all(output);
        assert!(!output_path.exists());
    }
}

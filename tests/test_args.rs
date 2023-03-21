// This is a unit test module for the `process_arguments` function in
// the `args` module. It uses the `clap` crate to create a mock set of
// command-line arguments, and then calls the function being tested.
#[cfg(test)]
mod tests {
    // Import necessary modules and dependencies.
    use clap::{Arg, Command};
    use ssg::args::process_arguments;
    use std::fs;
    use std::path::Path;

    #[test]
    fn test_process_arguments_valid_directories() {
        // Set up the directories for the test.
        let content = "tests/test_content";
        let output = "tests/test_output";
        let _ = fs::create_dir_all(content);
        let _ = fs::create_dir_all(output);

        // Create the command-line arguments for the test.
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

        // Call the function being tested.
        let result = process_arguments(&matches);

        // Assert that the function returns an error message.
        assert_eq!(
            result,
            Err("❌ Error: Argument \"name\" is required but missing."
                .to_owned())
        );

        // Assert that the directories have been created.
        let content_path = Path::new(content);
        assert!(content_path.exists() && content_path.is_dir());
        let output_path = Path::new(output);
        assert!(output_path.exists() && output_path.is_dir());

        // Clean up the directories after the test.
        let _ = fs::remove_dir_all(content);
        assert!(!content_path.exists());
        let _ = fs::remove_dir_all(output);
        assert!(!output_path.exists());
    }
}

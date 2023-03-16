#[cfg(test)]
mod tests {
    use ssg::cli::build_cli;

    #[test]
    // Test that the arguments for the build CLI are correctly set
    fn test_build_cli_args() {
        // Define the expected argument values
        let arg_specs = [
            ("content", None),
            ("output", None),
            ("help", None),
            ("version", None),
        ];

        // Call the build_cli function to get the command-line arguments
        let args = build_cli().unwrap();

        // Iterate through the expected argument values
        for (arg_name, expected_value) in arg_specs.iter() {
            // Get the actual value for the argument
            let arg_value: Option<&String> = args.get_one(arg_name);

            // Compare the actual and expected values
            assert_eq!(arg_value, *expected_value);
        }
    }
}

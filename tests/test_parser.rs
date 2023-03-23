#[cfg(test)]
mod tests {

    use clap::{Arg, ArgMatches, Command};
    use ssg::parser::args;

    #[test]
    fn test_args_required_args_missing() {
        // Test missing required arguments
        let matches = Command::new("test")
            .arg(Arg::new("new"))
            .arg(Arg::new("content"))
            .arg(Arg::new("output"))
            .get_matches_from(vec!["test"]);
        let result = args(matches);
        assert_eq!(
            result,
            Err("❌ Error: Argument \"name\" is required but missing."
                .to_owned())
        );
    }

    #[test]
    fn test_args_required_args_present() {
        // Test required arguments present
        let matches = Command::new("test")
            .arg(Arg::new("new"))
            .arg(Arg::new("content"))
            .arg(Arg::new("output"))
            .get_matches_from(vec![
                "test_name",
                "test_content",
                "output",
            ]);
        let result = args(matches);
        assert_eq!(
            result,
            Err("❌ Error: Argument \"output\" is required but missing.".to_owned())
        );
    }

    fn create_matches(args: Vec<&str>) -> ArgMatches {
        Command::new("test")
            .arg(Arg::new("new"))
            .arg(Arg::new("content"))
            .arg(Arg::new("output"))
            .get_matches_from(args)
    }

    #[test]
    fn test_args_all_args_present() {
        // Test all arguments present
        let matches =
            create_matches(vec!["test_name", "test_content", "output"]);
        let result = args(matches);
        assert_eq!(result, Err("❌ Error: Argument \"output\" is required but missing.".to_owned()));
    }
}

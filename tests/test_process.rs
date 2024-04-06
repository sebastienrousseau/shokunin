#[cfg(test)]
mod tests {
    use clap::{Arg, Command};
    use ssg::cmd::process::args;

    #[test]
    fn test_args_required_args_missing() {
        // Test missing required arguments
        let matches = Command::new("test")
            .arg(Arg::new("new"))
            .arg(Arg::new("content"))
            .arg(Arg::new("output"))
            .get_matches_from(vec!["test"]);
        let result = args(&matches);
        assert_eq!(
            result,
            Err(
                "❌ Error: A required parameter was omitted. Add the required parameter. \"content\"."
                    .to_owned()
            )
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
        let result = args(&matches);
        assert_eq!(
            result,
            Err(
                "❌ Error: A required parameter was omitted. Add the required parameter. \"output\"."
                    .to_owned()
            )
        );
    }
}

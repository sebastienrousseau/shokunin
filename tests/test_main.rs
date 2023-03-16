#[cfg(test)]
mod tests {
    use assert_cmd::Command;
    use ssg::run;

    // Unit test for the `main()` function.
    #[test]
    fn test_main_failure() {
        let mut cmd = Command::cargo_bin("ssg").unwrap();
        let assert = cmd.assert();
        assert.failure();
    }
    // Unit test for the `run()` function.
    #[test]
    fn test_main_run_err() {
        let result = run();
        assert!(result.is_err());
    }
}

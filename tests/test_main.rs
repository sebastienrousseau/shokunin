#[cfg(test)]
mod tests {
    use assert_cmd::Command;

    // Unit test for the `main()` function.
    #[test]
    fn test_main_run_ok() {
        let mut cmd = Command::cargo_bin("ssg").unwrap();
        cmd.assert().success();
    }
}

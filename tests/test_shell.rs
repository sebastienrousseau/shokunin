/// Copyright © 2024 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

#[cfg(test)]
mod tests {
    use staticdatagen::macros::shell_macros::CommandExecutor;
    use staticdatagen::{
        macro_execute_and_log, macro_log_complete, macro_log_error,
        macro_log_start,
    };

    // Helper function to create a CommandExecutor with an optional interpreter
    fn create_executor(interpreter: Option<&str>) -> CommandExecutor {
        CommandExecutor::new(interpreter)
    }

    // Helper function to verify the command format of CommandExecutor
    fn assert_command_format(executor: &CommandExecutor, expected: &str) {
        assert_eq!(format!("{:?}", executor.command), expected);
    }

    // Tests for CommandExecutor
    #[test]
    fn test_command_executor_new_default_and_custom() {
        // Default interpreter should be "sh"
        let executor = create_executor(None);
        assert_command_format(&executor, "\"sh\" \"-c\"");

        // Custom interpreter set to "bash"
        let executor = create_executor(Some("bash"));
        assert_command_format(&executor, "\"bash\" \"-c\"");
    }

    #[test]
    fn test_command_executor_command_setting() {
        // Create executor and set a command
        let mut executor = create_executor(None);
        let _ = executor.command("ls -l");
        // Check that the command was set correctly
        assert_command_format(&executor, "\"sh\" \"-c\" \"ls -l\"");
    }

    #[test]
    fn test_command_executor_execute_success() {
        // Execute a simple echo command and check output
        let mut executor = create_executor(None);
        let _ = executor.command("echo 'Hello, World!'");
        let result = executor.execute();
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.status.success());
        assert_eq!(String::from_utf8_lossy(&output.stdout), "Hello, World!\n");
    }

    // Tests for logging macros
    #[test]
    fn test_macro_log_start() {
        // Test if macro compiles without errors
        macro_log_start!("Test Operation", "Test Start Message");
    }

    #[test]
    fn test_macro_log_complete() {
        // Test if macro compiles without errors
        macro_log_complete!("Test Operation", "Test Completion Message");
    }

    #[test]
    fn test_macro_log_error() {
        // Test if macro compiles without errors
        macro_log_error!("Test Operation", "Test Error Message");
    }

    #[test]
    fn test_macro_execute_and_log() {
        // Test if macro compiles without errors
        let _result = macro_execute_and_log!(
            "ls -l",
            "example_pkg",
            "list_directory",
            "Start Message",
            "Completion Message",
            "Error Message",
            Some("output"),
            Some("bash")
        );
    }
}

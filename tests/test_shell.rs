/// Copyright © 2024 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

#[cfg(test)]
mod tests {
    // Import necessary modules and types
    use staticrux::macros::shell_macros::CommandExecutor;
    use staticrux::{
        macro_execute_and_log, macro_log_complete, macro_log_error,
        macro_log_start,
    };

    // Tests for CommandExecutor
    #[test]
    fn test_command_executor_new() {
        // Create CommandExecutor with default interpreter
        let executor = CommandExecutor::new::<&str>(None);
        // Assert that default interpreter is sh
        assert_eq!(format!("{:?}", executor.command), "\"sh\" \"-c\"");

        // Create CommandExecutor with custom interpreter
        let executor = CommandExecutor::new(Some("bash"));
        // Assert that custom interpreter is bash
        assert_eq!(
            format!("{:?}", executor.command),
            "\"bash\" \"-c\""
        );
    }

    #[test]
    fn test_command_executor_command() {
        // Create CommandExecutor
        let mut executor = CommandExecutor::new::<&str>(None);
        // Set command
        let _ = executor.command("ls -l");
        // Assert that command is set correctly
        assert_eq!(
            format!("{:?}", executor.command),
            "\"sh\" \"-c\" \"ls -l\""
        );
    }

    #[test]
    fn test_command_executor_execute_success() {
        // Create CommandExecutor
        let mut executor = CommandExecutor::new::<&str>(None);
        // Set command
        let _ = executor.command("echo 'Hello, World!'");
        // Execute command
        let result = executor.execute();
        // Assert that command execution is successful
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.status.success());
        assert_eq!(
            String::from_utf8_lossy(&output.stdout),
            "Hello, World!\n"
        );
    }

    #[test]
    fn test_command_executor_new_default_interpreter() {
        // Create CommandExecutor with default interpreter
        let executor = CommandExecutor::new::<&str>(None);
        // Assert that default interpreter is sh
        assert_eq!(format!("{:?}", executor.command), "\"sh\" \"-c\"");
    }

    #[test]
    fn test_command_executor_new_custom_interpreter() {
        // Create CommandExecutor with custom interpreter
        let executor = CommandExecutor::new(Some("bash"));
        // Assert that custom interpreter is bash
        assert_eq!(
            format!("{:?}", executor.command),
            "\"bash\" \"-c\""
        );
    }

    #[test]
    fn test_command_executor_set_command() {
        // Create CommandExecutor
        let mut executor = CommandExecutor::new::<&str>(None);
        // Set command
        let _ = executor.command("ls -l");
        // Assert that command is set correctly
        assert_eq!(
            format!("{:?}", executor.command),
            "\"sh\" \"-c\" \"ls -l\""
        );
    }

    // Tests for logging macros
    #[test]
    fn test_macro_log_start() {
        // As the logging macros simply print to stdout, we can't directly test their output.
        // We can, however, test if they compile and don't cause runtime errors.
        macro_log_start!("Test Operation", "Test Start Message");
    }

    #[test]
    fn test_macro_log_complete() {
        // Similar to macro_log_start, we test if the macro compiles without errors.
        macro_log_complete!(
            "Test Operation",
            "Test Completion Message"
        );
    }

    #[test]
    fn test_macro_log_error() {
        // Similar to macro_log_start, we test if the macro compiles without errors.
        macro_log_error!("Test Operation", "Test Error Message");
    }

    #[test]
    fn test_macro_execute_and_log() {
        // Similar to logging macros, we test if the macro compiles without errors.
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

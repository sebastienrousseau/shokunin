// Copyright © 2024 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Shell command execution macros and utilities
//!
//! This module provides utilities for executing shell commands, logging their execution,
//! and handling errors in a safe and controlled manner.
//!
//! # Features
//! - Safe shell command execution
//! - Comprehensive error handling
//! - Command execution logging
//! - Configurable shell interpreter
//!
//! # Examples
//! ```
//! use staticrux::macro_execute_and_log;
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let cmd = "ls -l";
//! let operation = "list_directory";
//! let start_msg = "Listing directory...";
//! let complete_msg = "Directory listing complete";
//! let error_msg = "Failed to list directory";
//!
//! macro_execute_and_log!(
//!     cmd,
//!     operation,
//!     start_msg,
//!     complete_msg,
//!     error_msg,
//!     None::<&str>,
//!     Some("bash")
//! )?;
//! # Ok(())
//! # }
//! ```

use std::ffi::OsStr;
use std::process::{Command, Output};
use thiserror::Error;

/// Errors that can occur during command execution
#[derive(Error, Debug, PartialEq)]
pub enum CommandError {
    /// The command string was empty
    #[error("Empty command provided")]
    EmptyCommand,

    /// The command execution failed
    #[error("Command execution failed: {0}")]
    ExecutionFailed(String),

    /// The shell interpreter was not found
    #[error("Shell interpreter not found: {0}")]
    InterpreterNotFound(String),

    /// The command output could not be captured
    #[error("Failed to capture command output: {0}")]
    OutputCaptureFailed(String),
}

/// Encapsulates command execution functionality with safety checks
#[derive(Debug)]
pub struct CommandExecutor {
    /// The command to execute
    command: Command,
    /// The original command string for logging
    command_str: String,
}

impl CommandExecutor {
    /// Creates a new CommandExecutor instance with the specified shell interpreter
    pub fn new<S>(interpreter: Option<S>) -> Result<Self, CommandError>
    where
        S: AsRef<str> + AsRef<OsStr>,
    {
        let shell =
            interpreter.as_ref().map(AsRef::as_ref).unwrap_or("sh");

        let mut command = Command::new(shell);
        command.arg("-c");

        Ok(CommandExecutor {
            command,
            command_str: String::new(),
        })
    }

    /// Sets the shell command to execute
    pub fn command<S: AsRef<str>>(&mut self, cmd: S) -> &mut Self {
        self.command_str = cmd.as_ref().to_string();
        self.command.arg(cmd.as_ref());
        self
    }

    /// Executes the command and returns the result
    pub fn execute(&mut self) -> Result<Output, CommandError> {
        if self.command_str.is_empty() {
            return Err(CommandError::EmptyCommand);
        }

        match self.command.output() {
            Ok(output) => {
                // Check if the command was successful
                if !output.status.success() {
                    let stderr =
                        String::from_utf8_lossy(&output.stderr);
                    return Err(CommandError::ExecutionFailed(
                        stderr.to_string(),
                    ));
                }
                Ok(output)
            }
            Err(e) => Err(CommandError::ExecutionFailed(e.to_string())),
        }
    }
}

/// Macro for executing a shell command with logging
///
/// # Arguments
///
/// * `$cmd` - The command to execute
/// * `$operation` - Description of the operation
/// * `$start_message` - Message to log before execution
/// * `$complete_message` - Message to log on successful completion
/// * `$error_message` - Message to log on failure
/// * `$output_dir` - Optional output directory
/// * `$interpreter` - Optional shell interpreter
///
/// # Returns
///
/// Returns a Result indicating success or failure
#[macro_export]
macro_rules! macro_execute_and_log {
    ($cmd:expr, $operation:expr, $start_message:expr, $complete_message:expr, $error_message:expr, $output_dir:expr, $interpreter:expr) => {{
        use log::{error, info};
        use $crate::macros::shell_macros::CommandExecutor;

        info!("[{}] {}", $operation, $start_message);

        let result = (|| -> Result<(), Box<dyn std::error::Error>> {
            let mut executor = CommandExecutor::new($interpreter)?;
            executor.command($cmd);

            let output = executor.execute()?;

            if !output.status.success() {
                let error = String::from_utf8_lossy(&output.stderr);
                return Err(format!("Command failed: {}", error).into());
            }

            if let Some(dir) = $output_dir {
                std::fs::write(
                    std::path::Path::new(dir)
                        .join(format!("{}.log", $operation)),
                    &output.stdout,
                )?;
            }

            Ok(())
        })();

        match result {
            Ok(()) => {
                info!("[{}] {}", $operation, $complete_message);
                Ok(())
            }
            Err(e) => {
                error!("[{}] {}: {}", $operation, $error_message, e);
                Err(e)
            }
        }
    }};
}

/// Macro for logging the start of an operation
#[macro_export]
macro_rules! macro_log_start {
    ($operation:expr, $message:expr) => {{
        log::info!("[{}] Starting: {}", $operation, $message);
    }};
}

/// Macro for logging the completion of an operation
#[macro_export]
macro_rules! macro_log_complete {
    ($operation:expr, $message:expr) => {{
        log::info!("[{}] Completed: {}", $operation, $message);
    }};
}

/// Macro for logging an error
#[macro_export]
macro_rules! macro_log_error {
    ($operation:expr, $message:expr) => {{
        log::error!("[{}] Error: {}", $operation, $message);
    }};
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str;

    #[test]
    fn test_command_executor_new() {
        assert!(CommandExecutor::new(None::<&str>).is_ok());
        assert!(CommandExecutor::new(Some("bash")).is_ok());
    }

    #[test]
    fn test_command_executor_empty_command() {
        let mut executor = CommandExecutor::new(None::<&str>).unwrap();
        assert_eq!(
            executor.execute().unwrap_err(),
            CommandError::EmptyCommand
        );
    }

    #[test]
    fn test_command_executor_echo() {
        let mut executor = CommandExecutor::new(None::<&str>).unwrap();
        executor.command("echo 'test'");
        let output = executor.execute().unwrap();
        assert_eq!(
            str::from_utf8(&output.stdout).unwrap().trim(),
            "test"
        );
    }

    #[test]
    fn test_command_executor_invalid_command() {
        let mut executor = CommandExecutor::new(None::<&str>).unwrap();
        executor.command("invalidcommand123");
        let err = executor.execute().unwrap_err();
        match err {
            CommandError::ExecutionFailed(msg) => {
                assert!(msg.contains("command not found"));
            }
            _ => panic!("Expected ExecutionFailed error"),
        }
    }

    #[test]
    fn test_command_executor_with_bash() {
        let mut executor = CommandExecutor::new(Some("bash")).unwrap();
        executor.command("echo $BASH_VERSION");
        assert!(executor.execute().is_ok());
    }

    #[test]
    fn test_command_executor_with_stderr() {
        let mut executor = CommandExecutor::new(None::<&str>).unwrap();
        executor.command("ls nonexistentfile");
        let err = executor.execute().unwrap_err();
        match err {
            CommandError::ExecutionFailed(msg) => {
                assert!(msg.contains("No such file or directory"));
            }
            _ => panic!("Expected ExecutionFailed error"),
        }
    }
}

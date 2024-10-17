// Copyright © 2024 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! This module provides utilities for executing shell commands, logging their execution, and handling errors.
//!
//! It includes a `CommandExecutor` struct for encapsulating command execution functionality,
//! custom error types for command execution failures, and macros for logging the start,
//! completion, and errors of shell command operations.
//!
//! # Configuration Options
//!
//! - `CommandExecutor::new`: Allows specifying the shell interpreter to use, which defaults to "sh".
//! - Logging macros (`macro_log_start`, `macro_log_complete`, `macro_log_error`): Customization options
//!   for logging start, completion, and error messages.
//!
//! # Examples
//!
//! ```
//! use ssg_core::{macro_execute_and_log, macro_log_start, macro_log_complete, macro_log_error};
//! use ssg_core::macros::shell_macros::CommandError;
//! use std::error::Error;
//!
//! fn main() -> Result<(), Box<dyn Error>> {
//!     let cmd = "ls -l";
//!     let pkg = "example_pkg";
//!     let operation = "list_directory";
//!     let start_message = "Listing directory contents...";
//!     let complete_message = "Listing directory completed successfully.";
//!     let error_message = "Listing directory failed.";
//!
//!     match macro_execute_and_log!(cmd, pkg, operation, start_message, complete_message, error_message, Some("output"), Some("bash")) {
//!         Ok(()) => println!("Command executed successfully."),
//!         Err(err) => eprintln!("Error executing command: {}", err),
//!     }
//!     Ok(())
//! }
//! ```
//!
//! This example demonstrates how to use the `macro_execute_and_log` macro to execute a shell command,
//! log the start and completion of the operation, and handle any errors that occur.
//!
//! # Logging Macros Customization
//!
//! The logging macros (`macro_log_start`, `macro_log_complete`, `macro_log_error`) can be customized
//! according to specific logging requirements. Users can implement their own logging strategies
//! by replacing the default `println!` statements with custom logging logic.
//!
//! For example, users can redirect log messages to a file, integrate with a logging framework,
//! or send log messages to a remote logging server by modifying the macro implementations.
//!
//! Additionally, users can adjust log message formats, add timestamps, or include additional metadata
//! in log messages by modifying the format strings within the macros.
//!
//! # Note
//!
//! This module assumes familiarity with Rust macros and their usage. Users are encouraged
//! to review Rust macro documentation for a better understanding of how to work with macros effectively.

// Standard library imports
use std::ffi::OsStr;
use std::fmt;
use std::process::{Command, Output};

/// Custom error type for command execution failures.
#[derive(Debug)]
pub enum CommandError {
    /// Indicates that the command string was empty.
    EmptyCommand,
    /// Indicates that the command execution failed with the provided error message.
    ExecutionFailed(String),
}

impl fmt::Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CommandError::EmptyCommand => write!(f, "Empty command"),
            CommandError::ExecutionFailed(msg) => {
                write!(f, "Command execution failed: {}", msg)
            }
        }
    }
}

impl PartialEq for CommandError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                CommandError::EmptyCommand,
                CommandError::EmptyCommand,
            ) => true,
            (
                CommandError::ExecutionFailed(msg1),
                CommandError::ExecutionFailed(msg2),
            ) => msg1 == msg2,
            _ => false,
        }
    }
}

/// Encapsulates command execution functionality.
#[derive(Debug)]
pub struct CommandExecutor {
    /// The command to execute.
    pub command: Command,
}

impl CommandExecutor {
    /// Creates a new CommandExecutor instance.
    pub fn new<S>(interpreter: Option<S>) -> Self
    where
        S: AsRef<str> + From<&'static str> + AsRef<OsStr>,
    {
        // Initialize the command with the specified shell interpreter or default to "sh"
        let mut command =
            Command::new(interpreter.unwrap_or_else(|| "sh".into()));
        // Specify the shell command to execute
        command.arg("-c");
        CommandExecutor { command }
    }

    /// Sets the shell command to execute.
    pub fn command<S: AsRef<str>>(&mut self, cmd: S) -> &mut Self {
        // Append the command to the existing command
        self.command.arg(cmd.as_ref());
        self
    }

    /// Executes the command and returns the result.
    pub fn execute(&mut self) -> Result<Output, CommandError> {
        // Get the command as a string
        let cmd_str = format!("{:?}", self.command);
        // Check if the command is empty
        if cmd_str.is_empty() {
            return Err(CommandError::EmptyCommand);
        }
        // Execute the command and handle any errors
        self.command.output().map_err(|err| {
            CommandError::ExecutionFailed(err.to_string())
        })
    }
}

/// Executes a shell command, logs the start and completion of the operation, and handles any errors that occur.
///
/// # Parameters
///
/// * `$cmd`: The shell command to execute.
/// * `$pkg`: The name of the package the command is being run on.
/// * `$operation`: A description of the operation being performed.
/// * `$start_message`: The log message to be displayed at the start of the operation.
/// * `$complete_message`: The log message to be displayed upon successful completion of the operation.
/// * `$error_message`: The log message to be displayed in case of an error.
/// * `$output_dir`: Optional parameter for specifying the directory to write command output.
/// * `$interpreter`: Optional parameter for specifying the shell interpreter (default is "sh").
///
/// # Returns
///
/// Returns a `Result<(), CommandError>` indicating the success or failure of the operation.
///
/// # Example
///
/// ```
/// use ssg_core::{macro_execute_and_log, macro_log_start, macro_log_complete, macro_log_error};
/// use ssg_core::macros::shell_macros::CommandError;
/// use std::error::Error;
///
/// fn main() -> Result<(), Box<dyn Error>> {
///     let cmd = "ls -l";
///     let pkg = "example_pkg";
///     let operation = "list_directory";
///     let start_message = "Listing directory contents...";
///     let complete_message = "Listing directory completed successfully.";
///     let error_message = "Listing directory failed.";
///
///     match macro_execute_and_log!(cmd, pkg, operation, start_message, complete_message, error_message, Some("output"), Some("bash")) {
///         Ok(()) => println!("Command executed successfully."),
///         Err(err) => eprintln!("Error executing command: {}", err),
///     }
///     Ok(())
/// }
/// ```
#[macro_export]
macro_rules! macro_execute_and_log {
    ($cmd:expr, $pkg:expr, $operation:expr, $start_message:expr, $complete_message:expr, $error_message:expr, $output_dir:expr, $interpreter:expr) => {{
        // Create a new CommandExecutor instance
        let mut executor =
            $crate::macros::shell_macros::CommandExecutor::new(
                $interpreter,
            );
        // Set the command to execute
        executor.command($cmd);
        // Execute the command and handle the result
        let result = executor.execute();

        match result {
            Ok(_) => {
                // Log the successful completion of the operation
                macro_log_complete!($operation, $complete_message);
                Ok(())
            }
            Err(err) => {
                // Log the error if the command execution failed
                macro_log_error!($operation, $error_message);
                Err(err)
            }
        }
    }};
}

/// Macro to log the start of an operation.
///
/// # Parameters
///
/// * `$operation`: A description of the operation being performed.
/// * `$message`: The log message to be displayed at the start of the operation.
///
#[macro_export]
macro_rules! macro_log_start {
    ($operation:expr, $message:expr) => {{
        // Your implementation for logging start message goes here
        println!("Start message: {}", $message);
    }};
}

/// Macro to log the completion of an operation.
///
/// # Parameters
///
/// * `$operation`: A description of the operation being performed.
/// * `$message`: The log message to be displayed upon successful completion of the operation.
///
#[macro_export]
macro_rules! macro_log_complete {
    ($operation:expr, $message:expr) => {{
        // Your implementation for logging completion message goes here
        println!("Completion message: {}", $message);
    }};
}

/// Macro to log an error.
///
/// # Parameters
///
/// * `$operation`: A description of the operation being performed.
/// * `$message`: The log message to be displayed in case of an error.
///
#[macro_export]
macro_rules! macro_log_error {
    ($operation:expr, $message:expr) => {{
        // Your implementation for logging error message goes here
        println!("Error message: {}", $message);
    }};
}

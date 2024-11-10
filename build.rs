//! This build script verifies that the current Rustc compiler version meets
//! the minimum required version for building the Shokunin Static Site Generator.
//! If the Rustc version is lower than the specified minimum, the build will
//! terminate with a non-zero exit code and display an error message.

use std::process;

/// Specifies the minimum required Rustc version for building this project.
///
/// This version is defined as a constant to allow easy modification
/// if the minimum required version changes in the future.
///
/// # Example
///
/// To update the required version, simply modify the value assigned to `MIN_VERSION`.
///
/// ```rust
/// const MIN_VERSION: &str = "1.58"; // Updates minimum version requirement to 1.58
/// ```
const MIN_VERSION: &str = "1.56";

/// Checks if the current Rustc version meets the minimum required version.
///
/// The function uses the `version_check` crate to verify that the Rust compiler's
/// version is at least `MIN_VERSION`. If the version is insufficient, an error
/// message is printed, and the process exits with code `1`.
///
/// # Behavior
///
/// - If the Rustc version is equal to or greater than `MIN_VERSION`, the build process
///   continues as normal.
/// - If the Rustc version is less than `MIN_VERSION` or cannot be determined,
///   the build process terminates with a helpful error message.
///
/// # Example
///
/// This function is automatically called in the main build script and does not require
/// any additional code in `build.rs`.
///
/// # Panics
///
/// This function will terminate the build process with an exit code of `1` if
/// the Rustc version is insufficient.
///
/// # Errors
///
/// The error message directs users to update their Rust toolchain if it does
/// not meet the required minimum version.
fn main() {
    if version_check::is_min_version(MIN_VERSION) != Some(true) {
        eprintln!(
            "'Shokunin Static Site Generator' requires Rustc version >= {}.",
            MIN_VERSION
        );
        process::exit(1);
    }
}

// Copyright © 2024 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

// Import the Uuid type from the uuid crate
use uuid::Uuid;

/// Generates a unique string.
///
/// This function generates a new unique string using UUID version 4 (random).
///
/// # Returns
///
/// A string containing the generated unique identifier.
///
/// # Examples
///
/// ```
/// use staticrux::utilities::uuid::generate_unique_string;
///
/// let unique_string = generate_unique_string();
/// println!("Unique string: {}", unique_string);
/// ```
pub fn generate_unique_string() -> String {
    // Generate a new UUID v4 (random) and convert it to a string
    Uuid::new_v4().to_string()
}

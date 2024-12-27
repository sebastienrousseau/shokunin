//! Tests for the `generate_unique_string` function from `utilities::uuid`.

// Copyright © 2025 Shokunin Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

#[cfg(test)]
mod tests {
    use staticdatagen::utilities::uuid::generate_unique_string;
    use std::{
        sync::{Arc, Mutex},
        thread,
    };
    use uuid::Uuid;

    /// Tests the length of the generated unique string.
    #[test]
    fn test_generate_unique_string_length() {
        let unique_string = generate_unique_string();

        // Ensure the length of the generated string is 36 characters
        assert_eq!(unique_string.len(), 36);
    }

    /// Tests the UUID format of the generated unique string.
    #[test]
    fn test_generate_unique_string_uuid_format() {
        let unique_string = generate_unique_string();
        let parsed_uuid = Uuid::parse_str(&unique_string);

        // Ensure the generated string can be parsed into a valid UUID
        assert!(parsed_uuid.is_ok());
    }

    /// Tests the performance of generating unique strings.
    #[test]
    fn test_generate_unique_string_performance() {
        const NUM_GENERATIONS: usize = 10000;
        // Measure the time taken to generate NUM_GENERATIONS unique strings
        let start_time = std::time::Instant::now();
        for _ in 0..NUM_GENERATIONS {
            let _ = generate_unique_string();
        }
        let elapsed_time = start_time.elapsed();

        // Ensure the function completes within a reasonable time frame (e.g., less than 1 second)
        assert!(elapsed_time < std::time::Duration::from_secs(1));
    }

    /// Tests the concurrency and thread safety of generating unique strings.
    #[test]
    fn test_generate_unique_string_concurrency() {
        const NUM_THREADS: usize = 8;
        const NUM_GENERATIONS_PER_THREAD: usize = 1000;

        // Create a shared counter to track the number of generated strings
        let counter = Arc::new(Mutex::new(0));
        // Spawn multiple threads to generate unique strings concurrently
        let mut handles = vec![];
        for _ in 0..NUM_THREADS {
            let counter = Arc::clone(&counter);
            let handle = thread::spawn(move || {
                for _ in 0..NUM_GENERATIONS_PER_THREAD {
                    let _ = generate_unique_string();
                    let mut count = counter.lock().unwrap();
                    *count += 1;
                }
            });
            handles.push(handle);
        }

        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }

        // Ensure the total number of generated strings matches the expected count
        let count = counter.lock().unwrap();
        assert_eq!(*count, NUM_THREADS * NUM_GENERATIONS_PER_THREAD);
    }
}

// Copyright © 2025 Shokunin Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! This crate tests CNAME generation functionality using `CnameGenerator`.

mod tests {
    use staticdatagen::generators::cname::CnameGenerator;
    use std::collections::HashMap;

    #[test]
    fn test_create_cname_data_with_valid_cname() {
        let mut metadata = HashMap::new();
        let _ = metadata
            .insert("cname".to_string(), "example.com".to_string());

        let result = CnameGenerator::from_metadata(&metadata);
        assert!(result.is_ok(), "Expected CNAME generation to succeed");

        let cname_data = result.unwrap();
        assert!(
            cname_data.contains("example.com"),
            "Expected generated data to contain 'example.com', got: {}",
            cname_data
        );
    }

    #[test]
    fn test_create_cname_data_with_missing_cname() {
        let metadata = HashMap::new(); // No "cname" entry

        let result = CnameGenerator::from_metadata(&metadata);
        assert!(
            result.is_err(),
            "Expected error due to missing 'cname' key in metadata"
        );
    }
}

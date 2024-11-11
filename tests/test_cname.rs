// Copyright © 2024 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

#[cfg(test)]
mod tests {
    use staticdatagen::modules::cname::create_cname_data;
    use std::collections::HashMap;

    #[test]
    fn test_create_cname_data_with_valid_cname() {
        let mut metadata = HashMap::new();
        let _ = metadata
            .insert("cname".to_string(), "example.com".to_string());

        let cname_data = create_cname_data(&metadata);

        assert_eq!(cname_data.cname, "example.com");
    }

    #[test]
    fn test_create_cname_data_with_missing_cname() {
        let metadata = HashMap::new(); // Empty metadata

        let cname_data = create_cname_data(&metadata);

        assert_eq!(cname_data.cname, "");
    }
}

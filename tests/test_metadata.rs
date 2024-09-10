// Copyright © 2024 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

#[cfg(test)]
mod tests {
    use ssg_core::{metadata::service::extract_and_prepare_metadata, models::data::MetaTagGroups, modules::metatags::{
        generate_apple_meta_tags, generate_ms_meta_tags,
        generate_og_meta_tags, generate_primary_meta_tags,
        generate_twitter_meta_tags,
    }};
    use std::collections::HashMap;

    #[test]
    fn test_extract_and_prepare_metadata_with_valid_content() {
        let file_content = "\
---
author: \"Jane Doe\"
category: \"Rust\"
description: \"A blog about Rust programming.\"
layout: \"post\"
permalink: \"https://example.com/blog/rust\"
keywords: \"rust,programming\"
title: \"Rust\"
---
This is a blog about Rust programming.
";

        let (metadata, keywords, all_meta_tags) =
            extract_and_prepare_metadata(file_content);

        // Check extracted metadata
        let mut expected_metadata = HashMap::new();
        let _ = expected_metadata
            .insert("author".to_string(), "Jane Doe".to_string());
        let _ = expected_metadata
            .insert("category".to_string(), "Rust".to_string());
        let _ = expected_metadata.insert(
            "description".to_string(),
            "A blog about Rust programming.".to_string(),
        );
        let _ = expected_metadata
            .insert("layout".to_string(), "post".to_string());
        let _ = expected_metadata.insert(
            "permalink".to_string(),
            "https://example.com/blog/rust".to_string(),
        );
        let _ = expected_metadata.insert(
            "keywords".to_string(),
            "rust,programming".to_string(),
        );
        let _ = expected_metadata
            .insert("title".to_string(), "Rust".to_string());
        assert_eq!(metadata, expected_metadata);

        // Check extracted keywords
        assert_eq!(keywords, vec!["rust", "programming"]);

        // Check generated meta tags
        let expected_meta_tags = MetaTagGroups {
            apple: generate_apple_meta_tags(&metadata),
            primary: generate_primary_meta_tags(&metadata),
            og: generate_og_meta_tags(&metadata),
            ms: generate_ms_meta_tags(&metadata),
            twitter: generate_twitter_meta_tags(&metadata),
        };
        assert_eq!(all_meta_tags, expected_meta_tags);
    }
}

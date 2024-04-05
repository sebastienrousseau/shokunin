// Copyright © 2024 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

use crate::models::data::NewsSiteMapData; // Import the NewsSiteMapData model from the local crate.
use regex::Regex; // Import the Regex type from the regex crate.
use std::collections::HashMap; // Import the HashMap type from the standard library.

/// Generates `NewsSiteMapData` from metadata.
///
/// # Arguments
/// * `metadata` - A hashmap containing page metadata, including last build date, change frequency, and page location.
///
/// # Returns
/// A `NewsSiteMapData` object populated with values from the metadata.
pub fn create_news_site_map_data(
    metadata: &HashMap<String, String>,
) -> NewsSiteMapData {
    // Convert the last build date from metadata to the desired format.
    let news_publication_date = convert_date_format(
        metadata.get("news_publication_date").unwrap_or(&"".to_string()),
    );

    // Construct and return NewsSiteMapData with converted and extracted metadata values.
    NewsSiteMapData {
        news_genres: metadata.get("news_genres").unwrap_or(&"".to_string()).to_string(),
        news_keywords: metadata.get("news_keywords").unwrap_or(&"".to_string()).to_string(),
        news_language: metadata.get("news_language").unwrap_or(&"".to_string()).to_string(),
        news_loc: metadata.get("news_loc").unwrap_or(&"".to_string()).to_string(),
        news_publication_date,
        news_publication_name: metadata.get("news_publication_name").unwrap_or(&"".to_string()).to_string(),
        news_title: metadata.get("news_title").unwrap_or(&"".to_string()).to_string(),
    }
}

/// Converts date strings from various formats to "YYYY-MM-DD".
///
/// Supports conversion from "DD MMM YYYY" format and checks if input is already in target format.
///
/// # Arguments
/// * `input` - A string slice representing the input date.
///
/// # Returns
/// A string representing the date in "YYYY-MM-DD" format, or the original input if conversion is not applicable.
fn convert_date_format(input: &str) -> String {
    // Define a regex to identify dates in the "DD MMM YYYY" format.
    let re = Regex::new(r"\d{2} \w{3} \d{4}").unwrap();

    // Check if input matches the expected date format.
    if let Some(date_match) = re.find(input) {
        let date_str = date_match.as_str();
        let parts: Vec<&str> = date_str.split_whitespace().collect();

        // Proceed with conversion if input format matches.
        if parts.len() == 3 {
            let day = parts[0];
            let month = match parts[1] {
                "Jan" => "01",
                "Feb" => "02",
                "Mar" => "03",
                "Apr" => "04",
                "May" => "05",
                "Jun" => "06",
                "Jul" => "07",
                "Aug" => "08",
                "Sep" => "09",
                "Oct" => "10",
                "Nov" => "11",
                "Dec" => "12",
                _ => return input.to_string(), // Return original input for unrecognized months.
            };
            let year = parts[2];

            // Return the formatted date string.
            return format!("{}-{}-{}", year, month, day);
        }
    }

    // Return the original input if it's already in the correct format or doesn't match "DD MMM YYYY".
    input.to_string()
}

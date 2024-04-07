// Copyright © 2024 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

use crate::models::data::NewsData; // Import the NewsData model from the local crate.
use std::collections::HashMap; // Import the HashMap type from the standard library.

/// Generates `NewsData` from metadata.
///
/// # Arguments
/// * `metadata` - A hashmap containing page metadata, including last build date, change frequency, and page location.
///
/// # Returns
/// A `NewsData` object populated with values from the metadata.
pub fn create_news_site_map_data(
    metadata: &HashMap<String, String>,
) -> NewsData {
    // Convert the last build date from metadata to the desired format.
    let news_publication_date = convert_date_format(
        metadata.get("news_publication_date").unwrap_or(&"".to_string()),
    );

    // Construct and return NewsData with converted and extracted metadata values.
    NewsData {
        news_genres: metadata.get("news_genres").unwrap_or(&"".to_string()).to_string(),
        news_image_loc: metadata.get("news_image_loc").cloned().unwrap_or_default(),
        news_keywords: metadata.get("news_keywords").cloned().unwrap_or_default(),
        news_language: metadata.get("news_language").cloned().unwrap_or_default(),
        news_loc: metadata.get("news_loc").cloned().unwrap_or_default(),
        news_publication_date,
        news_publication_name: metadata.get("news_publication_name").cloned().unwrap_or_default(),
        news_title: metadata.get("news_title").cloned().unwrap_or_default(),
    }
}

/// Converts date strings from "Tue, 20 Feb 2024 15:15:15 GMT" format to "2024-02-20T15:15:15+00:00" format.
///
/// # Arguments
/// * `input` - A string slice representing the input date.
///
/// # Returns
/// A string representing the date in "YYYY-MM-DDTHH:MM:SS+00:00" format, or the original input if conversion is not applicable.
fn convert_date_format(input: &str) -> String {
    // Split the input string by whitespace to extract date components.
    let parts: Vec<&str> = input.split_whitespace().collect();

    // Check if the input date string has the correct number of components.
    if parts.len() == 6 {
        let day = parts[1];
        let month = match parts[2] {
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
        let year = parts[3];
        let time = parts[4];

        // Assemble the converted date string.
        let converted_date = format!("{}-{}-{}T{}", year, month, day, time);

        // Append the timezone information.
        let timezone = "+00:00";
        return format!("{}{}", converted_date, timezone);
    }

    // Return the original input if it's not in the expected format.
    input.to_string()
}
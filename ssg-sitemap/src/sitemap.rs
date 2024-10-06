// src/sitemap.rs

use crate::error::SitemapError;
use dtt::datetime::DateTime;
use regex::Regex;
use std::collections::HashMap;
use std::str::FromStr;
use xml::writer::{EventWriter, XmlEvent};

/// Represents the data for a sitemap entry.
#[derive(Debug, Clone)]
pub struct SiteMapData {
    /// The change frequency of the URL.
    pub changefreq: String,
    /// The last modification date of the URL.
    pub lastmod: String,
    /// The location (URL) of the page.
    pub loc: String,
}

/// Represents the change frequency of a URL in the sitemap.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeFreq {
    /// The page is always changing.
    Always,
    /// The page changes hourly.
    Hourly,
    /// The page changes daily.
    Daily,
    /// The page changes weekly.
    Weekly,
    /// The page changes monthly.
    Monthly,
    /// The page changes yearly.
    Yearly,
    /// The page never changes.
    Never,
}

impl FromStr for ChangeFreq {
    type Err = SitemapError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "always" => Ok(ChangeFreq::Always),
            "hourly" => Ok(ChangeFreq::Hourly),
            "daily" => Ok(ChangeFreq::Daily),
            "weekly" => Ok(ChangeFreq::Weekly),
            "monthly" => Ok(ChangeFreq::Monthly),
            "yearly" => Ok(ChangeFreq::Yearly),
            "never" => Ok(ChangeFreq::Never),
            _ => Err(SitemapError::InvalidChangeFreq(s.to_string())),
        }
    }
}

/// Generates `SiteMapData` from metadata.
///
/// # Arguments
/// * `metadata` - A hashmap containing page metadata, including last build date, change frequency, and page location.
///
/// # Returns
/// A `SiteMapData` object populated with values from the metadata.
pub fn create_site_map_data(
    metadata: &HashMap<String, String>,
) -> SiteMapData {
    // Convert the last build date from metadata to the desired format.
    let lastmod = convert_date_format(
        metadata.get("last_build_date").unwrap_or(&"".to_string()),
    );

    // Construct and return SiteMapData with converted and extracted metadata values.
    SiteMapData {
        changefreq: metadata
            .get("changefreq")
            .cloned()
            .unwrap_or_default(),
        lastmod,
        loc: metadata.get("permalink").cloned().unwrap_or_default(),
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
pub fn convert_date_format(input: &str) -> String {
    // Define a regex to identify dates in the "DD MMM YYYY" format.
    let re = Regex::new(r"(\d{2}) (\w{3}) (\d{4})").unwrap();

    // Check if input matches the expected date format.
    if let Some(caps) = re.captures(input) {
        // Extract the day, month, and year from the matched date.
        let day = caps.get(1).unwrap().as_str();
        let month = caps.get(2).unwrap().as_str();
        let year = caps.get(3).unwrap().as_str();

        // Convert the short month name to a numerical representation.
        let month_num = match month.to_lowercase().as_str() {
            "jan" => "01",
            "feb" => "02",
            "mar" => "03",
            "apr" => "04",
            "may" => "05",
            "jun" => "06",
            "jul" => "07",
            "aug" => "08",
            "sep" => "09",
            "oct" => "10",
            "nov" => "11",
            "dec" => "12",
            _ => return input.to_string(), // If the month is not recognized, return the input as-is.
        };

        // Format the result as YYYY-MM-DD.
        return format!("{}-{}-{}", year, month_num, day);
    }

    // If the input doesn't match the expected format, try parsing it as an ISO 8601 date
    if let Ok(dt) = DateTime::parse(input) {
        if let Ok(formatted) = dt.format("[year]-[month]-[day]") {
            return formatted;
        }
    }

    // Return the original input if parsing fails
    input.to_string()
}

/// Represents a complete sitemap.
#[derive(Debug, Default, Clone)]
pub struct Sitemap {
    entries: Vec<SiteMapData>,
}

impl Sitemap {
    /// Creates a new empty `Sitemap`.
    pub fn new() -> Self {
        Sitemap {
            entries: Vec::new(),
        }
    }

    /// Adds a new entry to the sitemap.
    pub fn add_entry(&mut self, entry: SiteMapData) {
        self.entries.push(entry);
    }

    /// Generates the XML representation of the sitemap.
    pub fn to_xml(&self) -> Result<String, SitemapError> {
        let mut output = Vec::new();
        let mut writer = EventWriter::new(&mut output);

        writer.write(XmlEvent::StartDocument {
            version: xml::common::XmlVersion::Version10,
            encoding: Some("UTF-8"),
            standalone: None,
        })?;

        writer.write(XmlEvent::start_element("urlset").default_ns(
            "http://www.sitemaps.org/schemas/sitemap/0.9",
        ))?;

        for entry in &self.entries {
            writer.write(XmlEvent::start_element("url"))?;

            writer.write(XmlEvent::start_element("loc"))?;
            writer.write(XmlEvent::characters(&entry.loc))?;
            writer.write(XmlEvent::end_element())?;

            writer.write(XmlEvent::start_element("lastmod"))?;
            writer.write(XmlEvent::characters(&entry.lastmod))?;
            writer.write(XmlEvent::end_element())?;

            writer.write(XmlEvent::start_element("changefreq"))?;
            writer.write(XmlEvent::characters(&entry.changefreq))?;
            writer.write(XmlEvent::end_element())?;

            writer.write(XmlEvent::end_element())?;
        }

        writer.write(XmlEvent::end_element())?;

        String::from_utf8(output)
            .map_err(|e| SitemapError::EncodingError(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dtt::dtt_now;

    #[test]
    fn test_create_site_map_data() {
        let mut metadata = HashMap::new();
        metadata.insert(
            "last_build_date".to_string(),
            "20 May 2023".to_string(),
        );
        metadata.insert("changefreq".to_string(), "weekly".to_string());
        metadata.insert(
            "permalink".to_string(),
            "https://example.com".to_string(),
        );

        let site_map_data = create_site_map_data(&metadata);

        assert_eq!(site_map_data.lastmod, "2023-05-20");
        assert_eq!(site_map_data.changefreq, "weekly");
        assert_eq!(site_map_data.loc, "https://example.com");
    }

    #[test]
    fn test_convert_date_format() {
        assert_eq!(convert_date_format("20 May 2023"), "2023-05-20");
        assert_eq!(convert_date_format("2023-05-20"), "2023-05-20");
        assert_eq!(convert_date_format("Invalid Date"), "Invalid Date");
    }

    #[test]
    fn test_sitemap_to_xml() -> Result<(), SitemapError> {
        let mut sitemap = Sitemap::new();
        sitemap.add_entry(SiteMapData {
            loc: "https://example.com".to_string(),
            lastmod: "2023-05-20".to_string(),
            changefreq: "weekly".to_string(),
        });

        let xml = sitemap.to_xml()?;
        assert!(xml.contains("<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">"));
        assert!(xml.contains("<url>"));
        assert!(xml.contains("<loc>https://example.com</loc>"));
        assert!(xml.contains("<lastmod>2023-05-20</lastmod>"));
        assert!(xml.contains("<changefreq>weekly</changefreq>"));
        Ok(())
    }

    #[test]
    fn test_dtt_now_macro() {
        let now = dtt_now!();
        assert!(now.year() >= 2023);
    }
}

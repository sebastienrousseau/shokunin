use crate::error::FrontmatterError;
use crate::types::Format;

/// Extracts raw frontmatter from the content, detecting YAML, TOML, or JSON formats.
///
/// This function tries to extract frontmatter based on the common delimiters for
/// YAML (`---`), TOML (`+++`), and JSON (`{}`). If frontmatter is detected, it
/// returns the extracted frontmatter and the remaining content.
///
/// # Arguments
///
/// * `content` - The full content string that may contain frontmatter.
///
/// # Returns
///
/// A `Result` containing a tuple of two `&str` slices: the raw frontmatter and the remaining content.
/// If no valid frontmatter format is found, it returns `FrontmatterError::InvalidFormat`.
///
/// # Errors
///
/// - `FrontmatterError::InvalidFormat`: When the frontmatter format is not recognized.
/// - `FrontmatterError::ExtractionError`: When there is an issue extracting frontmatter.
///
/// # Example
///
/// ```rust
/// use ssg_frontmatter::extractor::{extract_delimited_frontmatter, extract_raw_frontmatter, extract_json_frontmatter};
/// let content = "---\ntitle: Example\n---\nContent here";
/// let result = extract_raw_frontmatter(content).unwrap();
/// assert_eq!(result.0, "title: Example");
/// assert_eq!(result.1, "Content here");
/// ```
pub fn extract_raw_frontmatter(
    content: &str,
) -> Result<(&str, &str), FrontmatterError> {
    // Try to extract YAML frontmatter.
    if let Some(yaml) =
        extract_delimited_frontmatter(content, "---\n", "\n---\n")
    {
        let remaining = content.split("\n---\n").nth(1).unwrap_or("");
        return Ok((yaml, remaining));
    }
    // Try to extract TOML frontmatter.
    if let Some(toml) =
        extract_delimited_frontmatter(content, "+++\n", "\n+++\n")
    {
        let remaining = content.split("\n+++\n").nth(1).unwrap_or("");
        return Ok((toml, remaining));
    }
    // Try to extract JSON frontmatter.
    if let Some(json) = extract_json_frontmatter(content) {
        let remaining = &content[json.len()..];
        return Ok((json, remaining.trim_start()));
    }
    // Return an error if no valid frontmatter format is found.
    Err(FrontmatterError::InvalidFormat)
}

/// Detects the format of the extracted frontmatter.
///
/// This function analyzes the raw frontmatter and determines whether it is in YAML,
/// TOML, or JSON format by examining the structure of the data.
///
/// # Arguments
///
/// * `raw_frontmatter` - The extracted frontmatter as a string slice.
///
/// # Returns
///
/// A `Result` containing the detected `Format` (either `Json`, `Toml`, or `Yaml`).
///
/// # Errors
///
/// - `FrontmatterError::InvalidFormat`: If the format cannot be determined.
///
/// # Example
///
/// ```rust
/// use ssg_frontmatter::extractor::detect_format;
/// use ssg_frontmatter::Format;
/// let raw = "---\ntitle: Example\n---";
/// let format = detect_format(raw).unwrap();
/// assert_eq!(format, Format::Yaml);
/// ```
pub fn detect_format(
    raw_frontmatter: &str,
) -> Result<Format, FrontmatterError> {
    let trimmed = raw_frontmatter.trim_start();

    // Detect JSON format by checking for a leading '{' character.
    if trimmed.starts_with('{') {
        Ok(Format::Json)
    }
    // Detect TOML format by checking if the frontmatter contains '=' (key-value pairs).
    else if trimmed.contains('=') {
        Ok(Format::Toml)
    }
    // Default to YAML if no other format matches.
    else {
        Ok(Format::Yaml)
    }
}

/// Extracts frontmatter enclosed by the given start and end delimiters.
///
/// This function checks for frontmatter enclosed by delimiters like `---` for YAML or `+++` for TOML.
/// It returns the extracted frontmatter if the delimiters are found.
///
/// # Arguments
///
/// * `content` - The full content string containing frontmatter.
/// * `start_delim` - The starting delimiter (e.g., `---\n` for YAML).
/// * `end_delim` - The ending delimiter (e.g., `\n---\n` for YAML).
///
/// # Returns
///
/// An `Option` containing the extracted frontmatter as a string slice. Returns `None`
/// if the delimiters are not found.
///
/// # Example
///
/// ```rust
/// use ssg_frontmatter::extractor::extract_delimited_frontmatter;
/// let content = "---\ntitle: Example\n---\nContent";
/// let frontmatter = extract_delimited_frontmatter(content, "---\n", "\n---\n").unwrap();
/// assert_eq!(frontmatter, "title: Example");
/// ```
pub fn extract_delimited_frontmatter<'a>(
    content: &'a str,
    start_delim: &str,
    end_delim: &str,
) -> Option<&'a str> {
    content.strip_prefix(start_delim)?.split(end_delim).next()
}

/// Extracts JSON frontmatter from the content by detecting balanced curly braces (`{}`).
///
/// This function attempts to locate a valid JSON object starting with `{` and checks for balanced
/// curly braces to identify the end of the frontmatter. If the JSON object is found, it returns
/// the frontmatter as a string slice.
///
/// # Arguments
///
/// * `content` - The full content string that may contain JSON frontmatter.
///
/// # Returns
///
/// An `Option` containing the extracted JSON frontmatter string. Returns `None` if no valid JSON frontmatter is detected.
///
/// # Example
///
/// ```rust
/// use ssg_frontmatter::extractor::extract_json_frontmatter;
/// let content = "{ \"title\": \"Example\" }\nContent";
/// let frontmatter = extract_json_frontmatter(content).unwrap();
/// assert_eq!(frontmatter, "{ \"title\": \"Example\" }");
/// ```
pub fn extract_json_frontmatter(content: &str) -> Option<&str> {
    let trimmed = content.trim_start();

    // If the content doesn't start with '{', it's not JSON frontmatter.
    if !trimmed.starts_with('{') {
        return None;
    }

    let mut brace_count = 0;

    // Iterate over the characters in the trimmed content, looking for balanced braces.
    for (idx, ch) in trimmed.char_indices() {
        match ch {
            '{' => brace_count += 1,
            '}' => {
                brace_count -= 1;
                // Once braces are balanced (brace_count == 0), we've reached the end of the JSON object.
                if brace_count == 0 {
                    return Some(&trimmed[..=idx]);
                }
            }
            _ => {}
        }
    }

    // If no balanced braces are found, return None.
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_raw_frontmatter_yaml() {
        let content = "---\ntitle: Example\n---\nContent here";
        let result = extract_raw_frontmatter(content).unwrap();
        assert_eq!(result.0, "title: Example");
        assert_eq!(result.1, "Content here");
    }

    #[test]
    fn test_extract_raw_frontmatter_toml() {
        let content = "+++\ntitle = \"Example\"\n+++\nContent here";
        let result = extract_raw_frontmatter(content).unwrap();
        assert_eq!(result.0, "title = \"Example\"");
        assert_eq!(result.1, "Content here");
    }

    #[test]
    fn test_extract_raw_frontmatter_json() {
        let content = "{ \"title\": \"Example\" }\nContent here";
        let result = extract_raw_frontmatter(content).unwrap();
        assert_eq!(result.0, "{ \"title\": \"Example\" }");
        assert_eq!(result.1, "Content here");
    }

    #[test]
    fn test_extract_raw_frontmatter_invalid() {
        let content = "Invalid frontmatter";
        let result = extract_raw_frontmatter(content);
        assert!(matches!(result, Err(FrontmatterError::InvalidFormat)));
    }

    #[test]
    fn test_detect_format() {
        let yaml = "title: Example";
        let toml = "title = \"Example\"";
        let json = "{ \"title\": \"Example\" }";

        assert_eq!(detect_format(yaml).unwrap(), Format::Yaml);
        assert_eq!(detect_format(toml).unwrap(), Format::Toml);
        assert_eq!(detect_format(json).unwrap(), Format::Json);
    }

    #[test]
    fn test_extract_delimited_frontmatter() {
        let content = "---\ntitle: Example\n---\nContent here";
        let result =
            extract_delimited_frontmatter(content, "---\n", "\n---\n")
                .unwrap();
        assert_eq!(result, "title: Example");
    }

    #[test]
    fn test_extract_json_frontmatter() {
        let content = "{ \"title\": \"Example\" }\nContent here";
        let result = extract_json_frontmatter(content).unwrap();
        assert_eq!(result, "{ \"title\": \"Example\" }");
    }
}

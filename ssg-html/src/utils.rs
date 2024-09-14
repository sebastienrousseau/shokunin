use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref FRONT_MATTER_REGEX: Regex =
        Regex::new(r"(?s)^---\s*$(.+?)^---\s*$").unwrap();
    static ref HEADER_REGEX: Regex =
        Regex::new(r"<(h[1-6])>(.+?)</h[1-6]>").unwrap();
}

/// Extract front matter from Markdown content
pub fn extract_front_matter(content: &str) -> String {
    FRONT_MATTER_REGEX.replace(content, "").trim().to_string()
}

/// Format a header with an ID and class
pub fn format_header_with_id_class(header: &str) -> String {
    HEADER_REGEX
        .replace(header, |caps: &regex::Captures| {
            let tag = &caps[1];
            let content = &caps[2];
            let binding = content
                .to_lowercase()
                .replace(|c: char| !c.is_alphanumeric(), "-");
            let id = binding.trim_matches('-');
            format!(
                r#"<{} id="{}" class="{}">{}</{}>

"#,
                tag, id, id, content, tag
            )
        })
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_front_matter() {
        let content = "---\ntitle: My Page\n---\n# Hello, world!\n\nThis is a test.";
        let result = extract_front_matter(content);
        assert_eq!(result, "---\ntitle: My Page\n---\n# Hello, world!\n\nThis is a test.");
    }

    #[test]
    fn test_format_header_with_id_class() {
        let header = "<h2>Hello, World!</h2>";
        let result = format_header_with_id_class(header);
        assert_eq!(result, "<h2 id=\"hello--world\" class=\"hello--world\">Hello, World!</h2>\n\n");
    }

    #[test]
    fn test_format_header_with_special_characters() {
        let header = "<h3>Test: Special & Characters</h3>";
        let result = format_header_with_id_class(header);
        assert_eq!(result, "<h3 id=\"test--special---characters\" class=\"test--special---characters\">Test: Special & Characters</h3>\n\n");
    }
}

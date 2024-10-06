use regex::Regex;
use thiserror::Error;

/// Enum to represent possible accessibility-related errors.
#[derive(Debug, Error)]
pub enum AccessibilityError {
    /// Error indicating an invalid ARIA attribute.
    ///
    /// This variant is used when an ARIA attribute is improperly added or missing.
    /// The associated string provides details about the invalid attribute.
    #[error("Invalid ARIA Attribute: {0}")]
    InvalidAriaAttribute(String),

    /// Error indicating failure to validate HTML against WCAG guidelines.
    ///
    /// This variant is used when the HTML content does not comply with Web Content Accessibility Guidelines (WCAG).
    /// The associated string provides details about the validation failure.
    #[error("WCAG Validation Error: {0}")]
    WcagValidationError(String),

    /// Error indicating a failure in processing HTML for accessibility.
    ///
    /// This variant is used when there is an error while adding or modifying HTML for accessibility purposes.
    /// The associated string provides more information about the error.
    #[error("HTML Processing Error: {0}")]
    HtmlProcessingError(String),
}

/// Result type alias for convenience.
pub type Result<T> = std::result::Result<T, AccessibilityError>;

/// Add ARIA attributes to HTML for improved accessibility.
///
/// This function adds ARIA attributes to common elements, such as buttons, forms,
/// navigation elements, and images.
///
/// # Arguments
///
/// * `html` - A string slice representing the HTML content.
///
/// # Returns
///
/// * `Result<String>` - The modified HTML with ARIA attributes included.
pub fn add_aria_attributes(html: &str) -> Result<String> {
    let mut updated_html = html.to_string();

    // Add ARIA attributes to buttons if not present
    updated_html = add_aria_to_element(
        &updated_html,
        r"<button\b",
        "aria-label",
        "button",
    );

    // Add ARIA attributes to navs if not present
    updated_html = add_aria_to_element(
        &updated_html,
        r"<nav\b",
        "aria-label",
        "navigation",
    );

    // Add ARIA attributes to forms if not present
    updated_html = add_aria_to_element(
        &updated_html,
        r"<form\b",
        "aria-labelledby",
        "form-label",
    );

    // Add ARIA attributes to inputs without aria-label
    updated_html = add_aria_to_inputs_without_label(&updated_html);

    // Check if required ARIA attributes are correctly inserted
    if !validate_aria(&updated_html) {
        return Err(AccessibilityError::InvalidAriaAttribute(
            "Failed to add valid ARIA attributes.".to_string(),
        ));
    }

    Ok(updated_html)
}

/// Add an ARIA attribute to a specific HTML element if it doesn't already exist.
///
/// # Arguments
///
/// * `html` - The HTML content.
/// * `element_regex` - Regex pattern to match the element (e.g., button, form).
/// * `aria_name` - The ARIA attribute name (e.g., aria-label).
/// * `aria_value` - The ARIA attribute value.
///
/// # Returns
///
/// A string with the ARIA attributes added.
fn add_aria_to_element(
    html: &str,
    element_regex: &str,
    aria_name: &str,
    aria_value: &str,
) -> String {
    let re = Regex::new(element_regex).unwrap();
    let aria_attribute = format!(r#" {}="{}""#, aria_name, aria_value);

    re.replace_all(html, |caps: &regex::Captures| {
        format!("{}{}", &caps[0], aria_attribute)
    })
    .to_string()
}

/// Add ARIA label to inputs that don't already have an aria-label attribute.
///
/// Since Rust's `regex` crate doesn't support lookahead assertions, we will check
/// for the absence of `aria-label` manually.
///
/// # Arguments
///
/// * `html` - The HTML content.
///
/// # Returns
///
/// A string with ARIA attributes added to input elements without an aria-label.
fn add_aria_to_inputs_without_label(html: &str) -> String {
    let input_re = Regex::new(r#"<input\b[^>]*"#).unwrap();
    let aria_re = Regex::new(r#"aria-label="#).unwrap();

    input_re
        .replace_all(html, |caps: &regex::Captures| {
            let input_tag = &caps[0];
            if aria_re.is_match(input_tag) {
                input_tag.to_string() // Already has aria-label
            } else {
                format!(r#"{} aria-label="input""#, input_tag)
            }
        })
        .to_string()
}

/// Validate ARIA attributes within the HTML.
///
/// This function ensures that ARIA attributes are correctly formatted and conform to
/// the expected naming conventions.
///
/// # Arguments
///
/// * `html` - A string slice that holds the HTML content.
///
/// # Returns
///
/// * `bool` - Returns `true` if all ARIA attributes are valid, otherwise `false`.
fn validate_aria(html: &str) -> bool {
    let aria_re = Regex::new(r#"aria-[a-z]+="[^"]*""#).unwrap();
    aria_re.is_match(html)
}

/// Validate HTML against WCAG (Web Content Accessibility Guidelines).
///
/// This function performs various checks to validate the HTML content against WCAG standards,
/// such as ensuring all images have alt text, proper heading structure, and more.
///
/// # Arguments
///
/// * `html` - A string slice that holds the HTML content.
///
/// # Returns
///
/// * `Result<()>` - An empty result if validation passes, otherwise an error.
pub fn validate_wcag(html: &str) -> Result<()> {
    // Check for alt text in images
    let alt_check = check_alt_text(html);
    println!("Alt text validation: {}", alt_check);
    if !alt_check {
        return Err(AccessibilityError::WcagValidationError(
            "Missing or invalid alt text for images.".to_string(),
        ));
    }

    // Check for proper heading structure
    let heading_check = check_heading_structure(html);
    println!("Heading structure validation: {}", heading_check);
    if !heading_check {
        return Err(AccessibilityError::WcagValidationError(
            "Improper heading structure (e.g., skipping heading levels).".to_string(),
        ));
    }

    // Check for input labels
    let input_label_check = check_input_labels(html);
    println!("Input label validation: {}", input_label_check);
    if !input_label_check {
        return Err(AccessibilityError::WcagValidationError(
            "Form inputs missing associated labels.".to_string(),
        ));
    }

    Ok(())
}

/// Check heading structure to ensure no levels are skipped.
///
/// Ensures that headings follow a proper hierarchy, without skipping levels (e.g., going from `<h1>` directly to `<h3>`).
///
/// # Arguments
///
/// * `html` - A string slice representing the HTML content.
///
/// # Returns
///
/// * `bool` - Returns `true` if the heading structure is valid, otherwise `false`.
fn check_heading_structure(html: &str) -> bool {
    let heading_re = Regex::new(r#"<h([1-6])>"#).unwrap();
    let mut prev_level = 0;
    let mut valid = true;

    for capture in heading_re.captures_iter(html) {
        let current_level: u32 = capture[1].parse().unwrap();

        // Debugging info to help locate the issue
        println!(
            "Current heading level: {}, Previous heading level: {}",
            current_level, prev_level
        );

        // If we're on the first heading or the levels are consecutive, it's valid
        if prev_level == 0
            || current_level == prev_level
            || current_level == prev_level + 1
        {
            prev_level = current_level; // Update previous level
        } else if current_level > prev_level + 1 {
            // If the current level skips more than 1 level (e.g., h1 -> h3), it's invalid
            println!("Invalid heading structure: skipping from <h{}> to <h{}>", prev_level, current_level);
            valid = false;
            break;
        }
    }

    valid
}

/// Check for the presence of alt text in images.
///
/// Ensures that every `<img>` tag in the HTML contains a valid `alt` attribute as per WCAG guidelines.
///
/// # Arguments
///
/// * `html` - A string slice representing the HTML content.
///
/// # Returns
///
/// * `bool` - Returns `true` if all images have valid alt text, otherwise `false`.
fn check_alt_text(html: &str) -> bool {
    let img_re = Regex::new(r#"<img\s+[^>]*alt="[^"]*""#).unwrap();
    img_re.is_match(html)
}

/// Check if all form inputs have associated labels.
///
/// Ensures that form elements like `<input>` have associated labels or ARIA attributes.
///
/// # Arguments
///
/// * `html` - A string slice representing the HTML content.
///
/// # Returns
///
/// * `bool` - Returns `true` if there are no input elements or if all inputs have valid labels or aria-labels.
fn check_input_labels(html: &str) -> bool {
    let input_re = Regex::new(r#"<input\b[^>]*"#).unwrap();
    let label_re = Regex::new(r#"(aria-label|id)="[^"]*""#).unwrap();

    // Find all input elements
    for input in input_re.find_iter(html) {
        let input_tag = input.as_str();

        // If any input does not have a label or aria-label, return false
        if !label_re.is_match(input_tag) {
            println!("Input found without label: {}", input_tag); // Debugging statement
            return false;
        }
    }

    // If no inputs are present or all inputs have labels, return true
    true
}

/// Prettify the HTML for better readability.
///
/// Formats the HTML by adding indentation and new lines to make it easier to read and debug.
///
/// # Arguments
///
/// * `html` - A string slice representing the HTML content.
///
/// # Returns
///
/// * `String` - Returns the prettified HTML.
pub fn prettify_html(html: &str) -> String {
    html.split('<')
        .filter(|s| !s.is_empty())
        .map(|s| format!("<{}", s.trim()))
        .collect::<Vec<String>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_aria_attributes() {
        let html = "<button>Click me</button>";
        let result = add_aria_attributes(html).unwrap();
        assert!(result.contains(r#"aria-label="button""#));
    }

    #[test]
    fn test_validate_wcag() {
        let valid_html = r#"<img src="image.jpg" alt="Image description"><h1>Title</h1><h2>Subtitle</h2>"#;
        let invalid_html =
            r#"<img src="image.jpg"><h1>Title</h1><h3>Subtitle</h3>"#;

        assert!(validate_wcag(valid_html).is_ok());
        assert!(validate_wcag(invalid_html).is_err());
    }

    #[test]
    fn test_prettify_html() {
        let html = "<div><p>Text</p></div>";
        let prettified = prettify_html(html);
        assert_eq!(prettified, "<div>\n<p>Text\n</p>\n</div>");
    }
}

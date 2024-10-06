use thiserror::Error;

/// Enum to represent various errors that can occur during HTML generation, processing, or optimization.
#[derive(Error, Debug)]
pub enum HtmlError {
    /// Error that occurs when a regular expression fails to compile.
    ///
    /// This variant contains the underlying error from the `regex` crate.
    #[error("Failed to compile regex: {0}")]
    RegexCompilationError(#[from] regex::Error),

    /// Error indicating failure in extracting front matter from the input content.
    ///
    /// This variant is used when there is an issue parsing the front matter of a document.
    /// The associated string provides details about the error.
    #[error("Failed to extract front matter: {0}")]
    FrontMatterExtractionError(String),

    /// Error indicating a failure in formatting an HTML header.
    ///
    /// This variant is used when the header cannot be formatted correctly. The associated string provides more details.
    #[error("Failed to format header: {0}")]
    HeaderFormattingError(String),

    /// Error for IO-related issues.
    ///
    /// This variant wraps standard IO errors and is used when an IO operation fails (e.g., reading or writing files).
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Error that occurs when parsing a selector fails.
    ///
    /// This variant is used when a CSS or HTML selector cannot be parsed.
    /// The first string is the selector, and the second string provides additional context.
    #[error("Failed to parse selector '{0}': {1}")]
    SelectorParseError(String, String),

    /// Error indicating failure to minify HTML content.
    ///
    /// This variant is used when there is an issue during the HTML minification process. The associated string provides details.
    #[error("Failed to minify HTML: {0}")]
    MinificationError(String),

    /// Error that occurs during the conversion of Markdown to HTML.
    ///
    /// This variant is used when the Markdown conversion process encounters an issue. The associated string provides more information.
    #[error("Markdown conversion error: {0}")]
    MarkdownConversionError(String),

    /// Error that occurs during SEO optimization.
    ///
    /// This variant is used when an SEO-related process fails, such as generating meta tags or structured data.
    /// The associated string provides more context.
    #[error("SEO optimization error: {0}")]
    SeoOptimizationError(String),

    /// Error that occurs when handling accessibility-related operations.
    ///
    /// This variant is used for errors that occur during accessibility checks or modifications (e.g., adding ARIA attributes).
    /// The associated string provides more details.
    #[error("Accessibility error: {0}")]
    AccessibilityError(String),

    // SEO module-specific errors
    /// Error indicating that a required HTML element is missing.
    ///
    /// This variant is used when a necessary HTML element (like a title tag) is not found.
    #[error("Missing required HTML element: {0}")]
    MissingHtmlElement(String),

    /// Error that occurs when structured data is invalid.
    ///
    /// This variant is used when JSON-LD or other structured data does not meet the expected format or requirements.
    #[error("Invalid structured data: {0}")]
    InvalidStructuredData(String),

    // Utils module-specific errors
    /// Error indicating an invalid front matter format.
    ///
    /// This variant is used when the front matter of a document does not follow the expected format.
    #[error("Invalid front matter format: {0}")]
    InvalidFrontMatterFormat(String),

    /// Error indicating an invalid header format.
    ///
    /// This variant is used when an HTML header does not conform to the expected format.
    #[error("Invalid header format: {0}")]
    InvalidHeaderFormat(String),

    /// Error that occurs when converting from UTF-8 fails.
    ///
    /// This variant wraps errors that occur when converting a byte sequence to a UTF-8 string.
    #[error("UTF-8 conversion error: {0}")]
    Utf8ConversionError(#[from] std::string::FromUtf8Error),

    // General purpose errors
    /// Error indicating a failure during parsing.
    ///
    /// This variant is used for general parsing errors where the specific source of the issue isn't covered by other variants.
    #[error("Parsing error: {0}")]
    ParsingError(String),

    /// Error indicating a validation failure.
    ///
    /// This variant is used when a validation step fails, such as schema validation or data integrity checks.
    #[error("Validation error: {0}")]
    ValidationError(String),

    /// A catch-all error for unexpected failures.
    ///
    /// This variant is used for errors that do not fit into other categories.
    #[error("Unexpected error: {0}")]
    UnexpectedError(String),
}

/// Type alias for a result using the `HtmlError` error type.
pub type Result<T> = std::result::Result<T, HtmlError>;

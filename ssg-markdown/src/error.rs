use thiserror::Error;

/// Errors that can occur during Markdown processing.
///
/// This enum defines various error types that might occur during different stages of the
/// Markdown processing pipeline, such as parsing, conversion, rendering, or using extensions.
#[derive(Error, Debug)]
pub enum MarkdownError {
    /// An error occurred while parsing the Markdown content.
    ///
    /// This variant contains a `String` that describes the specific parsing error.
    #[error("Failed to parse Markdown: {0}")]
    ParseError(String),

    /// An error occurred while converting Markdown to HTML.
    ///
    /// This variant contains a `String` that describes the conversion error.
    #[error("Failed to convert Markdown to HTML: {0}")]
    ConversionError(String),

    /// An error occurred while rendering HTML from the Markdown content.
    ///
    /// This variant contains a `String` that describes the rendering error.
    #[error("Failed to render HTML: {0}")]
    RenderError(String),

    /// An error occurred while processing a Markdown extension.
    ///
    /// This variant contains a `String` that describes the extension error.
    #[error("Extension error: {0}")]
    ExtensionError(String),
}

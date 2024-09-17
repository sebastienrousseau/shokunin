use thiserror::Error;

/// Represents errors that can occur during template processing.
#[derive(Error, Debug)]
pub enum TemplateError {
    /// Indicates an I/O error occurred.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Indicates an HTTP request error occurred.
    #[error("Request error: {0}")]
    Reqwest(#[from] reqwest::Error),

    /// Indicates an invalid template syntax was encountered.
    #[error("Invalid template syntax")]
    InvalidSyntax,

    /// Indicates a rendering error occurred.
    #[error("Rendering error: {0}")]
    RenderError(String),
}

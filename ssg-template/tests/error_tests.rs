use ssg_template::TemplateError;

/// Unit tests for TemplateError variants and their behavior.
#[cfg(test)]
mod template_error_tests {
    use super::*;
    use std::io;

    /// Test the `Io` variant of the `TemplateError` enum.
    /// This test checks if an I/O error is correctly wrapped inside a `TemplateError::Io`.
    #[test]
    fn test_template_error_io() {
        let io_error =
            io::Error::new(io::ErrorKind::NotFound, "file not found");
        let template_error = TemplateError::Io(io_error);
        assert!(matches!(template_error, TemplateError::Io(_)));
    }

    /// Test the `Reqwest` variant of the `TemplateError` enum.
    /// This test checks if an HTTP request error is correctly wrapped inside a `TemplateError::Reqwest`.
    #[test]
    fn test_template_error_reqwest() {
        let reqwest_error =
            reqwest::blocking::get("http://localhost:1").unwrap_err();
        let template_error = TemplateError::Reqwest(reqwest_error);
        assert!(matches!(template_error, TemplateError::Reqwest(_)));
    }

    /// Test the `InvalidSyntax` variant of the `TemplateError` enum.
    /// This test checks if the `InvalidSyntax` error is correctly represented.
    #[test]
    fn test_template_error_invalid_syntax() {
        let template_error = TemplateError::InvalidSyntax;
        assert!(matches!(template_error, TemplateError::InvalidSyntax));
    }

    /// Test the `RenderError` variant of the `TemplateError` enum.
    /// This test checks if a rendering error is correctly wrapped inside a `TemplateError::RenderError`.
    #[test]
    fn test_template_error_render_error() {
        let template_error =
            TemplateError::RenderError("Failed to render".to_string());
        assert!(matches!(
            template_error,
            TemplateError::RenderError(_)
        ));
    }

    /// Test the `Display` implementation for the `TemplateError::Io` variant.
    /// This test checks if the display output for an I/O error is formatted correctly.
    #[test]
    fn test_template_error_io_display() {
        let io_error =
            io::Error::new(io::ErrorKind::NotFound, "file not found");
        let template_error = TemplateError::Io(io_error);
        assert_eq!(
            format!("{}", template_error),
            "I/O error: file not found"
        );
    }

    /// Test the `Display` implementation for the `TemplateError::Reqwest` variant.
    /// This test checks if the display output for a Reqwest error is formatted correctly.
    #[test]
    fn test_template_error_reqwest_display() {
        let reqwest_error =
            reqwest::blocking::get("http://localhost:1").unwrap_err();
        let template_error = TemplateError::Reqwest(reqwest_error);
        assert!(
            format!("{}", template_error).starts_with("Request error:")
        );
    }
}

/// Additional tests for edge cases and error chaining related to the TemplateError enum.
#[cfg(test)]
mod additional_template_error_tests {
    use super::*;
    use std::io;

    /// Test chaining of IO errors using the `#[from]` attribute.
    /// This ensures that I/O errors are correctly converted into `TemplateError::Io`.
    #[test]
    fn test_template_error_io_chaining() {
        let io_error: io::Error = io::ErrorKind::NotFound.into();
        let template_error = TemplateError::from(io_error);
        assert!(matches!(template_error, TemplateError::Io(_)));
    }

    /// Test chaining of Reqwest errors using the `#[from]` attribute.
    /// This ensures that Reqwest errors are correctly converted into `TemplateError::Reqwest`.
    #[test]
    fn test_template_error_reqwest_chaining() {
        let reqwest_error =
            reqwest::blocking::get("http://localhost:1").unwrap_err();
        let template_error = TemplateError::from(reqwest_error);
        assert!(matches!(template_error, TemplateError::Reqwest(_)));
    }

    /// Test custom error message for the `RenderError` variant.
    /// This ensures that custom messages are preserved and displayed correctly.
    #[test]
    fn test_render_error_custom_message() {
        let custom_message = "Custom render error message".to_string();
        let template_error =
            TemplateError::RenderError(custom_message.clone());
        assert!(matches!(
            template_error,
            TemplateError::RenderError(_)
        ));
        assert_eq!(
            format!("{}", template_error),
            format!("Rendering error: {}", custom_message)
        );
    }

    /// Test an unreachable case for `TemplateError`.
    /// This is hypothetical, ensuring no undefined error variants are being used.
    #[test]
    fn test_template_error_unreachable() {
        let result: Result<(), TemplateError> =
            Err(TemplateError::InvalidSyntax);
        assert!(matches!(result, Err(TemplateError::InvalidSyntax)));
    }

    /// Test conversion consistency between different types of errors.
    /// This ensures that both I/O and Reqwest errors are correctly handled by `TemplateError`.
    #[test]
    fn test_template_error_conversion_consistency() {
        let io_error: io::Error =
            io::ErrorKind::PermissionDenied.into();
        let reqwest_error =
            reqwest::blocking::get("http://localhost:1").unwrap_err();

        let io_template_error = TemplateError::from(io_error);
        let reqwest_template_error = TemplateError::from(reqwest_error);

        assert!(matches!(io_template_error, TemplateError::Io(_)));
        assert!(matches!(
            reqwest_template_error,
            TemplateError::Reqwest(_)
        ));
    }
}

#[cfg(test)]
mod tests {

    fn test_macro_log(
        level: LogLevel,
        component: &str,
        description: &str,
        format: LogFormat,
    ) {
        let log =
            macro_log_info!(level, component, description, format);
        assert_eq!(log.level, level);
        assert_eq!(log.component, component);
        assert_eq!(log.description, description);
        assert_eq!(log.format, format);
    }

    #[test]
    fn test_macros() {
        test_macro_log(
            LogLevel::ALL,
            "component",
            "description",
            LogFormat::CLF,
        );
        test_macro_log(
            LogLevel::DEBUG,
            "component",
            "description",
            LogFormat::CLF,
        );
        test_macro_log(
            LogLevel::DISABLED,
            "component",
            "description",
            LogFormat::CLF,
        );
        test_macro_log(
            LogLevel::ERROR,
            "component",
            "description",
            LogFormat::CLF,
        );
        test_macro_log(
            LogLevel::FATAL,
            "component",
            "description",
            LogFormat::CLF,
        );
        test_macro_log(
            LogLevel::INFO,
            "component",
            "description",
            LogFormat::CLF,
        );
        test_macro_log(
            LogLevel::NONE,
            "component",
            "description",
            LogFormat::CLF,
        );
        test_macro_log(
            LogLevel::TRACE,
            "component",
            "description",
            LogFormat::CLF,
        );
        test_macro_log(
            LogLevel::VERBOSE,
            "component",
            "description",
            LogFormat::CLF,
        );
        test_macro_log(
            LogLevel::WARNING,
            "component",
            "description",
            LogFormat::CLF,
        );
    }

    use ssg::{
        loggers::{Log, LogFormat, LogLevel},
        macro_log_info,
    };

    #[test]
    fn test_log_level_display() {
        let level = LogLevel::INFO;
        assert_eq!(format!("{level}"), "INFO");
    }

    #[test]
    fn test_log_format_display() {
        let format = LogFormat::JSON;
        assert_eq!(format!("{format}"), "JSON\n");
    }

    #[test]
    fn test_log_new() {
        let log = Log::new(
            "session123",
            "2023-02-28T12:34:56Z",
            LogLevel::WARNING,
            "auth",
            "Invalid credentials",
            LogFormat::CLF,
        );

        assert_eq!(log.session_id, "session123");
        assert_eq!(log.time, "2023-02-28T12:34:56Z");
        assert_eq!(log.level, LogLevel::WARNING);
        assert_eq!(log.component, "auth");
        assert_eq!(log.description, "Invalid credentials");
        assert_eq!(log.format, LogFormat::CLF);
    }

    #[test]
    fn test_log_default() {
        let log = Log::default();

        assert!(log.session_id.is_empty());
        assert!(log.time.is_empty());
        assert_eq!(log.level, LogLevel::INFO);
        assert!(log.component.is_empty());
        assert!(log.description.is_empty());
        assert_eq!(log.format, LogFormat::CLF);
    }
}
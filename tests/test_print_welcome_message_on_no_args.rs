#[cfg(test)]
mod tests {
    use ssg::print_welcome_message_on_no_args;

    #[test]
    fn test_print_welcome_message_on_no_args() {
        let result = print_welcome_message_on_no_args();
        assert!(result.is_ok())
    }
}

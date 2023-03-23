#[cfg(test)]
mod tests {
    use ssg::print_welcome_message;
    use std::error::Error;

    #[test]
    fn test_print_welcome_message() -> Result<(), Box<dyn Error>> {
        let title = "Test Title";
        let description = "Test Description";
        print_welcome_message(title, description)?;

        Ok(())
    }

    #[test]
    fn test_print_welcome_message_longer_title(
    ) -> Result<(), Box<dyn Error>> {
        let title =
            "A Very Long Title That Will Exceed The Description Length";
        let description = "Short Description";
        print_welcome_message(title, description)?;

        Ok(())
    }

    #[test]
    fn test_print_welcome_message_longer_description(
    ) -> Result<(), Box<dyn Error>> {
        let title = "Short Title";
        let description =
            "A Very Long Description That Will Exceed The Title Length";
        print_welcome_message(title, description)?;

        Ok(())
    }
}

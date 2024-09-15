use comrak::ComrakOptions;
use ssg_markdown::process_markdown;

#[test]
fn test_basic_markdown_conversion() {
    let markdown = "# Hello, world!";
    let options = ComrakOptions::default();
    let result = process_markdown(markdown, &options).unwrap();
    assert_eq!(result.trim(), "<h1>Hello, world!</h1>");
}

#[test]
fn test_markdown_with_extensions() {
    let markdown = "This is a ~~strikethrough~~ test.";
    let mut options = ComrakOptions::default();
    options.extension.strikethrough = true;
    let result = process_markdown(markdown, &options).unwrap();
    assert_eq!(
        result.trim(),
        "<p>This is a <del>strikethrough</del> test.</p>"
    );
}

#[test]
fn test_markdown_with_links() {
    let markdown = "[OpenAI](https://openai.com)";
    let options = ComrakOptions::default();
    let result = process_markdown(markdown, &options).unwrap();
    assert_eq!(
        result.trim(),
        r#"<p><a href="https://openai.com">OpenAI</a></p>"#
    );
}

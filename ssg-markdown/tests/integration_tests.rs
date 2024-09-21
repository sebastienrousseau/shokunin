use ssg_markdown::{process_markdown, MarkdownOptions};

#[test]
fn test_basic_markdown_conversion() {
    let markdown = "# Hello, world!";
    let options = MarkdownOptions::new()
        .with_enhanced_tables(false) // Disable enhanced tables for this test
        .with_comrak_options({
            let mut opts = comrak::ComrakOptions::default();
            opts.extension.table = false; // Ensure the table extension is disabled
            opts
        });

    let result = process_markdown(markdown, &options).unwrap();
    assert_eq!(result.trim(), "<h1>Hello, world!</h1>");
}

#[test]
fn test_markdown_with_extensions() {
    let markdown = "This is a ~~strikethrough~~ test.";
    let options = MarkdownOptions::new()
        .with_enhanced_tables(false) // Disable enhanced tables for this test
        .with_comrak_options({
            let mut opts = comrak::ComrakOptions::default();
            opts.extension.strikethrough = true; // Enable strikethrough
            opts.extension.table = false; // Disable the table extension
            opts
        });

    let result = process_markdown(markdown, &options).unwrap();
    assert_eq!(
        result.trim(),
        "<p>This is a <del>strikethrough</del> test.</p>"
    );
}

#[test]
fn test_markdown_with_links() {
    let markdown =
        "[Shokunin Static Site Generator (SSG)](https://shokunin.one/)";
    let options = MarkdownOptions::new()
        .with_enhanced_tables(false) // Disable enhanced tables for this test
        .with_comrak_options({
            let mut opts = comrak::ComrakOptions::default();
            opts.extension.table = false; // Disable the table extension
            opts
        });

    let result = process_markdown(markdown, &options).unwrap();
    assert_eq!(
        result.trim(),
        r#"<p><a href="https://shokunin.one/">Shokunin Static Site Generator (SSG)</a></p>"#
    );
}

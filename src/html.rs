/// Generates HTML meta tags for the given metadata key-value pairs. Returns
/// a string containing the HTML code for the meta tags. Each meta tag is
/// created using the `name` and `content` attributes of the input metadata,
/// with the `name` attribute corresponding to the key and the `content`
/// attribute corresponding to the value. The resulting tags are concatenated
/// into a single string, separated by newline characters.
///
/// # Arguments
///
/// * `meta` - A slice of key-value pairs representing the metadata to be used
///            in the generated meta tags. Each key-value pair is represented
///            as a tuple of `String` objects, with the first element
///            representing the `name` attribute and the second element
///            representing the `content` attribute of the meta tag.
///
/// # Example
///
/// ```
///
/// use ssg::html::generate_meta_tags;
///
/// let meta = vec![
///     ("description".to_owned(), "My awesome website".to_owned()),
///     ("keywords".to_owned(), "rust, programming, web development".to_owned()),
/// ];
///
/// let result = generate_meta_tags(&meta);
/// assert_eq!(result, "<meta name=\"description\" content=\"My awesome website\">\n<meta name=\"keywords\" content=\"rust, programming, web development\">");
///
/// ```
pub fn generate_meta_tags(meta: &[(String, String)]) -> String {
    meta.iter()
        .map(|(key, value)| {
            format!("<meta name=\"{}\" content=\"{}\">", key, value)
        })
        .collect::<Vec<_>>()
        .join("\n")
}
/// Generates an HTML page from the given Markdown content, title, and
/// description.
///
/// This function takes in a Markdown string `content`, as well as a
/// `title` and `description` string to use for the HTML page. The
/// function converts the Markdown content to HTML using the Comrak
/// library, and adds a header and subheader to the HTML page using the
/// title and description strings, respectively.
///
/// If `content` begins with front matter (denoted by "---\n"), the
/// front matter is skipped and only the Markdown content below it is
/// used to generate the HTML. If `title` or `description` are empty
/// strings, they are not included in the generated HTML page.
///
/// The resulting HTML page is returned as a string.
///
/// # Examples
///
/// ```
///
/// use ssg::html::generate_html;
///
/// let content = "## Hello, world!\n\nThis is a test.";
/// let title = "My Page";
/// let description = "This is a test page";
/// let html = generate_html(content, title, description);
/// assert_eq!(
///     html,
///     "<h1>My Page</h1><h2>This is a test page</h2><h2>Hello, world!</h2>\n<p>This is a test.</p>\n"
/// );
///
/// ```
pub fn generate_html(
    content: &str,
    title: &str,
    description: &str,
) -> String {
    let options = comrak::ComrakOptions::default();
    let markdown_content = if content.starts_with("---\n") {
        if let Some(end_pos) = content.find("\n---\n") {
            &content[end_pos + 5..] // Skip the "---\n\n" that follows the front matter
        } else {
            ""
        }
    } else {
        content
    };
    let header = if !title.is_empty() {
        format!("<h1>{}</h1>", title)
    } else {
        "".to_string()
    };
    let subheader = if !description.is_empty() {
        format!("<h2>{}</h2>", description)
    } else {
        "".to_string()
    };
    let markdown_html =
        comrak::markdown_to_html(markdown_content, &options);
    format!("{}{}{}", header, subheader, markdown_html)
}

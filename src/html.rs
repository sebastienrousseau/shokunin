use comrak;

pub(crate) fn generate_meta_tags(meta: &[(String, String)]) -> String {
    meta.iter()
        .map(|(key, value)| {
            format!("<meta name=\"{}\" content=\"{}\">", key, value)
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub(crate) fn generate_html(
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

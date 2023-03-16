/// Extracts metadata from the front matter of a Markdown file and
/// returns it as a tuple. The front matter is defined as any YAML block
/// that appears at the beginning of the file, enclosed by "---" lines.
/// The function expects the entire content of the file as a single
/// string.
///
/// The metadata extracted by the function includes the title,
/// description, keywords, and permalink of the page, if they are
/// present in the front matter. If any of these fields are not present,
/// an empty string is returned for that field in the tuple.
///
/// The function uses a simple parsing approach to extract the metadata
/// from the front matter.
///
/// It splits the front matter into lines, then looks for lines
/// containing a colon (":"). If a line containing a colon is found, the
/// text before the colon is treated as the key and the text after the
/// colon is treated as the value. The key-value pairs are then stored
/// in local variables according to the type of metadata being
/// extracted.
///
/// If no front matter is present in the input string, or if an error
/// occurs during parsing, an empty string is returned for all fields
/// in the tuple.
pub fn extract_front_matter(
    content: &str,
) -> (String, String, String, String) {
    let mut title = String::new();
    let mut description = String::new();
    let mut keywords = String::new();
    let mut permalink = String::new();

    if content.starts_with("---\n") {
        if let Some(end_pos) = content.find("\n---\n") {
            let front_matter = &content[..end_pos];
            for line in front_matter.lines() {
                if let Some(pos) = line.find(':') {
                    let key = line[..pos].trim();
                    let value = line[pos + 1..].trim();
                    match key {
                        "title" => title = value.to_owned(),
                        "description" => description = value.to_owned(),
                        "keywords" => keywords = value.to_owned(),
                        "permalink" => permalink = value.to_owned(),
                        _ => (),
                    }
                }
            }
        }
    }
    (title, description, keywords, permalink)
}

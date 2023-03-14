pub(crate) fn extract_front_matter(content: &str) -> (String, String, String, String) {
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

use serde_json::Value as JsonValue;
use std::collections::HashMap;
use toml::Value as TomlValue;
use yaml_rust::YamlLoader;

/// ## Function: extract - Extracts metadata from the front matter of a Markdown file
///
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
///
/// # Arguments
///
/// * `content` - A reference to a string containing the entire content
///              of the Markdown file.
/// # Returns
///
/// * A tuple containing the title, description, keywords, permalink and
///   layout of the page, if they are present in the front matter.
///   If any of these fields are not present, an empty string is
///   returned for that field in the tuple.
///
/// # Example
///
/// ```
///use ssg::frontmatter::extract;
///
///    let content = "---\n\
///        title: My Page\n\
///        date: 2000-01-01\n\
///        description: A page about something\n\
///        keywords: something, cool, interesting\n\
///        permalink: /my-page/\n\
///        layout: page\n\
///        ---\n\
///        # My Page\n\
///        This is my page about something. It's really cool and interesting!";
///
///    let result = extract(&content);
///    assert_eq!(result["title"], "My Page");
///    assert_eq!(result["date"], "2000-01-01");
///    assert_eq!(result["description"], "A page about something");
///    assert_eq!(result["keywords"], "something, cool, interesting");
///    assert_eq!(result["permalink"], "/my-page/");
///    assert_eq!(result["layout"], "page");
///
/// ```
///

/// Extracts metadata from the front matter of a Markdown file and
/// returns it as a tuple. The front matter is defined as any YAML block
/// that appears at the beginning of the file, enclosed by "---" lines.
///

pub fn extract(content: &str) -> HashMap<String, String> {
    let mut front_matter = HashMap::new();

    if content.starts_with("---\n") {
        if let Some(end_pos) = content.find("\n---\n") {
            let front_matter_str = &content[4..end_pos];
            let docs =
                YamlLoader::load_from_str(front_matter_str).unwrap();
            let doc = &docs[0];
            for (key, value) in doc.as_hash().unwrap().iter() {
                front_matter.insert(
                    key.as_str().unwrap().to_string(),
                    value.as_str().unwrap().to_string(),
                );
            }
        }
    } else if content.starts_with("+++\n") {
        if let Some(end_pos) = content.find("\n+++\n") {
            let front_matter_str = &content[4..end_pos];
            let toml_value: TomlValue =
                front_matter_str.parse().unwrap();
            for (key, value) in toml_value.as_table().unwrap().iter() {
                front_matter.insert(
                    key.to_string(),
                    value.as_str().unwrap().to_string(),
                );
            }
        }
    } else if content.starts_with('{') {
        let end_pos = content.rfind('}').unwrap();
        let front_matter_str = &content[0..=end_pos];

        let json_value: serde_json::Result<JsonValue> =
            serde_json::from_str(front_matter_str);
        match json_value {
            Ok(value) => {
                let front_matter_obj = value.get("frontmatter");
                match front_matter_obj {
                    Some(obj) => {
                        for (key, value) in
                            obj.as_object().unwrap().iter()
                        {
                            front_matter.insert(
                                key.to_string(),
                                value.as_str().unwrap().to_string(),
                            );
                        }
                    }
                    None => {
                        eprintln!(
                            "Error: Could not find frontmatter in JSON"
                        );
                    }
                }
                if let Some(content) = value.get("content") {
                    front_matter.insert(
                        "content".to_string(),
                        content.as_str().unwrap_or("").to_string(),
                    );
                }
            }
            Err(err) => {
                eprintln!("Error parsing JSON: {:?}", err);
            }
        }
    }
    front_matter
}

// Copyright © 2024 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

use regex::Regex;
use std::{
    error::Error,
    fs::{self},
    io,
    path::{Path, PathBuf},
};
/// Ensures a directory exists, creating it if necessary.
///
/// This function takes a reference to a `Path` object for a directory and a human-readable name for the directory, and creates the directory if it does not already exist.
///
/// # Arguments
///
/// * `dir` - A reference to a `Path` object for the directory.
/// * `name` - A human-readable name for the directory, used in error messages.
///
/// # Returns
///
/// * `Result<(), String>` - A result indicating success or failure.
///     - `Ok(())` if the directory exists or was created successfully.
///     - `Err(String)` if the directory does not exist and could not be created.
///
/// # Example
///
/// ```
/// use ssg_core::utilities::directory::directory;
/// use std::path::Path;
/// use std::fs;
///
/// // Create a "logs" directory if it doesn't exist
/// let dir = Path::new("logs");
/// directory(dir, "logs").expect("Could not create logs directory");
/// fs::remove_dir_all(dir).expect("Could not remove logs directory");
/// ```
///
pub fn directory(dir: &Path, name: &str) -> Result<String, String> {
    if dir.exists() {
        if !dir.is_dir() {
            return Err(format!(
                "❌ Error: {} is not a directory.",
                name
            ));
        }
    } else {
        match fs::create_dir_all(dir) {
            Ok(_) => {}
            Err(e) => {
                return Err(format!(
                    "❌ Error: Cannot create {} directory: {}",
                    name, e
                ))
            }
        }
    }
    Ok(String::new())
}

/// Moves the output directory to the public directory.
///
/// This function takes a reference to a `Path` object for the output directory and a string for the site name, and moves the output directory to the public directory.
///
/// # Arguments
///
/// * `site_name` - A string for the site name.
/// * `out_dir` - A reference to a `Path` object for the output directory.
///
/// # Returns
///
/// * `Result<(), std::io::Error>` - A result indicating success or failure.
///     - `Ok(())` if the output directory was moved successfully.
///     - `Err(std::io::Error)` if the output directory could not be moved.
///
pub fn move_output_directory(
    site_name: &str,
    out_dir: &Path,
) -> io::Result<()> {
    println!("❯ Moving output directory...");

    let public_dir = Path::new("public");

    if public_dir.exists() {
        fs::remove_dir_all(public_dir)?;
    }

    fs::create_dir(public_dir)?;

    let site_name = site_name.replace(' ', "_");
    let new_project_dir = public_dir.join(site_name);
    fs::create_dir_all(&new_project_dir)?;

    fs::rename(out_dir, &new_project_dir)?;

    println!("  Done.\n");

    Ok(())
}

/// Finds all HTML files in a directory.
///
/// This function takes a reference to a `Path` object for a directory and returns a vector of `PathBuf` objects for all HTML files in the directory and its subdirectories.
///
/// # Arguments
///
/// * `dir` - A reference to a `Path` object for the directory.
///
/// # Returns
///
/// * `Result<Vec<PathBuf>, std::io::Error>` - A result containing a vector of
///    `PathBuf` objects for all HTML files in the directory and its
///     subdirectories.
///     - `Ok(Vec<PathBuf>)` if the directory exists and contains HTML files.
///     - `Err(std::io::Error)` if the directory does not exist or does not
///        contain HTML files.
///
pub fn find_html_files(dir: &Path) -> io::Result<Vec<PathBuf>> {
    let mut html_files = Vec::new();

    for entry in fs::read_dir(dir)? {
        let entry = entry?;

        if entry.path().is_dir() {
            let sub_html_files = find_html_files(&entry.path())?;
            html_files.extend(sub_html_files);
        } else if let Some(extension) =
            entry.path().extension().and_then(|ext| ext.to_str())
        {
            if extension.eq_ignore_ascii_case("html") {
                html_files.push(entry.path());
            }
        }
    }

    Ok(html_files)
}

/// Cleans up the directory at the given path.
///
/// If the directory does not exist, this function does nothing.
///
/// # Arguments
///
/// * `directories` - An array of references to `Path` objects representing the
///    directories to be cleaned up.
///
/// # Returns
///
/// * `Result<(), Box<dyn Error>>` - A result indicating success or failure.
///     - `Ok(())` if the directories were cleaned up successfully.
///     - `Err(Box<dyn Error>)` if an error occurred during the cleanup process.
///
pub fn cleanup_directory(
    directories: &[&Path],
) -> Result<(), Box<dyn Error>> {
    for directory in directories {
        if !directory.exists() {
            continue;
        }

        println!("\n❯ Cleaning up directories");

        fs::remove_dir_all(directory)?;

        println!("  Done.\n");
    }

    Ok(())
}

/// Creates a new directory at the given path.
///
/// If the directory already exists, this function does nothing.
///
/// # Arguments
///
/// * `directories` - An array of references to `Path` objects representing the
///    directories to be created.
///
/// # Returns
///
/// * `Result<(), Box<dyn Error>>` - A result indicating success or failure.
///     - `Ok(())` if the directories were created successfully.
///     - `Err(Box<dyn Error>)` if an error occurred during the creation process.
///
pub fn create_directory(
    directories: &[&Path],
) -> Result<(), Box<dyn Error>> {
    for directory in directories {
        if directory.exists() {
            continue;
        }

        fs::create_dir(directory)?;
    }

    Ok(())
}

/// Converts a string to title case.
///
/// This function takes a reference to a string and returns a new string with
/// the first letter of each word capitalized.
///
/// # Arguments
///
/// * `s` - A reference to a string.
///
/// # Returns
///
/// * `String` - A string with the first letter of each word capitalized.
///
pub fn to_title_case(s: &str) -> String {
    let re = Regex::new(r"(\b)(\w)(\w*)").unwrap();
    let result = re.replace_all(s, |caps: &regex::Captures<'_>| {
        format!("{}{}{}", &caps[1], caps[2].to_uppercase(), &caps[3])
    });
    result.to_string()
}

/// Formats a header string with an ID and class attribute.
///
/// This function takes a reference to a string containing the header and a reference to a `Regex` object
///
/// # Arguments
///
/// * `header_str` - A reference to a string containing the header.
/// * `id_regex` - A reference to a `Regex` object.
///
/// # Returns
///
/// * `String` - A string containing the formatted header.
///
///
pub fn format_header_with_id_class(
    header_str: &str,
    id_regex: &Regex,
) -> String {
    let mut formatted_header_str = String::new();
    let mut in_header_tag = false;
    let mut id_attribute_added = false;
    let mut class_attribute_added = false;

    let re = Regex::new(r"</?[^>]+>").unwrap();
    let text_only = re.replace_all(header_str, "");
    let re = Regex::new(r"^<(\w+)").unwrap();
    let captures = re.captures(header_str);
    let header_type = captures
        .map_or("", |cap| cap.get(1).map_or("", |m| m.as_str()))
        .to_lowercase();
    let first_word = text_only
        .split(|c: char| !c.is_alphanumeric())
        .next()
        .unwrap_or("")
        .to_lowercase();
    let aria_label = to_title_case(first_word.as_str());
    for c in header_str.chars() {
        if !in_header_tag {
            formatted_header_str.push(c);
            if c == '<' {
                in_header_tag = true;
            }
        } else {
            if !id_attribute_added && (c == ' ' || c == '>') {
                formatted_header_str.push_str(&format!(
                    " id=\"{}-{}\" tabindex=\"0\" aria-label=\"{} Heading\" {}",
                    header_type,
                    first_word,
                    id_regex.replace_all(&aria_label, "-"),
                    if header_type == "h1" {
                        "itemprop=\"headline\""
                    } else {
                        "itemprop=\"name\""
                    }
                ));

                // print!("header_type={:?}", header_type);
                id_attribute_added = true;
            }

            if !class_attribute_added && c == '>' {
                let class_value = format!(
                    "{}",
                    id_regex.replace_all(&first_word, "-")
                );
                formatted_header_str
                    .push_str(&format!(" class=\"{}\"", class_value));
                class_attribute_added = true;
            }
            formatted_header_str.push(c);
            if c == '>' {
                in_header_tag = false;
            }
        }
    }

    formatted_header_str
}

/// Extracts the front matter from the given content.
///
/// This function takes a reference to a string containing the content of a page and returns a string containing the content without the front matter.
///
/// # Arguments
///
/// * `content` - A reference to a string containing the content of a page.
///
/// # Returns
///
/// * `&str` - A string containing the content without the front matter.
///  - If the content does not contain front matter, the original content is returned.
/// - If the content contains front matter, the content without the front matter is returned.
/// - If the content contains front matter but the front matter is invalid, an empty string is returned, regardless of whether the content contains additional content or not.
pub fn extract_front_matter(content: &str) -> &str {
    if content.starts_with("---\n") {
        if let Some(end_pos) = content.find("\n---\n") {
            &content[end_pos + 5..]
        } else {
            ""
        }
    } else if content.starts_with("+++\n") {
        if let Some(end_pos) = content.find("\n+++\n") {
            &content[end_pos + 5..]
        } else {
            ""
        }
    } else if content.starts_with("{\n") {
        if let Some(end_pos) = content.find("\n}\n") {
            &content[end_pos + 2..]
        } else {
            ""
        }
    } else {
        content
    }
}

/// Creates and returns an instance of `comrak::ComrakOptions` with non- standard Markdown features enabled. This allows for a greater variety of elements in the markdown syntax.
///
/// # Returns
///
/// * `comrak::ComrakOptions` - A customized ComrakOptions object with specific features enabled.
///
/// # Features
///
/// * `Autolink`: Detects URLs and email addresses and makes them clickable.
/// * `Description Lists`: Allows the creation of description lists.
/// * `Footnotes`: Allows the creation of footnotes.
/// * `Front Matter Delimiter`: Ignored front matter starting with '---'
/// * `Header IDs`: Adds an ID to each header.
/// * `Strikethrough`: Allows the creation of strikethrough text.
/// * `Superscript`: Allows the creation of superscript text.
/// * `Table`: Allows the creation of tables.
/// * `Tag Filter`: Allows HTML tags to be filtered.
/// * `Task List`: Allows the creation of task lists.
/// * `Smart`: Enables smart punctuation.
/// * `GitHub Pre Lang`: Renders GitHub-style fenced code blocks.
/// * `Hardbreaks`: Renders hard line breaks as <br> tags.
/// * `Unsafe`: Allows raw HTML to be rendered.
///
///
/// Initializes and returns the default Comrak options, enabling several
/// non-standard Markdown features.
///
/// # Returns
///
/// * `ComrakOptions` - A `comrak::ComrakOptions` instance with non-standard Markdown features enabled.
pub fn create_comrak_options() -> comrak::ComrakOptions<'static> {
    let mut options = comrak::ComrakOptions::default();

    // Enable non-standard Markdown features:

    // Enables auto-linking of URLs and email addresses, making them clickable in the output.
    options.extension.autolink = true;

    // Enables creation of description lists in the Markdown source.
    options.extension.description_lists = true;

    // Enables creation of footnotes in the Markdown source.
    options.extension.footnotes = true;

    // Sets the front matter delimiter to '---', thereby ignoring any front matter in the Markdown source.
    options.extension.front_matter_delimiter = Some("---".to_owned());

    // Enables automatic generation of ID attributes for each header in the Markdown source.
    // options.extension.header_ids = Some("".to_string());

    // Enables creation of strikethrough text in the Markdown source.
    options.extension.strikethrough = true;

    // Enables creation of superscript text in the Markdown source.
    options.extension.superscript = true;

    // Enables creation of tables in the Markdown source.
    options.extension.table = true;

    // Enables HTML tag filtering in the Markdown source.
    options.extension.tagfilter = true;

    // Enables creation of task lists in the Markdown source.
    options.extension.tasklist = true;

    // Enables smart punctuation, transforming straight quotes into curly quotes, -- into en-dashes, etc.
    options.parse.smart = true;

    // Enables GitHub-style fenced code blocks in the Markdown source.
    options.render.github_pre_lang = true;

    // Disables transformation of hard line breaks to <br> tags in the HTML output.
    options.render.hardbreaks = false;

    // Allows raw HTML to be included in the output.
    options.render.unsafe_ = true;

    options
}

/// Updates the 'class' attributes within the provided HTML line.
/// If `.class=&quot;` is found within the line, it extracts the class value using `class_regex` and replaces it in the correct place using `img_regex`.
///
/// # Arguments
///
/// * `line` - A reference to a string containing an HTML line that potentially contains class attributes.
///
/// * `class_regex` - A reference to a `Regex` object used to extract the class attribute value.
///
/// * `img_regex` - A reference to a `Regex` object used to identify where the class attribute should be replaced.
///
/// # Returns
///
/// * `String` - The updated HTML line.
///  - If the line contains `.class=&quot;`, the returned string will be the updated line where the class attribute value is correctly placed.
///  - If the line does not contain `.class=&quot;`, the original line is returned.
///
pub fn update_class_attributes(
    line: &str,
    class_regex: &Regex,
    img_regex: &Regex,
) -> String {
    if line.contains(".class=&quot;") && line.contains("<img") {
        let captures = class_regex.captures(line).unwrap();
        let class_value = captures.get(1).unwrap().as_str();
        let updated_line = class_regex.replace(line, "");
        let updated_line_with_class = img_regex.replace(
            &updated_line,
            &format!("$1 class=\"{}\" />", class_value),
        );
        return updated_line_with_class.into_owned();
    }
    line.to_owned()
}

/// Truncates a path to only have a set number of path components.
///
/// Will truncate a path to only show the last `length` components in a path.
/// If a length of `0` is provided, the path will not be truncated.
/// A value will only be returned if the path has been truncated.
///
/// # Arguments
///
/// * `path` - The path to truncate.
/// * `length` - The number of path components to keep.
///
/// # Returns
///
/// * An `Option` of the truncated path as a string. If the path was not truncated, `None` is returned.
pub fn truncate(path: &Path, length: usize) -> Option<String> {
    // Checks if the length is 0. If it is, returns `None`.
    if length == 0 {
        return None;
    }

    // Creates a new PathBuf object to store the truncated path.
    let mut truncated = PathBuf::new();

    // Iterates over the components of the path in reverse order.
    let mut count = 0;
    while let Some(component) = path.components().next_back() {
        // Adds the component to the truncated path.
        truncated.push(component);
        count += 1;

        // If the count reaches the desired length, breaks out of the loop.
        if count == length {
            break;
        }
    }

    // If the count is equal to the desired length, returns the truncated path as a string.
    if count == length {
        Some(truncated.to_string_lossy().to_string())
    } else {
        // Otherwise, returns `None`.
        None
    }
}

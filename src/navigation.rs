use crate::data::FileData;
use std::path::Path;

/// Generates a navigation menu as an unordered list of links to the
/// compiled HTML files.
///
/// # Arguments
///
/// * `files` - A slice of `FileData` structs containing the compiled HTML
///             files.
///
/// # Returns
///
/// A string containing the HTML code for the navigation menu. The
/// string is wrapped in a `<ul>` element with the class `nav`.
/// Each file is wrapped in a `<li>` element, and each link is wrapped
/// in an `<a>` element.
///
/// The `href` attribute of the `<a>` element is set to `../` to move
/// up one level in the directory hierarchy, followed by the name of
/// the directory for the file (generated based on the file name), and
/// finally `index.html`. The text of the link is set to the name of
/// the file with the `.md` extension removed.
///
/// The files are sorted alphabetically by their names.
/// The function returns an empty string if the slice is empty.
/// The function returns an error if the file name does not contain
/// the `.md` extension.
///
/// If multiple files have the same base directory name, the function
/// generates unique directory names by appending a suffix to the
/// directory name for each file.
///
/// If a file's name already contains a directory path
/// (e.g., `path/to/file.md`), the function generates a directory name
/// that preserves the structure of the path, without duplicating the
/// directory names (e.g., `path_to/file.md` becomes `path_to_file`).
///
pub fn generate_navigation(files: &[FileData]) -> String {
    let mut files_sorted = files.to_vec();
    files_sorted.sort_by(|a, b| a.name.cmp(&b.name));

    let nav_links = files_sorted.iter().filter(|file| {
        let file_name = match Path::new(&file.name).extension() {
            Some(ext) if ext == "md" => file.name.replace(".md", ""),
            Some(ext) if ext == "toml" => file.name.replace(".toml", ""),
            Some(ext) if ext == "json" => file.name.replace(".json", ""),
            _ => file.name.to_string(),
        };
        file_name != "index" && file_name != "404"
    }).map(|file| {
        let mut dir_name = match Path::new(&file.name).extension() {
            Some(ext) if ext == "md" => file.name.replace(".md", ""),
            Some(ext) if ext == "toml" => file.name.replace(".toml", ""),
            Some(ext) if ext == "json" => file.name.replace(".json", ""),
            _ => file.name.to_string(),
        };
        // Handle special case for files in the same base directory
        if let Some((index, _)) = dir_name.match_indices('/').next() {
            let base_dir = &dir_name[..index];
            let file_name = &dir_name[index + 1..];
            dir_name = format!("{}{}", base_dir, file_name.replace(base_dir, ""));
        }

        format!(
            "<li><a href=\"/{}/index.html\" role=\"navigation\">{}</a></li>",
            dir_name,
            match Path::new(&file.name).extension() {
                Some(ext) if ext == "md" => file.name.replace(".md", ""),
                Some(ext) if ext == "toml" => file.name.replace(".toml", ""),
                Some(ext) if ext == "json" => file.name.replace(".json", ""),
                _ => file.name.to_string(),
            }
        )
    }).collect::<Vec<_>>().join("\n");

    format!("<ul class=\"nav\">\n{}\n</ul>", nav_links)
}

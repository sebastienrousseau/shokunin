use crate::models::data::FileData;
use crate::utilities::directory::to_title_case;
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
#[allow(clippy::redundant_clone)]
pub fn generate_navigation(files: &[FileData]) -> String {
    let supported_extensions = ["md", "toml", "json"];
    let mut files_sorted = files.to_vec();
    files_sorted.sort_by(|a, b| a.name.cmp(&b.name));

    let nav_links = files_sorted.iter().filter_map(|file| {
        let path = Path::new(&file.name);
        let file_stem = path.file_stem()
            .and_then(|n| n.to_str())
            .unwrap_or(&file.name)
            .to_string();

        let file_extension = path.extension()
            .and_then(|ext| ext.to_str());

        let file_name = match file_extension {
            Some(ext) if supported_extensions.contains(&ext) => file_stem,
            _ => file.name.to_string(),
        };

        if
            file_name == "index" ||
            file_name == "404" ||
            file_name == "privacy" ||
            file_name == "terms" ||
            file_name == "offline"
        {
            None
        } else {
            let mut dir_name = file_name.clone();

            // Handle special case for files in the same base directory
            if let Some((index, _)) = dir_name.match_indices('/').next() {
                let base_dir = &dir_name[..index];
                let file_name = &dir_name[index + 1..];
                dir_name = format!("{}{}", base_dir, file_name.replace(base_dir, ""));
            }

            Some(format!(
                "<li class=\"nav-item\"><a href=\"/{}/index.html\" class=\"text-uppercase p-2 \">{}</a></li>",
                dir_name,
                to_title_case(&file_name),
            ))
        }
    }).collect::<Vec<_>>().join("\n");

    format!(
        "<ul class=\"navbar-nav ms-auto mb-2 mb-lg-0\">\n{}\n</ul>",
        nav_links
    )
}

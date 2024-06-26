// Copyright © 2024 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

use crate::macro_render_layout;
use std::{
    collections::HashMap,
    fs::{self, File},
    path::{Path, PathBuf},
};

#[derive(Debug, Default, PartialEq, Eq, Clone)]
/// ## Struct: `PageOptions` - Options for rendering a page template
///
/// This struct contains the options for rendering a page template.
/// These options are used to construct a context `HashMap` that is
/// passed to the `render_template` function.
///
/// # Arguments
///
/// * `elements` - A `HashMap` containing the elements of the page.
///
pub struct PageOptions<'a> {
    /// Elements of the page
    pub elements: HashMap<&'a str, &'a str>,
}

impl<'a> PageOptions<'a> {
    /// ## Function: `new` - Create a new `PageOptions`
    pub fn new() -> PageOptions<'a> {
        PageOptions {
            elements: HashMap::new(),
        }
    }
    /// ## Function: `set` - Set a page option
    pub fn set(&mut self, key: &'a str, value: &'a str) {
        self.elements.insert(key, value);
    }

    /// ## Function: `get` - Get a page option
    pub fn get(&self, key: &'a str) -> Option<&&'a str> {
        self.elements.get(key)
    }
}

/// ## Function: `render_template` - Render a template with the given context
///
/// This function takes in a template string and a context hash map as
/// arguments. The template string is a string containing placeholders
/// for values that will be replaced with the values in the context hash
/// map. The placeholders take the form "{{key}}", where "key" is the
/// key of a value in the context hash map. For example, a template
/// string might contain the placeholder "{{name}}" which would be
/// replaced with the value of the "name" key in the context hash map.
///
/// The function replaces all placeholders in the template string with
/// their corresponding values from the context hash map. If a
/// placeholder cannot be replaced (i.e. there is no corresponding key
/// in the context hash map), an error is returned with a message
/// indicating which placeholder could not be resolved.
///
/// If all placeholders are successfully replaced, the resulting string
/// with replaced placeholders is returned as a `String` wrapped in a
/// `Result::Ok`. If an error occurs, an error message is returned as a
/// `String` wrapped in a `Result::Err`.
///
pub fn render_template(
    template: &str,
    context: &HashMap<&str, &str>,
) -> Result<String, String> {
    let mut output = template.to_owned();
    for (key, value) in context {
        output = output.replace(&format!("{{{{{}}}}}", key), value);
    }
    // Check if all keys have been replaced
    if output.contains("{{") {
        Err(format!(
            "Failed to render template, unresolved template tags: {}",
            output
        ))
    } else {
        Ok(output)
    }
}

/// Function: `render_page` - Render an HTML page
///
/// Renders an HTML page with given attributes contained within a `PageOptions` struct.
///
/// This function takes a `PageOptions` struct, which contains various elements of an HTML page,
/// stored as a HashMap. The key-value pairs in the HashMap are used to dynamically construct the page.
/// The HashMap is passed to the `macro_render_layout!` function along with the template HTML file and layout.
/// The resulting string returned by the macro is the final HTML page that is generated.
///
/// # Arguments
///
/// * `options`        - A reference to a `PageOptions` struct containing key-value pairs of page attributes.
/// * `template_path`  - A reference to a string representing the path to the template file.
/// * `layout`         - A reference to a string representing the layout.
///
/// # Returns
///
/// If the function succeeds, it returns `Ok(html)`, where `html` is the HTML page generated by the function.
/// If the function encounters an error, it returns `Err(error)`, where `error` is a string describing the error that occurred.
///
pub fn render_page(
    options: &PageOptions<'_>,
    template_path: &String,
    layout: &String,
) -> Result<String, String> {
    // Renders the page using the specified template and layout
    macro_render_layout!(layout, template_path, options.elements)
}

/// Custom error type to handle both reqwest and io errors
#[derive(Debug)]
pub enum TemplateError {
    /// Error from reqwest
    Reqwest(reqwest::Error),
    /// Error from io
    Io(std::io::Error),
}

impl From<reqwest::Error> for TemplateError {
    fn from(err: reqwest::Error) -> TemplateError {
        TemplateError::Reqwest(err)
    }
}

impl From<std::io::Error> for TemplateError {
    fn from(err: std::io::Error) -> TemplateError {
        TemplateError::Io(err)
    }
}

/**
 * Creates a template folder based on the provided template path or uses
 * the default template folder
 *
 * - If a URL is provided as the template path, the function downloads
 *   the template files to a temporary directory.
 * - If a local path is provided, the function uses it as the template
 *   directory path.
 * - If no path is provided, the function downloads the default template
 *   files to a temporary directory.
 *
 * # Arguments
 *
 * * `template_path` - An optional `&str` containing the path to the
 *   template folder
 *
 * # Returns
 *
 * Returns a `Result` that contains `()` if successful, or a
 * `TemplateError` if an error occurs.
 *
 */
pub fn create_template_folder(
    template_path: Option<&str>,
) -> Result<String, TemplateError> {
    // Get the current working directory
    let current_dir = std::env::current_dir()?;

    let template_dir_path = match template_path {
        Some(path)
            if path.starts_with("http://")
                || path.starts_with("https://") =>
        {
            download_files_from_url(path)?
        }
        Some(path) => {
            let local_path = current_dir.join(path);
            if local_path.exists() && local_path.is_dir() {
                println!("Using local template directory: {}", path);
                local_path
            } else {
                return Err(TemplateError::Io(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("Template directory not found: {}", path),
                )));
            }
        }
        None => {
            let default_url = "https://raw.githubusercontent.com/sebastienrousseau/shokunin/main/template/";
            download_files_from_url(default_url)?
        }
    };

    Ok(String::from(template_dir_path.to_str().unwrap()))
}

fn download_files_from_url(
    url: &str,
) -> Result<PathBuf, TemplateError> {
    let tempdir = tempfile::tempdir()?;
    let template_dir_path = tempdir.into_path();
    println!(
        "Creating temporary directory for template: {:?}",
        template_dir_path
    );

    let files = [
        "contact.html",
        "index.html",
        "main.js",
        "page.html",
        "post.html",
        "sw.js",
    ];

    for file in files.iter() {
        let file_url = format!("{}/{}", url, file);
        let file_path = template_dir_path.join(file);
        let mut download = reqwest::blocking::get(&file_url)?;
        let mut file = File::create(&file_path)?;
        download.copy_to(&mut file)?;
        println!("Downloaded template file to: {:?}", file_path);
    }

    Ok(template_dir_path)
}

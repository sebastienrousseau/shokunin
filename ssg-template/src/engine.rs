// engine.rs

use crate::{Context, TemplateError};
use std::collections::HashMap;
use std::fs::{self, File};
use std::path::{Path, PathBuf};

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

/// The main template rendering engine.
pub struct Engine {
    template_path: String,
}

impl Engine {
    /// Creates a new `Engine` instance.
    ///
    /// # Arguments
    ///
    /// * `template_path` - A string slice that holds the path to the template directory.
    ///
    /// # Examples
    ///
    /// ```
    /// use ssg_template::Engine;
    ///
    /// let engine = Engine::new("path/to/templates");
    /// ```
    pub fn new(template_path: &str) -> Self {
        Self {
            template_path: template_path.to_string(),
        }
    }

    /// Renders a page using the specified layout and context.
    ///
    /// # Arguments
    ///
    /// * `context` - A reference to a `Context` object containing the rendering context.
    /// * `layout` - A string slice specifying the layout to use.
    ///
    /// # Returns
    ///
    /// A `Result` containing the rendered page as a `String`, or a `TemplateError` if rendering fails.
    ///
    /// # Examples
    ///
    /// ```
    /// use ssg_template::{Engine, Context};
    ///
    /// let engine = Engine::new("path/to/templates");
    /// let mut context = Context::new();
    /// context.set("title", "My Page");
    ///
    /// match engine.render_page(&context, "default") {
    ///     Ok(rendered) => println!("Rendered page: {}", rendered),
    ///     Err(e) => eprintln!("Rendering error: {}", e),
    /// }
    /// ```
    pub fn render_page(
        &self,
        context: &Context,
        layout: &str,
    ) -> Result<String, TemplateError> {
        let template_path = Path::new(&self.template_path)
            .join(format!("{}.html", layout));
        let template_content = fs::read_to_string(&template_path)
            .map_err(TemplateError::Io)?;

        self.render_template(&template_content, &context.elements)
    }

    /// Renders a template string with the given context.
    ///
    /// # Arguments
    ///
    /// * `template` - A string slice that holds the template to render.
    /// * `context` - A reference to a `HashMap` containing the context for rendering.
    ///
    /// # Returns
    ///
    /// A `Result` containing the rendered string, or a `TemplateError` if rendering fails.
    pub fn render_template(
        &self,
        template: &str,
        context: &HashMap<&str, &str>,
    ) -> Result<String, TemplateError> {
        let mut output = template.to_owned();
        for (key, value) in context {
            output = output.replace(&format!("{{{{{}}}}}", key), value);
        }
        // Check if all keys have been replaced
        if output.contains("{{") {
            Err(TemplateError::RenderError(format!(
                "Failed to render template, unresolved template tags: {}",
                output
            )))
        } else {
            Ok(output)
        }
    }

    /// Creates a template folder based on the provided template path or uses the default template folder.
    ///
    /// # Arguments
    ///
    /// * `template_path` - An optional string slice containing the path to the template folder.
    ///
    /// # Returns
    ///
    /// A `Result` containing the path to the template folder as a `String`, or a `TemplateError` if an error occurs.
    ///
    /// # Examples
    ///
    /// ```
    /// use ssg_template::Engine;
    ///
    /// let engine = Engine::new("path/to/templates");
    /// match engine.create_template_folder(Some("custom/templates")) {
    ///     Ok(path) => println!("Template folder created at: {}", path),
    ///     Err(e) => eprintln!("Error creating template folder: {}", e),
    /// }
    /// ```
    pub fn create_template_folder(
        &self,
        template_path: Option<&str>,
    ) -> Result<String, TemplateError> {
        let current_dir =
            std::env::current_dir().map_err(TemplateError::Io)?;

        let template_dir_path = match template_path {
            Some(path)
                if path.starts_with("http://")
                    || path.starts_with("https://") =>
            {
                self.download_files_from_url(path)?
            }
            Some(path) => {
                let local_path = current_dir.join(path);
                if local_path.exists() && local_path.is_dir() {
                    println!(
                        "Using local template directory: {}",
                        path
                    );
                    local_path
                } else {
                    return Err(TemplateError::Io(
                        std::io::Error::new(
                            std::io::ErrorKind::NotFound,
                            format!(
                                "Template directory not found: {}",
                                path
                            ),
                        ),
                    ));
                }
            }
            None => {
                let default_url = "https://raw.githubusercontent.com/sebastienrousseau/shokunin/main/template/";
                self.download_files_from_url(default_url)?
            }
        };

        Ok(template_dir_path.to_str().unwrap().to_string())
    }

    /// Downloads template files from a URL and saves them to a temporary directory.
    ///
    /// # Arguments
    ///
    /// * `url` - A string slice containing the URL to download template files from.
    ///
    /// # Returns
    ///
    /// A `Result` containing the path to the temporary directory as a `PathBuf`,
    /// or a `TemplateError` if an error occurs during the download process.
    fn download_files_from_url(
        &self,
        url: &str,
    ) -> Result<PathBuf, TemplateError> {
        let tempdir = tempfile::tempdir().map_err(TemplateError::Io)?;
        let template_dir_path = tempdir.path().to_owned();
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
            let mut response = reqwest::blocking::get(&file_url)
                .map_err(TemplateError::Reqwest)?;
            let mut file =
                File::create(&file_path).map_err(TemplateError::Io)?;
            response
                .copy_to(&mut file)
                .map_err(TemplateError::Reqwest)?;
            println!("Downloaded template file to: {:?}", file_path);
        }

        Ok(template_dir_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_template() {
        let engine = Engine::new("dummy/path");
        let mut context = HashMap::new();
        context.insert("name", "World");
        context.insert("greeting", "Hello");

        let template = "{{greeting}}, {{name}}!";
        let result =
            engine.render_template(template, &context).unwrap();
        assert_eq!(result, "Hello, World!");
    }

    // Add more tests for other methods
}

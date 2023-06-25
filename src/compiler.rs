// Copyright © 2023 Shokunin (職人) Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

use crate::data::FileData;
use crate::{
    data::{
        CnameData, IconData, ManifestOptions, SitemapData, TxtData,
    },
    file::add,
    frontmatter::extract,
    html::generate_html,
    json::{cname, manifest, sitemap, txt},
    macro_cleanup_directories, macro_create_directories,
    macro_generate_metatags, macro_metadata_option,
    navigation::generate_navigation,
    rss::{generate_rss, RssOptions},
    template::{render_page, PageOptions},
    utilities::minify_html,
};
use std::{error::Error, fs, path::Path};

/// Compiles files in a source directory, generates HTML pages from them, and
/// writes the resulting pages to an output directory. Also generates an index
/// page containing links to the generated pages. This function takes in the
/// paths to the source and output directories as arguments.
///
/// The function reads in all Markdown files in the source directory and
/// extracts metadata from them. The metadata is used to generate appropriate
/// meta tags for the resulting HTML pages. The Markdown content of the files
/// is then converted to HTML using the `generate_html` function and rendered
/// into complete HTML pages using the `render_page` function. The resulting
/// pages are written to the output directory as HTML files with the same names
/// as the original Markdown files.
///
/// Finally, the function generates an index HTML page containing links to all
/// the generated pages. The resulting index page is written to the output
/// directory as "index.html.
///
/// If any errors occur during the process (e.g. a file cannot be read or
/// written), an error is returned. Otherwise, `Ok(())` is returned.
///
/// # Arguments
///
/// * `content_path` - The path to the content directory.
/// * `build_path` - The path to the output directory.
/// * `site_path` - The name of the site.
/// * `template_path` - The path to the template directory.
///
/// # Returns
///
/// `Ok(())` if the compilation is successful.
/// `Err` if an error occurs during the compilation. The error is
/// wrapped in a `Box<dyn Error>` to allow for different error types.
///
pub fn compile(
    build_path: &Path,    // The path to the temp directory
    content_path: &Path,  // The path to the content directory
    site_path: &Path,     // The path to the site directory
    template_path: &Path, // The path to the template directory
) -> Result<(), Box<dyn Error>> {
    // Declare the paths variables
    let build_path = build_path;
    let content_path = content_path;
    let site_path = site_path;
    let template_path = template_path;

    // Creating the build and site directories
    macro_create_directories!(build_path, site_path)?;

    // Read the files in the source directory
    let files = add(content_path)?;

    // Generate the HTML code for the navigation menu
    let navigation = generate_navigation(&files);

    let files_compiled: Vec<FileData> = files
        .into_iter()
        .map(|file| {
            // Extract metadata from front matter
            let metadata = extract(&file.content);
            let meta = macro_generate_metatags!(
                "url",
                &macro_metadata_option!(metadata, "permalink"),
            );

            // Generate HTML content
            let html_content = generate_html(
                &file.content,
                &macro_metadata_option!(metadata, "title"),
                &macro_metadata_option!(metadata, "description"),
                Some(&macro_metadata_option!(metadata, "content")),
            );

            // Generate HTML
            let mut page_options = PageOptions::new();
            for (key, value) in metadata.iter() {
                page_options.set(key, value);
            }

            // Adding meta and navigation
            page_options.set("meta", &meta);
            page_options.set("navigation", &navigation);
            page_options.set("content", &html_content);

            let content = render_page(
                &page_options,
                &template_path.to_str().unwrap().to_string(),
                metadata.get("layout").unwrap_or(&"".to_string()),
            )
            .unwrap();

            // Generate RSS
            let rss = generate_rss(&RssOptions {
                atom_link: macro_metadata_option!(
                    metadata,
                    "atom_link"
                ),
                category: macro_metadata_option!(metadata, "category"),
                copyright: macro_metadata_option!(
                    metadata,
                    "copyright"
                ),
                cloud: macro_metadata_option!(metadata, "cloud"),
                description: macro_metadata_option!(
                    metadata,
                    "description"
                ),
                docs: macro_metadata_option!(metadata, "docs"),
                enclosure: macro_metadata_option!(
                    metadata,
                    "enclosure"
                ),
                generator: macro_metadata_option!(
                    metadata,
                    "generator"
                ),
                image: macro_metadata_option!(metadata, "image"),
                item_description: macro_metadata_option!(
                    metadata,
                    "item_description"
                ),
                item_guid: macro_metadata_option!(
                    metadata,
                    "item_guid"
                ),
                item_link: macro_metadata_option!(
                    metadata,
                    "item_link"
                ),
                item_pub_date: macro_metadata_option!(
                    metadata,
                    "item_pub_date"
                ),
                item_title: macro_metadata_option!(
                    metadata,
                    "item_title"
                ),
                language: macro_metadata_option!(metadata, "language"),
                last_build_date: macro_metadata_option!(
                    metadata,
                    "last_build_date"
                ),
                link: macro_metadata_option!(metadata, "permalink"),
                managing_editor: macro_metadata_option!(
                    metadata,
                    "managing_editor"
                ),
                pub_date: macro_metadata_option!(metadata, "pub_date"),
                title: macro_metadata_option!(metadata, "title"),
                ttl: macro_metadata_option!(metadata, "ttl"),
                webmaster: macro_metadata_option!(
                    metadata,
                    "webmaster"
                ),
            });
            let rss_data = rss.unwrap();

            // Generate JSON
            let json = ManifestOptions {
                name: metadata
                    .get("name")
                    .unwrap_or(&"".to_string())
                    .to_string(),
                short_name: (macro_metadata_option!(
                    metadata,
                    "short_name"
                )),
                start_url: ".".to_string(),
                display: "standalone".to_string(),
                background_color: "#ffffff".to_string(),
                description: (macro_metadata_option!(
                    metadata,
                    "description"
                )),
                icons: match metadata.get("icon") {
                    Some(icon) => {
                        let icons = vec![IconData {
                            src: icon.to_string(),
                            sizes: "512x512".to_string(),
                            icon_type: Some("image/png".to_string()),
                            purpose: Some("any maskable".to_string()),
                        }];
                        icons
                    }
                    None => Vec::new(),
                },
                orientation: "portrait-primary".to_string(),
                scope: "/".to_string(),
                theme_color: (macro_metadata_option!(
                    metadata,
                    "theme_color"
                )),
            };

            let cname_options: CnameData = CnameData {
                cname: macro_metadata_option!(metadata, "cname"),
            };

            let sitemap_options: SitemapData = SitemapData {
                loc: macro_metadata_option!(metadata, "permalink"),
                lastmod: macro_metadata_option!(
                    metadata,
                    "last_build_date"
                ),
                changefreq: "weekly".to_string(),
            };

            let txt_options: TxtData = TxtData {
                permalink: macro_metadata_option!(
                    metadata,
                    "permalink"
                ),
            };

            let json_data = manifest(&json);
            let txt_data = txt(&txt_options);
            let cname_data = cname(&cname_options);
            let sitemap_data = sitemap(&sitemap_options, site_path);

            FileData {
                name: file.name,
                content,
                rss: rss_data,
                json: json_data,
                txt: txt_data,
                cname: cname_data,
                sitemap: sitemap_data,
            }
        })
        .collect();

    // Generate the HTML code for the navigation menu
    generate_navigation(&files_compiled);

    // Write the compiled files to the output directory
    println!(
        "❯ Writing the generated, compiled and minified files to the `{}` directory...",
        build_path.display()
    );
    for file in &files_compiled {
        let file_name = match Path::new(&file.name).extension() {
            Some(ext) if ext == "md" => file.name.replace(".md", ""),
            Some(ext) if ext == "toml" => {
                file.name.replace(".toml", "")
            }
            Some(ext) if ext == "json" => {
                file.name.replace(".json", "")
            }
            Some(ext) if ext == "js" => file.name.replace(".js", ""),
            Some(ext) if ext == "xml" => file.name.replace(".xml", ""),
            Some(ext) if ext == "txt" => file.name.replace(".txt", ""),
            _ => file.name.to_string(),
        };

        // Check if the filename is "index.md" and write it to the root directory
        if file_name == "index" {
            let cname_file = build_path.join("CNAME");
            let html_file = build_path.join("index.html");
            let json_file = build_path.join("manifest.json");
            let robots_file = build_path.join("robots.txt");
            let rss_file = build_path.join("rss.xml");
            let sitemap_file = build_path.join("sitemap.xml");

            fs::write(&cname_file, &file.cname)?;
            fs::write(&html_file, &file.content)?;
            fs::write(&json_file, &file.json)?;
            fs::write(&robots_file, &file.txt)?;
            fs::write(&rss_file, &file.rss)?;
            fs::write(&sitemap_file, &file.sitemap)?;

            // Create a backup of the source html file
            // let backup_file = backup_file(&html_file)?;
            // fs::write(&backup_file, &file.content)?;

            // Minify the html file and write it to the output directory
            let minified_file = minify_html(&html_file)?;
            fs::write(&html_file, &minified_file)?;

            println!("  - {}", html_file.display());
            println!("  - {}", rss_file.display());
            println!("  - {}", json_file.display());
            println!("  - {}", robots_file.display());
            println!("  - {}", cname_file.display());
            println!("  - {}", sitemap_file.display());
        } else {
            let dir_name = build_path.join(file_name.clone());
            fs::create_dir_all(&dir_name)?;

            let html_file = dir_name.join("index.html");
            let rss_file = dir_name.join("rss.xml");
            let json_file = dir_name.join("manifest.json");
            let robots_file = dir_name.join("robots.txt");
            let sitemap_file = dir_name.join("sitemap.xml");
            // let cname_file = dir_name.join("CNAME");

            fs::write(&html_file, &file.content)?;
            fs::write(&rss_file, &file.rss)?;
            fs::write(&json_file, &file.json)?;
            fs::write(&robots_file, &file.txt)?;
            fs::write(&sitemap_file, &file.sitemap)?;
            // fs::write(&cname_file, &file.name)?;

            // Create a backup of the source html file
            // let backup_file = backup_file(&html_file)?;
            // fs::write(&backup_file, &file.content)?;

            // Minify the html file and write it to the output directory
            let minified_file = minify_html(&html_file)?;
            fs::write(&html_file, &minified_file)?;

            println!("  - {}", html_file.display());
            println!("  - {}", rss_file.display());
            println!("  - {}", json_file.display());
            println!("  - {}", robots_file.display());
            println!("  - {}", sitemap_file.display());
            // println!("  - {}", cname_file.display());
        }
    }

    // Remove the site directory if it exists
    macro_cleanup_directories!(site_path);

    // Move the content of the build directory to the site directory and
    // remove the build directory
    fs::rename(build_path, site_path)?;

    Ok(())
}

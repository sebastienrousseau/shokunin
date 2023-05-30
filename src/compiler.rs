// Copyright © 2023 Shokunin (職人) Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

use crate::{
    file::{add, File},
    frontmatter::extract,
    html::generate_html,
    json::{
        cname, manifest, txt, CnameOptions, IconOptions,
        ManifestOptions, TxtOptions,
    },
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

    let files_compiled: Vec<File> = files
        .into_iter()
        .map(|file| {
            // Extract metadata from front matter
            let metadata = extract(&file.content);
            let meta =
                macro_generate_metatags!("url", &macro_metadata_option!(metadata, "permalink"),);

            // Generate HTML
            let content = render_page(
                &PageOptions {
                    author: &macro_metadata_option!(metadata, "author"),
                    banner: &macro_metadata_option!(metadata, "banner"),
                    banner_width: &macro_metadata_option!(metadata, "banner_width"),
                    banner_height: &macro_metadata_option!(metadata, "banner_height"),
                    banner_alt: &macro_metadata_option!(metadata, "banner_alt"),
                    bing_site_verification: &macro_metadata_option!(
                        metadata,
                        "bing_site_verification"
                    ),
                    charset: &macro_metadata_option!(metadata, "charset"),
                    content: &generate_html(
                        &file.content,
                        &macro_metadata_option!(metadata, "title"),
                        &macro_metadata_option!(metadata, "description"),
                        Some(&macro_metadata_option!(metadata, "content")),
                    ),
                    copyright: &macro_metadata_option!(metadata, "copyright"),
                    cname: &macro_metadata_option!(metadata, "cname"),
                    css: &macro_metadata_option!(metadata, "css"),
                    date: &macro_metadata_option!(metadata, "date"),
                    description: &macro_metadata_option!(metadata, "description"),
                    generator: &macro_metadata_option!(metadata, "generator"),
                    google_site_verification: &macro_metadata_option!(
                        metadata,
                        "google_site_verification"
                    ),
                    image: &macro_metadata_option!(metadata, "image"),
                    keywords: &macro_metadata_option!(metadata, "keywords"),
                    lang: &macro_metadata_option!(metadata, "language"),
                    layout: &macro_metadata_option!(metadata, "layout"),
                    logo: &macro_metadata_option!(metadata, "logo"),
                    logo_width: &macro_metadata_option!(metadata, "logo_width"),
                    logo_height: &macro_metadata_option!(metadata, "logo_height"),
                    logo_alt: &macro_metadata_option!(metadata, "logo_alt"),
                    meta: &meta,
                    msvalidate1: &macro_metadata_option!(metadata, "msvalidate1"),
                    msapplication_config: &macro_metadata_option!(metadata, "msapplication_config"),
                    msapplication_tap_highlight: &macro_metadata_option!(
                        metadata,
                        "msapplication-tap-highlight"
                    ),
                    msapplication_tile_color: &macro_metadata_option!(
                        metadata,
                        "msapplication-tile-color"
                    ),
                    msapplication_tile_image: &macro_metadata_option!(
                        metadata,
                        "msapplication-tile-image"
                    ),
                    name: &macro_metadata_option!(metadata, "name"),
                    navigation: &navigation,
                    og_description: &macro_metadata_option!(metadata, "og_description"),
                    og_image_alt: &macro_metadata_option!(metadata, "og_image_alt"),
                    og_image: &macro_metadata_option!(metadata, "og_image"),
                    og_locale: &macro_metadata_option!(metadata, "og_locale"),
                    og_site_name: &macro_metadata_option!(metadata, "og_site_name"),
                    og_title: &macro_metadata_option!(metadata, "og_title"),
                    og_type: &macro_metadata_option!(metadata, "og_type"),
                    og_url: &macro_metadata_option!(metadata, "og_url"),
                    robots: &macro_metadata_option!(metadata, "robots"),
                    subtitle: &macro_metadata_option!(metadata, "subtitle"),
                    theme_color: &macro_metadata_option!(metadata, "theme_color"),
                    title: &macro_metadata_option!(metadata, "title"),
                    twitter_card: &macro_metadata_option!(metadata, "twitter_card"),
                    twitter_creator: &macro_metadata_option!(metadata, "twitter_creator"),
                    twitter_description: &macro_metadata_option!(metadata, "twitter_description"),
                    twitter_image_alt: &macro_metadata_option!(metadata, "twitter_image_alt"),
                    twitter_image: &macro_metadata_option!(metadata, "twitter_image"),
                    twitter_site: &macro_metadata_option!(metadata, "twitter_site"),
                    twitter_title: &macro_metadata_option!(metadata, "twitter_title"),
                    twitter_url: &macro_metadata_option!(metadata, "twitter_url"),
                    url: &macro_metadata_option!(metadata, "url"),
                },
                &template_path.to_str().unwrap().to_string(),
                metadata.get("layout").unwrap_or(&"".to_string()),
            )
            .unwrap();

            // Generate RSS
            let rss = generate_rss(&RssOptions {
                title: (macro_metadata_option!(metadata, "title")),
                link: (macro_metadata_option!(metadata, "link")),
                description: (macro_metadata_option!(metadata, "description")),
                atom_link: (macro_metadata_option!(metadata, "atom_link")),
                last_build_date: (macro_metadata_option!(metadata, "last_build_date")),
                pub_date: (macro_metadata_option!(metadata, "pub_date")),
                generator: (macro_metadata_option!(metadata, "generator")),
                item_title: (macro_metadata_option!(metadata, "item_title")),
                item_link: (macro_metadata_option!(metadata, "item_link")),
                item_guid: (macro_metadata_option!(metadata, "item_guid")),
                item_description: (macro_metadata_option!(metadata, "item_description")),
                item_pub_date: (macro_metadata_option!(metadata, "item_pub_date")),
            });
            let rss_data = rss.unwrap();

            // Generate JSON
            let json = ManifestOptions {
                background_color: "#000".to_string(),
                description: metadata
                    .get("description")
                    .unwrap_or(&"".to_string())
                    .to_string(),
                dir: "/".to_string(),
                display: "fullscreen".to_string(),
                icons: match metadata.get("icon") {
                    Some(icon) => {
                        let icons = vec![IconOptions {
                            src: icon.to_string(),
                            sizes: "512x512".to_string(),
                            icon_type: Some("image/png".to_string()),
                            purpose: Some("any maskable".to_string()),
                        }];
                        icons
                    }
                    None => Vec::new(),
                },
                identity: "/".to_string(),
                lang: "en-GB".to_string(),
                name: metadata.get("name").unwrap_or(&"".to_string()).to_string(),
                orientation: "any".to_string(),
                scope: "/".to_string(),
                short_name: "/".to_string(),
                start_url: "/".to_string(),
                theme_color: "#fff".to_string(),
            };

            let cname_options: CnameOptions = CnameOptions {
                cname: macro_metadata_option!(metadata, "cname"),
            };

            let txt_options: TxtOptions = TxtOptions {
                permalink: macro_metadata_option!(metadata, "permalink"),
            };

            let json_data = manifest(&json);
            let txt_data = txt(&txt_options);
            let cname_data = cname(&cname_options);

            File {
                name: file.name,
                content,
                rss: rss_data,
                json: json_data,
                txt: txt_data,
                cname: cname_data,
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
            Some(ext) if ext == ".xml" => file.name.replace(".xml", ""),
            Some(ext) if ext == ".txt" => file.name.replace(".txt", ""),
            _ => file.name.to_string(),
        };

        // Check if the filename is "index.md" and write it to the root directory
        if file_name == "index" {
            let html_file = build_path.join("index.html");
            let rss_file = build_path.join("rss.xml");
            let json_file = build_path.join("manifest.json");
            let robots_file = build_path.join("robots.txt");
            let cname_file = build_path.join("CNAME");

            fs::write(&html_file, &file.content)?;
            fs::write(&cname_file, &file.cname)?;
            fs::write(&rss_file, &file.rss)?;
            fs::write(&json_file, &file.json)?;
            fs::write(&robots_file, &file.txt)?;

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
        } else {
            let dir_name = build_path.join(file_name.clone());
            fs::create_dir_all(&dir_name)?;

            let html_file = dir_name.join("index.html");
            let rss_file = dir_name.join("rss.xml");
            let json_file = dir_name.join("manifest.json");
            let robots_file = dir_name.join("robots.txt");
            // let cname_file = dir_name.join("CNAME");

            fs::write(&html_file, &file.content)?;
            fs::write(&rss_file, &file.rss)?;
            fs::write(&json_file, &file.json)?;
            fs::write(&robots_file, &file.txt)?;
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

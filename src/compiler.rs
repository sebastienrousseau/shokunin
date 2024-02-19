// Copyright © 2024 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

use rlg::LogLevel::ERROR;

use crate::{
    macro_cleanup_directories, macro_create_directories,
    macro_log_info, macro_metadata_option, macro_set_rss_data_fields,
    models::data::{FileData, PageData, RssData},
    modules::{
        cname::create_cname_data,
        html::generate_html,
        human::create_human_data,
        json::{cname, human, sitemap, txt},
        manifest::create_manifest_data,
        metadata::extract_and_prepare_metadata,
        navigation::generate_navigation,
        rss::generate_rss,
        sitemap::create_site_map_data,
        tags::*,
        txt::create_txt_data,
    },
    utilities::{
        file::add,
        template::{render_page, PageOptions},
        write::write_files_to_build_directory,
    },
};
use std::{collections::HashMap, error::Error, fs, path::Path};

/// Compiles files in a source directory, generates HTML pages from them, and
/// writes the resulting pages to an output directory. Also generates an index
/// page containing links to the generated pages.
///
/// This function takes paths to source, output, site, and template directories
/// as arguments and performs the compilation process. It reads Markdown files,
/// extracts metadata, generates HTML content, and writes resulting pages.
///
/// # Arguments
///
/// * `build_dir_path` - The path to the temporary build directory.
/// * `content_path` - The path to the content directory.
/// * `site_path` - The path to the output site directory.
/// * `template_path` - The path to the template directory.
///
/// # Returns
///
/// Returns `Ok(())` if the compilation is successful, otherwise returns an error
/// wrapped in a `Box<dyn Error>`.
pub fn compile(
    build_dir_path: &Path, // The path to the temp directory
    content_path: &Path,   // The path to the content directory
    site_path: &Path,      // The path to the site directory
    template_path: &Path,  // The path to the template directory
) -> Result<(), Box<dyn Error>> {
    // Create build and site directories
    macro_create_directories!(build_dir_path, site_path)?;

    // Read files in the source directory
    let source_files = add(content_path)?;

    // Generate navigation bar HTML
    let navigation = generate_navigation(&source_files); // Generates a navigation bar HTML

    let mut global_tags_data: HashMap<String, Vec<PageData>> =
        HashMap::new();

    // Process source files and store results in 'compiled_files' vector
    let compiled_files: Vec<FileData> = source_files
        .into_iter()
        .map(|file| {
            let (metadata, keywords, all_meta_tags) =
                extract_and_prepare_metadata(&file.content);

            // Generate HTML
            let html_content = generate_html(
                &file.content,
                &macro_metadata_option!(metadata, "title"),
                &macro_metadata_option!(metadata, "description"),
                Some(&macro_metadata_option!(metadata, "content")),
            )
            .unwrap_or_else(|err| {
                let description =
                    format!("Error generating HTML: {:?}", err);
                macro_log_info!(
                    &ERROR,
                    "compiler.rs - Line 81",
                    &description,
                    &LogFormat::CLF
                );
                String::from("Fallback HTML content")
            });

            // Create page options
            let mut page_options = PageOptions::new();
            for (key, value) in metadata.iter() {
                page_options.set(key, value);
            }

            // Set various meta tags
            page_options.set("apple", &all_meta_tags.apple);
            page_options.set("content", &html_content);
            page_options.set("microsoft", &all_meta_tags.ms);
            page_options.set("navigation", &navigation);
            page_options.set("opengraph", &all_meta_tags.og);
            page_options.set("primary", &all_meta_tags.primary);
            page_options.set("twitter", &all_meta_tags.twitter);

            // Render page content
            let content = render_page(
                &page_options,
                &template_path.to_str().unwrap().to_string(),
                &metadata.get("layout").cloned().unwrap_or_default(),
            )
            .unwrap();

            // Generate RSS data
            let mut rss_data = RssData::new();

            // Set fields using the helper macro
            macro_set_rss_data_fields!(
                rss_data,
                atom_link,
                macro_metadata_option!(metadata, "atom_link")
            );
            macro_set_rss_data_fields!(
                rss_data,
                author,
                macro_metadata_option!(metadata, "author")
            );
            macro_set_rss_data_fields!(
                rss_data,
                category,
                macro_metadata_option!(metadata, "category")
            );
            macro_set_rss_data_fields!(
                rss_data,
                copyright,
                macro_metadata_option!(metadata, "copyright")
            );
            macro_set_rss_data_fields!(
                rss_data,
                description,
                macro_metadata_option!(metadata, "description")
            );
            macro_set_rss_data_fields!(
                rss_data,
                docs,
                macro_metadata_option!(metadata, "docs")
            );
            macro_set_rss_data_fields!(
                rss_data,
                generator,
                macro_metadata_option!(metadata, "generator")
            );
            macro_set_rss_data_fields!(
                rss_data,
                image,
                macro_metadata_option!(metadata, "image")
            );
            macro_set_rss_data_fields!(
                rss_data,
                item_guid,
                macro_metadata_option!(metadata, "item_guid")
            );
            macro_set_rss_data_fields!(
                rss_data,
                item_description,
                macro_metadata_option!(metadata, "item_description")
            );
            macro_set_rss_data_fields!(
                rss_data,
                item_link,
                macro_metadata_option!(metadata, "item_link")
            );
            macro_set_rss_data_fields!(
                rss_data,
                item_pub_date,
                macro_metadata_option!(metadata, "item_pub_date")
            );
            macro_set_rss_data_fields!(
                rss_data,
                item_title,
                macro_metadata_option!(metadata, "item_title")
            );
            macro_set_rss_data_fields!(
                rss_data,
                language,
                macro_metadata_option!(metadata, "language")
            );
            macro_set_rss_data_fields!(
                rss_data,
                last_build_date,
                macro_metadata_option!(metadata, "last_build_date")
            );
            macro_set_rss_data_fields!(
                rss_data,
                link,
                macro_metadata_option!(metadata, "permalink")
            );
            macro_set_rss_data_fields!(
                rss_data,
                managing_editor,
                macro_metadata_option!(metadata, "managing_editor")
            );
            macro_set_rss_data_fields!(
                rss_data,
                pub_date,
                macro_metadata_option!(metadata, "pub_date")
            );
            macro_set_rss_data_fields!(
                rss_data,
                title,
                macro_metadata_option!(metadata, "title")
            );
            macro_set_rss_data_fields!(
                rss_data,
                ttl,
                macro_metadata_option!(metadata, "ttl")
            );

            // Generate RSS
            let rss = generate_rss(&rss_data);
            let rss_data = rss.unwrap();

            // Generate a manifest data structure by extracting relevant information from the metadata.
            let json = create_manifest_data(&metadata);

            // Create a structure to hold CNAME-related options, populated with values from the metadata.
            let cname_options = create_cname_data(&metadata);

            // Generate a structure for human-readable data, filling it with values from the metadata.
            let human_options = create_human_data(&metadata);

            // Initialize a structure to store sitemap-related information, using values from the metadata.
            let sitemap_options = create_site_map_data(&metadata);

            let tags_data = generate_tags(&file, &metadata);
            // println!("Tags: {:?}", tags_data);

            // Update the global tags data
            for (tag, pages_data) in tags_data.iter() {
                let page_info: Vec<PageData> = pages_data
                    .iter()
                    .map(|page_data| PageData {
                        title: page_data
                            .get("title")
                            .cloned()
                            .unwrap_or_default(),
                        description: page_data
                            .get("description")
                            .cloned()
                            .unwrap_or_default(),
                        permalink: page_data
                            .get("permalink")
                            .cloned()
                            .unwrap_or_default(),
                        date: page_data
                            .get("date")
                            .cloned()
                            .unwrap_or_default(),
                    })
                    .collect();

                global_tags_data
                    .entry(tag.clone())
                    .or_default()
                    .extend(page_info);
            }

            // Generate a TxtData structure, filling it with values extracted from the metadata.
            let txt_options = create_txt_data(&metadata);

            // Generate the data for the various files
            let txt_data = txt(&txt_options);
            let cname_data = cname(&cname_options);
            let human_data = human(&human_options);
            let sitemap_data = sitemap(sitemap_options, site_path);
            let json_data = serde_json::to_string(&json)
                .unwrap_or_else(|e| {
                    eprintln!("Error serializing JSON: {}", e);
                    String::new()
                });

            // Return FileData
            FileData {
                cname: cname_data,
                content,
                keyword: keywords.join(", "),
                human: human_data,
                json: json_data,
                name: file.name,
                rss: rss_data,
                sitemap: sitemap_data,
                txt: txt_data,
            }
        })
        .collect();

    // Write compiled files to output directory
    let cli_description = format!(
        "<Notice>: Successfully generated, compiled, and minified all HTML files to the `{:?}` directory",
        build_dir_path.display()
    );

    // Log the generated files information to a log file (shokunin.log)
    macro_log_info!(
        &ERROR,
        "compiler.rs - Line 280",
        &cli_description,
        &LogFormat::CLF
    );

    // Print the generated files to the console
    println!("{} ", cli_description);

    // Iterate over compiled files and write pages to output directory
    for file in &compiled_files {
        write_files_to_build_directory(
            build_dir_path,
            file,
            template_path,
        )?;
    }

    let tags_html_content = generate_tags_html(&global_tags_data);
    write_tags_html_to_file(&tags_html_content, build_dir_path)?;

    // Cleanup site directory
    macro_cleanup_directories!(site_path);

    // Move build content to site directory and remove build directory
    fs::rename(build_dir_path, site_path)?;

    Ok(())
}

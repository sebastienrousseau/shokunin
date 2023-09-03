// Copyright © 2023 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

use crate::{
    cname::create_cname_data,
    data::{FileData, RssData, MetaTagGroups},
    file::add,
    frontmatter::extract,
    human::create_human_data,
    json::{cname, human, sitemap, tags, txt},
    keywords::extract_keywords,
    macro_cleanup_directories, macro_create_directories,
    macro_metadata_option,macro_set_rss_data_fields,
    manifest::create_manifest_data,
    modules::html::generate_html,
    modules::metatags::generate_all_meta_tags,
    modules::rss::generate_rss,
    navigation::generate_navigation,
    sitemap::create_site_map_data,
    tags::create_tags_data,
    template::{render_page, PageOptions},
    txt::create_txt_data,
    write::write_files,
};
use std::{error::Error, fs, path::Path, collections::HashMap};

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

    // Process source files and store results in 'compiled_files' vector
    let compiled_files: Vec<FileData> = source_files
        .into_iter()
        .map(|file| {
            let (metadata, keywords, all_meta_tags) = extract_and_prepare_metadata(&file.content);

            // Generate HTML
            let html_content = generate_html(
                &file.content,
                &macro_metadata_option!(metadata, "title"),
                &macro_metadata_option!(metadata, "description"),
                Some(&macro_metadata_option!(metadata, "content")),
            )
            .unwrap_or_else(|err| {
                // Log the error and provide a fallback HTML content
                println!("Error generating HTML: {:?}", err);
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

            // Create a structure for tags-related data, populating it with values from the metadata.
            let tags_options = create_tags_data(&metadata);

            // Generate a TxtData structure, filling it with values extracted from the metadata.
            let txt_options = create_txt_data(&metadata);

            // Generate the data for the various files
            let txt_data = txt(&txt_options);
            let cname_data = cname(&cname_options);
            let human_data = human(&human_options);
            let sitemap_data = sitemap(sitemap_options, site_path);
            let tags_data: String = tags(&tags_options).html;
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
                tags: tags_data,
                txt: txt_data,
            }
        })
        .collect();

    // Write compiled files to output directory
    println!(
        "❯ Writing the generated, compiled and minified files to the `{}` directory...",
        build_dir_path.display()
    );

    // Iterate over compiled files and write pages to output directory
    for file in &compiled_files {
        write_files(build_dir_path, file, template_path)?;
    }

    // Cleanup site directory
    macro_cleanup_directories!(site_path);

    // Move build content to site directory and remove build directory
    fs::rename(build_dir_path, site_path)?;

    Ok(())
}

/// Extracts metadata from the content, generates keywords based on the metadata,
/// and prepares meta tag groups.
///
/// This function performs three key tasks:
/// 1. It extracts metadata from the front matter of the content.
/// 2. It extracts keywords based on this metadata.
/// 3. It generates various meta tags required for the page.
///
/// # Arguments
///
/// * `content` - A string slice representing the content from which to extract metadata.
///
/// # Returns
///
/// Returns a tuple containing:
/// * `HashMap<String, String>`: Extracted metadata
/// * `Vec<String>`: A list of keywords
/// * `MetaTagGroups`: A structure containing various meta tags
///
/// # Examples
///
/// ```rust
/// use ssg::data::FileData;
/// use ssg::compiler::extract_and_prepare_metadata;
///
/// let file_content = "---\n\n# Front Matter (YAML)\n\nauthor: \"Jane Doe\"\ncategory: \"Rust\"\ndescription: \"A blog about Rust programming.\"\nlayout: \"post\"\npermalink: \"https://example.com/blog/rust\"\ntags: \"rust,programming\"\ntitle: \"Rust\"\n\n---\n\n# Content\n\nThis is a blog about Rust programming.\n";
///
/// let (metadata, keywords, all_meta_tags) = extract_and_prepare_metadata(&file_content);
/// ```
pub fn extract_and_prepare_metadata(content: &str) -> (HashMap<String, String>, Vec<String>, MetaTagGroups) {
    // Extract metadata from front matter
    let metadata = extract(content);

    // Extract keywords
    let keywords = extract_keywords(&metadata);

    // Generate all meta tags
    let all_meta_tags = generate_all_meta_tags(&metadata);

    (metadata, keywords, all_meta_tags)
}


// Copyright © 2024 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

use anyhow::{Context, Result};
use rlg::log_level::LogLevel::ERROR;
use ssg_html::{generate_html, HtmlConfig};

use crate::{
    macro_cleanup_directories, macro_create_directories,
    macro_log_info, macro_metadata_option, macro_set_rss_data_fields,
    models::data::{FileData, PageData, RssData},
    modules::{
        cname::create_cname_data,
        human::create_human_data,
        json::{cname, human, news_sitemap, sitemap, txt},
        manifest::create_manifest_data,
        navigation::NavigationGenerator,
        news_sitemap::create_news_site_map_data,
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
use ssg_metadata::extract_and_prepare_metadata;
use std::{collections::HashMap, fs, path::Path};

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
) -> Result<()> {
    // Create build and site directories
    macro_create_directories!(build_dir_path, site_path)
        .context("Failed to create directories")?;

    // Read files in the source directory
    let source_files =
        add(content_path).context("Failed to read source files")?;

    // Generate navigation bar HTML
    let navigation =
        NavigationGenerator::generate_navigation(&source_files);

    let mut global_tags_data: HashMap<String, Vec<PageData>> =
        HashMap::new();

    // Process source files and store results in 'compiled_files' vector
    let compiled_files: Result<Vec<FileData>> = source_files
        .into_iter()
        .map(|file| -> Result<FileData> {
            let (metadata, keywords, all_meta_tags) =
                extract_and_prepare_metadata(&file.content).context(
                    "Failed to extract and prepare metadata",
                )?;

            // Create HtmlConfig instance
            let config = HtmlConfig {
                enable_syntax_highlighting: true, // You can make this configurable if needed
                minify_output: false, // Set to true if you want minified output
                add_aria_attributes: true,
                generate_structured_data: true,
            };

            // Generate HTML
            // print!("Generating HTML from Markdown: {}", file.content);
            let html_content = generate_html(&file.content, &config)
                .context("Failed to generate HTML")?;

            // Determine the filename without the extension
            // let filename_without_extension = Path::new(&file.name)
            //     .file_stem()
            //     .and_then(|stem| stem.to_str())
            //     .unwrap_or(&file.name);
            // let common_path = build_dir_path.to_str().unwrap();

            // let mut pdf_source_paths: Vec<PathBuf> = Vec::new();
            // let source_for_pdf =
            //     Path::new(content_path).join(&file.name);
            // pdf_source_paths.push(source_for_pdf);
            // let mut pdf_instance = PDFComposer::new();
            // pdf_instance.set_pdf_version(PDFVersion::V1_7);
            // pdf_instance.set_paper_size(PaperSize::A5);
            // pdf_instance.set_orientation(PaperOrientation::Landscape);
            // pdf_instance.set_margins("20");
            // pdf_instance.set_font(FontsStandard::TimesRoman);
            // pdf_instance.add_source_files(pdf_source_paths);
            // let author_entry = PDFDocInfoEntry {
            //     doc_info_entry: "Author",
            //     yaml_entry: "author",
            // };
            // let generator_entry = PDFDocInfoEntry {
            //     doc_info_entry: "generator",
            //     yaml_entry: "generator",
            // };
            // pdf_instance.set_doc_info_entry(author_entry);
            // pdf_instance.set_doc_info_entry(generator_entry);
            // let pdf_destination =
            //     Path::new(common_path).join(filename_without_extension);
            // pdf_instance.set_output_directory(
            //     pdf_destination.to_str().unwrap(),
            // );

            // pdf_instance.generate_pdfs();

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

            // Initialize a structure to store news sitemap-related information, using values from the metadata.
            let news_sitemap_options =
                create_news_site_map_data(&metadata);

            let tags_data = generate_tags(&file, &metadata);

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
            let news_sitemap_data = news_sitemap(news_sitemap_options);
            let json_data = serde_json::to_string(&json)
                .unwrap_or_else(|e| {
                    eprintln!("Error serializing JSON: {}", e);
                    String::new()
                });
            // Return FileData
            Ok(FileData {
                cname: cname_data,
                content,
                keyword: keywords.join(", "),
                human: human_data,
                json: json_data,
                name: file.name,
                rss: rss_data,
                sitemap: sitemap_data,
                sitemap_news: news_sitemap_data,
                txt: txt_data,
            })
        })
        .collect();

    // Write compiled files to output directory
    let cli_description = format!(
        "<Notice>: Successfully generated, compiled, and minified all HTML to the `{:?}` directory",
        site_path.display()
    );

    // Log the generated files information to a log file (shokunin.log)
    macro_log_info!(
        &ERROR,
        "compiler.rs - Line 280",
        &cli_description,
        &LogFormat::CLF
    );

    // Print the generated files to the console
    // println!("{} ", cli_description);

    // Iterate over compiled files and write pages to output directory
    for file in &compiled_files? {
        write_files_to_build_directory(
            build_dir_path,
            file,
            template_path,
        )?;
    }

    let tags_html_content = generate_tags_html(&global_tags_data);
    write_tags_html_to_file(&tags_html_content, build_dir_path)?;

    // Cleanup site directory
    macro_cleanup_directories!(site_path)
        .context("Failed to clean up site directory")?;

    // Move build content to site directory and remove build directory
    fs::rename(build_dir_path, site_path)
        .context("Failed to rename build directory")?;

    Ok(())
}

// Copyright © 2024 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

use anyhow::{Context as AnyhowContext, Result};
use html_generator::{generate_html, HtmlConfig};
use rlg::log_level::LogLevel;
use rss_gen::{
    data::{RssData, RssDataField, RssItem, RssItemField},
    generate_rss, macro_set_rss_data_fields,
};
use sitemap_gen::create_site_map_data;
use std::time::Duration;

use crate::{
    macro_cleanup_directories, macro_create_directories,
    macro_log_info, macro_metadata_option,
    models::data::{FileData, PageData},
    modules::{
        cname::create_cname_data,
        human::create_human_data,
        json::{cname, human, news_sitemap, sitemap, txt},
        manifest::create_manifest_data,
        navigation::NavigationGenerator,
        news_sitemap::create_news_site_map_data,
        tags::*,
        txt::create_txt_data,
    },
    utilities::{file::add, write::write_files_to_build_directory},
};
use metadata_gen::extract_and_prepare_metadata;
use ssg_template::{Context as TemplateContext, Engine, PageOptions};
use std::{collections::HashMap, fs, path::Path};

/// Compiles files in a source directory, generates HTML pages from them, and
/// writes the resulting pages to an output directory. Also generates an index
/// page containing links to the generated pages.
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
/// wrapped in a `anyhow::Error`.
pub fn compile(
    build_dir_path: &Path,
    content_path: &Path,
    site_path: &Path,
    template_path: &Path,
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

    // Initialize the templating engine
    let mut engine = Engine::new(
        template_path.to_str().unwrap(),
        Duration::from_secs(60),
    );

    // Process source files and store results in 'compiled_files' vector
    let compiled_files: Result<Vec<FileData>> = source_files
        .into_iter()
        .map(|file| {
            process_file(
                &file,
                &mut engine,
                template_path,
                &navigation,
                &mut global_tags_data,
                site_path,
            )
        })
        .collect();

    // Write compiled files to output directory
    let cli_description = format!(
        "<Notice>: Successfully generated, compiled, and minified all HTML to the `{:?}` directory",
        site_path.display()
    );

    // Log the generated files information to a log file (shokunin.log)
    macro_log_info!(
        &LogLevel::ERROR,
        "compiler.rs",
        &cli_description,
        &LogFormat::CLF
    );

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

/// Processes a single file, generating all necessary data and content.
fn process_file(
    file: &FileData,
    engine: &mut Engine,
    _template_path: &Path,
    navigation: &str,
    global_tags_data: &mut HashMap<String, Vec<PageData>>,
    site_path: &Path,
) -> Result<FileData> {
    let (metadata, keywords, all_meta_tags) =
        extract_and_prepare_metadata(&file.content)
            .context("Failed to extract and prepare metadata")?;

    // Create HtmlConfig instance
    let config = HtmlConfig {
        enable_syntax_highlighting: true,
        minify_output: false,
        add_aria_attributes: true,
        generate_structured_data: true,
    };

    // Generate HTML
    let html_content = generate_html(&file.content, &config)
        .context("Failed to generate HTML")?;

    // Create page options
    let mut page_options = PageOptions::new();
    for (key, value) in metadata.iter() {
        page_options.set(key.to_string(), value.to_string());
    }

    // Set various meta tags
    page_options.set("apple".to_string(), all_meta_tags.apple.clone());
    page_options.set("content".to_string(), html_content);
    page_options.set("microsoft".to_string(), all_meta_tags.ms.clone());
    page_options.set("navigation".to_string(), navigation.to_owned());
    page_options.set("opengraph".to_string(), all_meta_tags.og);
    page_options.set("primary".to_string(), all_meta_tags.primary);
    page_options.set("twitter".to_string(), all_meta_tags.twitter);

    // Convert PageOptions to TemplateContext
    let mut context = TemplateContext::new();
    for (key, value) in page_options.elements.iter() {
        context.set(key.to_string(), value.to_string());
    }

    // Render page content
    let content = engine.render_page(
        &context,
        metadata.get("layout").cloned().unwrap_or_default().as_str(),
    )?;

    // Generate RSS data for the main feed
    let mut rss_data = RssData::new(None);

    // Set fields using the helper macro for feed-level metadata
    macro_set_rss_data_fields!(
        rss_data,
        AtomLink = macro_metadata_option!(metadata, "atom_link"),
        Author = macro_metadata_option!(metadata, "author"),
        Category = macro_metadata_option!(metadata, "category"),
        Copyright = macro_metadata_option!(metadata, "copyright"),
        Description = macro_metadata_option!(metadata, "description"),
        Docs = macro_metadata_option!(metadata, "docs"),
        Generator = macro_metadata_option!(metadata, "generator"),
        ImageUrl = macro_metadata_option!(metadata, "image"),
        Guid = macro_metadata_option!(metadata, "guid"),
        Language = macro_metadata_option!(metadata, "language"),
        LastBuildDate =
            macro_metadata_option!(metadata, "last_build_date"),
        Link = macro_metadata_option!(metadata, "permalink"),
        ManagingEditor =
            macro_metadata_option!(metadata, "managing_editor"),
        PubDate = macro_metadata_option!(metadata, "pub_date"),
        Title = macro_metadata_option!(metadata, "title"),
        Ttl = macro_metadata_option!(metadata, "ttl"),
        Webmaster = macro_metadata_option!(metadata, "webmaster")
    );

    // Handle RSS item data
    if metadata.contains_key("item_title")
        && metadata.contains_key("item_description")
    {
        let mut rss_item = RssItem::new();

        rss_item = rss_item.set(
            RssItemField::Title,
            macro_metadata_option!(metadata, "item_title"),
        );
        rss_item = rss_item.set(
            RssItemField::Link,
            macro_metadata_option!(metadata, "item_link"),
        );
        rss_item = rss_item.set(
            RssItemField::Description,
            macro_metadata_option!(metadata, "item_description"),
        );

        if metadata.contains_key("item_guid") {
            rss_item = rss_item.set(
                RssItemField::Guid,
                macro_metadata_option!(metadata, "item_guid"),
            );
        }

        if metadata.contains_key("item_pub_date") {
            rss_item = rss_item.set(
                RssItemField::PubDate,
                macro_metadata_option!(metadata, "item_pub_date"),
            );
        }

        // Only add the RSS item if it contains valid title, link, and description
        if !rss_item.title.is_empty()
            && !rss_item.link.is_empty()
            && !rss_item.description.is_empty()
        {
            rss_data.add_item(rss_item);
        }
    }

    // Ensure managingEditor and webMaster have real names in the format "email (Real Name)"
    let managing_editor =
        macro_metadata_option!(metadata, "managing_editor");
    if !managing_editor.contains('(') {
        rss_data = rss_data.set(
            RssDataField::ManagingEditor,
            format!("{} (Default Name)", managing_editor),
        );
    }

    let web_master = macro_metadata_option!(metadata, "webmaster");
    if !web_master.contains('(') {
        rss_data = rss_data.set(
            RssDataField::Webmaster,
            format!("{} (Default Name)", web_master),
        );
    }

    // Validate image format - if it’s not JPEG or PNG, use a default image
    let image_url = macro_metadata_option!(metadata, "image");
    if !(image_url.ends_with(".jpg")
        || image_url.ends_with(".jpeg")
        || image_url.ends_with(".png")
        || image_url.ends_with(".gif")
        || image_url.ends_with(".webp"))
    {
        rss_data = rss_data.set(
            RssDataField::ImageUrl,
            image_url.clone(),
        );
    }

    // Fix atom:link href value to ensure it matches the actual RSS location
    rss_data = rss_data.set(
        RssDataField::AtomLink,
        format!("{}/rss.xml", image_url),
    );

    // Generate RSS XML
    let rss = generate_rss(&rss_data)?;
    // println!("{}", &rss);

    // Generate various data structures
    let json = create_manifest_data(&metadata);
    let cname_options = create_cname_data(&metadata);
    let human_options = create_human_data(&metadata);
    let sitemap_options = create_site_map_data(&metadata);
    let news_sitemap_options = create_news_site_map_data(&metadata);
    let tags_data = generate_tags(file, &metadata);

    // Update the global tags data
    update_global_tags_data(global_tags_data, &tags_data);

    // Generate a TxtData structure
    let txt_options = create_txt_data(&metadata);

    // Generate the data for the various files
    let txt_data = txt(&txt_options);
    let cname_data = cname(&cname_options);
    let human_data = human(&human_options);
    let sitemap_data = sitemap(sitemap_options?, site_path);
    let news_sitemap_data = news_sitemap(news_sitemap_options);
    let json_data = serde_json::to_string(&json).unwrap_or_else(|e| {
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
        name: file.name.clone(),
        rss,
        sitemap: sitemap_data,
        sitemap_news: news_sitemap_data,
        txt: txt_data,
    })
}

/// Updates the global tags data with new tag information.
fn update_global_tags_data(
    global_tags_data: &mut HashMap<String, Vec<PageData>>,
    tags_data: &HashMap<String, Vec<HashMap<String, String>>>,
) {
    for (tag, pages_data) in tags_data {
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
}

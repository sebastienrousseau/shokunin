// Copyright © 2023 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

use crate::{
    data::{
        CnameData, FileData, HumansData, IconData,
        ManifestData, RssData, SitemapData, TagsData, TxtData,
    },
    file::add,
    frontmatter::extract,
    html::generate_html,
    json::{cname, human, sitemap, tags, txt},
    keywords::extract_keywords,
    macro_cleanup_directories,
    macro_create_directories,
    macro_metadata_option,
    macro_set_rss_data_fields,
    navigation::generate_navigation,
    rss::generate_rss,
    template::{render_page, PageOptions},
    utilities::minify_html,
    metatags::{
        generate_apple_meta_tags,
        generate_ms_meta_tags,
        generate_og_meta_tags,
        generate_primary_meta_tags,
        generate_twitter_meta_tags,
    },
};
use std::{error::Error, fs, path::Path, collections::HashMap};


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
/// * `build_dir_path` - The path to the output directory.
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
    build_dir_path: &Path, // The path to the temp directory
    content_path: &Path,   // The path to the content directory
    site_path: &Path,      // The path to the site directory
    template_path: &Path,  // The path to the template directory
) -> Result<(), Box<dyn Error>> {
    // Creating the build and site path directories
    macro_create_directories!(build_dir_path, site_path)?;

    // Read the files in the source directory
    let source_files = add(content_path)?;

    // Generate the HTML code for the navigation menu
    let navigation = generate_navigation(&source_files);

    // The unique tags
    let mut tag_to_urls_and_titles: HashMap<String, Vec<(String, String)>> = HashMap::new();

    // Create a new vector named 'compiled_files' to store the processed file data.
    // This vector will hold the result of processing each file from 'source_files'.
    let compiled_files: Vec<FileData> = source_files
        .into_iter()
        .map(|file| {

    // Extract metadata from the front matter of the file's content and store it in 'metadata'.
    let metadata = extract(&file.content);

    // Extract keywords from the extracted metadata and store them in a combined variable named 'keywords'.
    let keywords = extract_keywords(&metadata);

    // Generate HTML meta tags specifically designed for Apple devices and settings.
    // These include meta tags such as 'apple-mobile-web-app-capable' and 'apple-touch-icon'.
    // The output is a string containing these meta tags formatted in HTML.
    let apple_meta_tags = generate_apple_meta_tags(&metadata);

    // Generate essential HTML meta tags common to all web pages.
    // These include primary information like 'author', 'description', and 'viewport' settings.
    // The output is a string containing these primary meta tags formatted in HTML.
    let primary_meta_tags = generate_primary_meta_tags(&metadata);

    // Generate HTML Open Graph meta tags, primarily used for enhancing the website's
    // social media presence. These include tags like 'og:title', 'og:description', and 'og:url'.
    // The output is a string containing these Open Graph meta tags formatted in HTML.
    let og_meta_tags = generate_og_meta_tags(&metadata);

    // Generate HTML meta tags specific to Microsoft's browser features.
    // These include tags like 'msapplication-navbutton-color'.
    // The output is a string containing these Microsoft-specific meta tags formatted in HTML.
    let ms_application_meta_tags = generate_ms_meta_tags(&metadata);

    // Generate HTML meta tags specific to Twitter cards.
    // These include Twitter-specific tags like 'twitter:card' and 'twitter:description'.
    // The output is a string containing these Twitter-specific meta tags formatted in HTML.
    let twitter_meta_tags = generate_twitter_meta_tags(&metadata);

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

    // Add metatags to page options for use in templates and navigation generation
    page_options.set("apple", &apple_meta_tags);
    page_options.set("content", &html_content);
    page_options.set("microsoft", &ms_application_meta_tags);
    page_options.set("navigation", &navigation);
    page_options.set("opengraph", &og_meta_tags);
    page_options.set("primary", &primary_meta_tags);
    page_options.set("twitter", &twitter_meta_tags);


            let content = render_page(
                &page_options,
                &template_path.to_str().unwrap().to_string(),
                metadata.get("layout").unwrap_or(&"".to_string()),
            )
            .unwrap();

            // Create a new RssData instance
            let mut rss_data = RssData::new();

            // Set fields using the helper macro
            macro_set_rss_data_fields!(rss_data, atom_link, macro_metadata_option!(metadata, "atom_link"));
            macro_set_rss_data_fields!(rss_data, author, macro_metadata_option!(metadata, "author"));
            macro_set_rss_data_fields!(rss_data, category, macro_metadata_option!(metadata, "category"));
            macro_set_rss_data_fields!(rss_data, copyright, macro_metadata_option!(metadata, "copyright"));
            macro_set_rss_data_fields!(rss_data, description, macro_metadata_option!(metadata, "description"));
            macro_set_rss_data_fields!(rss_data, docs, macro_metadata_option!(metadata, "docs"));
            macro_set_rss_data_fields!(rss_data, generator, macro_metadata_option!(metadata, "generator"));
            macro_set_rss_data_fields!(rss_data, image, macro_metadata_option!(metadata, "image"));
            macro_set_rss_data_fields!(rss_data, item_guid, macro_metadata_option!(metadata, "item_guid"));
            macro_set_rss_data_fields!(rss_data, item_description, macro_metadata_option!(metadata, "item_description"));
            macro_set_rss_data_fields!(rss_data, item_link, macro_metadata_option!(metadata, "item_link"));
            macro_set_rss_data_fields!(rss_data, item_pub_date, macro_metadata_option!(metadata, "item_pub_date"));
            macro_set_rss_data_fields!(rss_data, item_title, macro_metadata_option!(metadata, "item_title"));
            macro_set_rss_data_fields!(rss_data, language, macro_metadata_option!(metadata, "language"));
            macro_set_rss_data_fields!(rss_data, last_build_date, macro_metadata_option!(metadata, "last_build_date"));
            macro_set_rss_data_fields!(rss_data, link, macro_metadata_option!(metadata, "permalink"));
            macro_set_rss_data_fields!(rss_data, managing_editor, macro_metadata_option!(metadata, "managing_editor"));
            macro_set_rss_data_fields!(rss_data, pub_date, macro_metadata_option!(metadata, "pub_date"));
            macro_set_rss_data_fields!(rss_data, title, macro_metadata_option!(metadata, "title"));
            macro_set_rss_data_fields!(rss_data, ttl, macro_metadata_option!(metadata, "ttl"));

            // Generate RSS
            let rss = generate_rss(&rss_data);
            let rss_data = rss.unwrap();


            // Generate JSON
            let json = ManifestData {
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
                            icon_type: Some(
                                "image/svg+xml".to_string(),
                            ),
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

            let human_options: HumansData = HumansData {
                author: macro_metadata_option!(metadata, "author"),
                author_website: macro_metadata_option!(
                    metadata,
                    "author_website"
                ),
                author_twitter: macro_metadata_option!(
                    metadata,
                    "author_twitter"
                ),
                author_location: macro_metadata_option!(
                    metadata,
                    "author_location"
                ),
                thanks: macro_metadata_option!(metadata, "thanks"),
                site_last_updated: macro_metadata_option!(
                    metadata,
                    "site_last_updated"
                ),
                site_standards: macro_metadata_option!(
                    metadata,
                    "site_standards"
                ),
                site_components: macro_metadata_option!(
                    metadata,
                    "site_components"
                ),
                site_software: macro_metadata_option!(
                    metadata,
                    "site_software"
                ),
            };

            let sitemap_options: SitemapData = SitemapData {
                loc: macro_metadata_option!(metadata, "permalink"),
                lastmod: macro_metadata_option!(
                    metadata,
                    "last_build_date"
                ),
                changefreq: "weekly".to_string(),
            };

            let tags_options: TagsData = TagsData {
                titles: macro_metadata_option!(
                    metadata,
                    "title"
                ),
                descriptions: macro_metadata_option!(
                    metadata,
                    "description"
                ),
                permalinks: macro_metadata_option!(
                    metadata,
                    "permalink"
                ),
                keywords: macro_metadata_option!(
                    metadata,
                    "keywords"
                ),
            };

            let txt_options: TxtData = TxtData {
                permalink: macro_metadata_option!(
                    metadata,
                    "permalink"
                ),
            };

            let txt_data = txt(&txt_options);
            let cname_data = cname(&cname_options);
            let human_data = human(&human_options);
            let sitemap_data = sitemap(&sitemap_options, site_path);
            let tags_data: String = tags(&tags_options).html;
            let json_data = serde_json::to_string(&json)
                .unwrap_or_else(|e| {
                    eprintln!("Error serializing JSON: {}", e);
                    String::new()
                });

            // Return the FileData
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

        // Convert HashSet to Vec
        // let mut all_keywords_vec: Vec<String> = all_keywords_set.into_iter().collect();
        // all_keywords_vec.sort();

        // let all_tags = all_keywords_vec.join(", ");  // Join all unique and sorted keywords into a single string separated by commas and spaces
        // println!("All Tags: {:?}", all_tags);   // Print this once after all files have been processed

        // Generate HTML for all_tags
        // let all_tags_html = format!("<div id=\"all-tags\">\n<h2>All Tags:</h2>\n<p>{}</p>\n</div>", all_tags);
        // final_html_tags.push_str(&all_tags_html);


    // println!("Tags: {:?}", unique_tags);
    // generate_tags(&compiled_files, &unique_tags.iter().map(|tag| tag.as_str()).collect::<Vec<&str>>());

    // Generate the HTML code for the navigation menu
    generate_navigation(&compiled_files);

    // Write the compiled files to the output directory
    println!(
        "❯ Writing the generated, compiled and minified files to the `{}` directory...",
        build_dir_path.display()
    );
    for file in &compiled_files {
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

        let tags = file.keyword.split(','); // Split by comma
        for tag in tags {
            let tag = tag.trim().to_lowercase(); // Remove whitespace and convert to lowercase
            let title = &file_name; // Assume `file.title` contains the title of the page
            let file_name_without_extension = Path::new(&file.name)
                .file_stem()
                .unwrap()
                .to_str()
                .unwrap();
            let url = if file_name_without_extension == "index" {
                "/index.html".to_string() // Special case for the index page
            } else {
                format!("/{}/index.html", file_name_without_extension) // Generate URLs pointing to HTML files
            };
            tag_to_urls_and_titles.entry(tag).or_insert_with(Vec::new).push((url, title.clone()));
        }

        // Check if the filename is "index.md" and write it to the root directory
        if file_name == "index" {
            let cname_file = build_dir_path.join("CNAME");
            let html_file = build_dir_path.join("index.html");
            let human_file = build_dir_path.join("humans.txt");
            let json_file = build_dir_path.join("manifest.json");
            let main_file = build_dir_path.join("main.js");
            let robots_file = build_dir_path.join("robots.txt");
            let rss_file = build_dir_path.join("rss.xml");
            let tags_file = build_dir_path.join("tags.html");
            let sitemap_file = build_dir_path.join("sitemap.xml");
            let sw_file = build_dir_path.join("sw.js");

            fs::copy(&template_path.join("main.js"), &main_file)?;
            fs::copy(&template_path.join("sw.js"), &sw_file)?;
            fs::write(&cname_file, &file.cname)?;
            fs::write(&html_file, &file.content)?;
            fs::write(&human_file, &file.human)?;
            fs::write(&json_file, &file.json)?;
            fs::write(&robots_file, &file.txt)?;
            fs::write(&rss_file, &file.rss)?;
            fs::write(&sitemap_file, &file.sitemap)?;
            fs::write(&tags_file, &file.tags)?;

            // Create a backup of the source html file
            // let backup_file = backup_file(&html_file)?;
            // fs::write(&backup_file, &file.content)?;

            // Minify the html file and write it to the output directory
            let minified_file = minify_html(&html_file)?;
            fs::write(&html_file, &minified_file)?;

            println!("  - {}", cname_file.display());
            println!("  - {}", html_file.display());
            println!("  - {}", human_file.display());
            println!("  - {}", json_file.display());
            println!("  - {}", main_file.display());
            println!("  - {}", robots_file.display());
            println!("  - {}", rss_file.display());
            println!("  - {}", sitemap_file.display());
            println!("  - {}", sw_file.display());
            println!("  - {}", tags_file.display());
        } else {
            let dir_name = build_dir_path.join(file_name.clone());
            fs::create_dir_all(&dir_name)?;

            let html_file = dir_name.join("index.html");
            let json_file = dir_name.join("manifest.json");
            let robots_file = dir_name.join("robots.txt");
            let rss_file = dir_name.join("rss.xml");
            let sitemap_file = dir_name.join("sitemap.xml");
            let tags_file = dir_name.join("tags.html");

            fs::write(&html_file, &file.content)?;
            fs::write(&json_file, &file.json)?;
            fs::write(&robots_file, &file.txt)?;
            fs::write(&rss_file, &file.rss)?;
            fs::write(&sitemap_file, &file.sitemap)?;
            fs::write(&tags_file, &file.tags)?;

            // Create a backup of the source html file
            // let backup_file = backup_file(&html_file)?;
            // fs::write(&backup_file, &file.content)?;

            // Minify the html file and write it to the output directory
            let minified_file = minify_html(&html_file)?;
            fs::write(&html_file, &minified_file)?;

            println!("  - {}", html_file.display());
            println!("  - {}", json_file.display());
            println!("  - {}", robots_file.display());
            println!("  - {}", rss_file.display());
            println!("  - {}", sitemap_file.display());
            println!("  - {}", tags_file.display());
        }
    }

    let mut final_html_tags = String::new();
    final_html_tags.push_str("<div id=\"all-tags\">\n<h2>All Tags</h2>\n");

    // 1. Convert the keys to a Vec
    let mut tags: Vec<_> = tag_to_urls_and_titles.keys().collect();


    // 2. Sort the Vec
    tags.sort();

    // 3. Iterate over the sorted Vec
    for tag in tags {
        if let Some(urls_and_titles) = tag_to_urls_and_titles.get(tag) {  // Look up URLs and titles in HashMap
            let count = urls_and_titles.len();  // Get the number of pages for this tag
            let tag_url = format!("/tags/{}/index.html", tag);  // Generate the URL for this tag's page
            final_html_tags.push_str(&format!("<h3><a href=\"{}\">{} ({})</a></h3>\n<ul>\n", tag_url, tag, count));
            for (url, title) in urls_and_titles {
                final_html_tags.push_str(&format!("<li><a href=\"{}\">{}</a></li>\n", url, title));
            }
            final_html_tags.push_str("</ul>\n");
        }
    }

    final_html_tags.push_str("</div>\n");

    fs::write(build_dir_path.join("tags.html"), &final_html_tags)?;

    // Remove the site directory if it exists
    macro_cleanup_directories!(site_path);

    // Move the content of the build directory to the site directory and
    // remove the build directory
    fs::rename(build_dir_path, site_path)?;

    Ok(())
}

use crate::file::{add, File};
use crate::frontmatter::extract;
use crate::html::generate_html;
use crate::json::manifest;
use crate::json::ManifestOptions;
use crate::metatags::generate_metatags;
use crate::navigation::generate_navigation;
use crate::rss::generate_rss;
use crate::rss::RssOptions;
use crate::template::render_page;
use crate::template::{create_template_folder, PageOptions};
use std::{error::Error, fs, path::Path};

/// Compiles files in a source directory, generates HTML pages from
/// them, and writes the resulting pages to an output directory. Also
/// generates an index page containing links to the generated pages.
/// This function takes in the paths to the source and output
/// directories as arguments.
///
/// The function reads in all Markdown files in the source directory and
/// extracts metadata from them.
/// The metadata is used to generate appropriate meta tags for the
/// resulting HTML pages. The Markdown content of the files is then
/// converted to HTML using the `generate_html` function and rendered
/// into complete HTML pages using the `render_page` function. The
/// resulting pages are written to the output directory as HTML files
/// with the same names as the original Markdown files.
///
/// Finally, the function generates an index HTML page containing links
/// to all the generated pages. The resulting index page is written to
/// the output directory as "index.html".
///
/// If any errors occur during the process (e.g. a file cannot be read
/// or written), an error is returned. Otherwise, `Ok(())` is returned.
///
/// # Arguments
///
/// * `src_dir` - The path to the source directory.
/// * `out_dir` - The path to the output directory.
/// * `template_path` - The path to the template directory.
/// * `site_name` - The name of the site.
///
/// # Returns
///
/// `Ok(())` if the compilation is successful.
/// `Err` if an error occurs during the compilation. The error is
/// wrapped in a `Box<dyn Error>` to allow for different error types.
///
pub fn compile(
    src_dir: &Path,
    out_dir: &Path,
    template_path: Option<&String>,
    site_name: String,
) -> Result<(), Box<dyn Error>> {
    // Constants
    let src_dir = Path::new(src_dir);
    let out_dir = Path::new(out_dir);

    println!("\n❯ Generating a new site: \"{}\"", site_name);
    println!("  Done.\n");

    // Delete the output directory
    println!("\n❯ Deleting any previous directory...");
    fs::remove_dir_all(out_dir)?;
    fs::remove_dir(Path::new(&site_name))?;
    println!("  Done.\n");

    // Creating the template directory
    println!("\n❯ Creating template directory...");
    let template_path = create_template_folder(template_path)
        .expect("❌ Error: Could not create template directory");
    println!("  Done.\n");

    // Create the output directory
    println!("❯ Creating output directory...");
    fs::create_dir(out_dir)?;
    println!("  Done.\n");

    // Read the files in the source directory
    println!("❯ Reading files...");
    let files = add(src_dir)?;
    println!("  Found {} files to generate.", files.len());
    println!("  Done.\n");

    // Compile the files
    println!("❯ Compiling files...");

    // Generate the HTML code for the navigation menu
    let navigation = generate_navigation(&files);

    let files_compiled: Vec<File> = files
    .into_iter()
    .map(|file| {

        // Extract metadata from front matter
        let metadata = extract(&file.content);
        let meta = generate_metatags(&[("url".to_owned(), metadata.get("permalink").unwrap_or(&"".to_string()).to_string())]);

        // Generate HTML
        let content = render_page(&PageOptions {
            author: metadata.get("author").unwrap_or(&"".to_string()),
            banner: metadata.get("banner").unwrap_or(&"".to_string()),
            charset: metadata.get("charset").unwrap_or(&"".to_string()),
            content: &generate_html(&file.content,metadata.get("title").unwrap_or(&"".to_string()),metadata.get("description").unwrap_or(&"".to_string()),Some(metadata.get("content").unwrap_or(&"".to_string())),),
            copyright: format!("Copyright © {} 2023. All rights reserved.", site_name).as_str(),
            css: "style.css",
            date: metadata.get("date").unwrap_or(&"".to_string()),
            description: metadata.get("description").unwrap_or(&"".to_string()),
            generator: metadata.get("generator").unwrap_or(&"".to_string()),
            google_site_verification: metadata.get("google_site_verification").unwrap_or(&"".to_string()),
            image: metadata.get("image").unwrap_or(&"".to_string()),
            keywords: metadata.get("keywords").unwrap_or(&"".to_string()),
            lang: metadata.get("language").unwrap_or(&"".to_string()),
            layout: metadata.get("layout").unwrap_or(&"".to_string()),
            meta: &meta,
            bing_site_verification: metadata.get("bing_site_verification").unwrap_or(&"".to_string()),
            name: metadata.get("name").unwrap_or(&"".to_string()),
            navigation: &navigation,
            og_description: metadata.get("og_description").unwrap_or(&"".to_string()),
            og_image_alt: metadata.get("og_image_alt").unwrap_or(&"".to_string()),
            og_image: metadata.get("og_image").unwrap_or(&"".to_string()),
            og_locale: metadata.get("og_locale").unwrap_or(&"".to_string()),
            og_site_name: metadata.get("og_site_name").unwrap_or(&"".to_string()),
            og_title: metadata.get("og_title").unwrap_or(&"".to_string()),
            og_type: metadata.get("og_type").unwrap_or(&"".to_string()),
            og_url: metadata.get("og_url").unwrap_or(&"".to_string()),
            subtitle: metadata.get("subtitle").unwrap_or(&"".to_string()),
            twitter_card: metadata.get("twitter_card").unwrap_or(&"".to_string()),
            twitter_creator: metadata.get("twitter_creator").unwrap_or(&"".to_string()),
            twitter_description: metadata.get("twitter_description").unwrap_or(&"".to_string()),
            twitter_image_alt: metadata.get("twitter_image_alt").unwrap_or(&"".to_string()),
            twitter_image: metadata.get("twitter_image").unwrap_or(&"".to_string()),
            twitter_site: metadata.get("twitter_site").unwrap_or(&"".to_string()),
            twitter_title: metadata.get("twitter_title").unwrap_or(&"".to_string()),
            twitter_url: metadata.get("twitter_url").unwrap_or(&"".to_string()),
            title: metadata.get("title").unwrap_or(&"".to_string()),
        }, &template_path, metadata.get("layout").unwrap_or(&"".to_string()))
        .unwrap();


        // Generate RSS
        let rss = generate_rss(&RssOptions {
            title: metadata.get("title").unwrap_or(&"".to_string()).to_string(),
            link: metadata.get("link").unwrap_or(&"".to_string()).to_string(),
            description: metadata.get("description").unwrap_or(&"".to_string()).to_string(),
            atom_link: metadata.get("atom_link").unwrap_or(&"".to_string()).to_string(),
            last_build_date: metadata.get("last_build_date").unwrap_or(&"".to_string()).to_string(),
            pub_date: metadata.get("pub_date").unwrap_or(&"".to_string()).to_string(),
            generator: metadata.get("generator").unwrap_or(&"".to_string()).to_string(),
            item_title: metadata.get("item_title").unwrap_or(&"".to_string()).to_string(),
            item_link: metadata.get("item_link").unwrap_or(&"".to_string()).to_string(),
            item_guid: metadata.get("item_guid").unwrap_or(&"".to_string()).to_string(),
            item_description: metadata.get("item_description").unwrap_or(&"".to_string()).to_string(),
            item_pub_date: metadata.get("item_pub_date").unwrap_or(&"".to_string()).to_string(),
        });
        let rss_data = rss.unwrap();

        // Generate JSON
        let options = ManifestOptions {
            background_color: "#000".to_string(),
            description: metadata.get("description").unwrap_or(&"".to_string()).to_string(),
            dir: "/".to_string(),
            display: "fullscreen".to_string(),
            icons: "{ \"src\": \"icon/lowres.webp\", \"sizes\": \"64x64\", \"type\": \"image/webp\" }, { \"src\": \"icon/lowres.png\", \"sizes\": \"64x64\" }".to_string(),
            identity: "/".to_string(),
            lang: "en-GB".to_string(),
            name: metadata.get("name").unwrap_or(&"".to_string()).to_string(),
            orientation: "any".to_string(),
            scope: "/".to_string(),
            short_name: "/".to_string(),
            start_url: "/".to_string(),
            theme_color: "#fff".to_string(),
        };
        let json_data = manifest(&options);

        File {
            name: file.name,
            content,
            rss: rss_data,
            json: json_data,
        }
    })
    .collect();

    // Generate the HTML code for the navigation menu
    generate_navigation(&files_compiled);

    println!("  Done.\n");

    // Write the compiled files to the output directory
    println!("❯ Writing files...");
    for file in &files_compiled {
        let file_name = match Path::new(&file.name).extension() {
            Some(ext) if ext == "md" => file.name.replace(".md", ""),
            Some(ext) if ext == "toml" => {
                file.name.replace(".toml", "")
            }
            Some(ext) if ext == "json" => {
                file.name.replace(".json", "")
            }
            Some(ext) if ext == ".xml" => file.name.replace(".xml", ""),
            _ => file.name.to_string(),
        };

        // Check if the filename is "index.md" and write it to the root directory
        if file_name == "index" {
            let out_file = out_dir.join("index.html");
            let rss_file = out_dir.join("rss.xml");
            let out_json_file = out_dir.join("manifest.json");

            fs::write(&out_file, &file.content)?;
            fs::write(&rss_file, &file.rss)?;
            fs::write(&out_json_file, &file.json)?;

            println!("  - {}", out_file.display());
            println!("  - {}", rss_file.display());
            println!("  - {}", out_json_file.display());
        } else {
            let dir_name = out_dir.join(file_name.clone());
            fs::create_dir_all(&dir_name)?;

            let out_file = dir_name.join("index.html");
            let rss_file = dir_name.join("rss.xml");
            let out_json_file = dir_name.join("manifest.json");
            println!("  - {}", out_file.display());
            fs::write(&out_file, &file.content)?;
            fs::write(&rss_file, &file.rss)?;
            fs::write(&out_json_file, &file.json)?;

            println!("  - {}", out_file.display());
            println!("  - {}", rss_file.display());
            println!("  - {}", out_json_file.display());
        }
    }
    println!("  Done.\n");
    Ok(())
}

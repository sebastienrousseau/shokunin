// Copyright © 2023 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT


use crate::models::data::FileData;
use crate::utilities::minification::minify_html;
use std::error::Error;
use std::fs;
use std::path::Path;

/// Write the files to the build directory
pub fn write_files(build_dir_path: &Path, file: &FileData, template_path: &Path) -> Result<(), Box<dyn Error>> {
    let file_name = match Path::new(&file.name).extension() {
        Some(ext) if ext == "js" => file.name.replace(".js", ""),
        Some(ext) if ext == "json" => file.name.replace(".json", ""),
        Some(ext) if ext == "md" => file.name.replace(".md", ""),
        Some(ext) if ext == "toml" => file.name.replace(".toml", ""),
        Some(ext) if ext == "txt" => file.name.replace(".txt", ""),
        Some(ext) if ext == "xml" => file.name.replace(".xml", ""),
        _ => file.name.to_string(),
    };

    if file_name == "index" {
        let file_paths = [
            ("CNAME", &file.cname),
            ("humans.txt", &file.human),
            ("index.html", &file.content),
            ("manifest.json", &file.json),
            ("robots.txt", &file.txt),
            ("rss.xml", &file.rss),
            ("sitemap.xml", &file.sitemap),
        ];
        println!("\n {}", file_name.to_uppercase());
        for (file_name, content) in &file_paths {
            let file_path = build_dir_path.join(file_name);
            fs::write(&file_path, content)?;
            if file_name == &"index.html" {
                let minified_content = minify_html(&file_path)?;
                fs::write(&file_path, &minified_content)?;
            }
            println!("  - {}", file_path.display());
        }

        let other_files = ["main.js", "sw.js"];
        for file_name in &other_files {
            let dest_path = build_dir_path.join(file_name);
            fs::copy(&template_path.join(file_name), &dest_path)?;
        }
    } else {
        let dir_name = build_dir_path.join(file_name.clone());
        fs::create_dir_all(&dir_name)?;

        let file_paths = [
            ("index.html", &file.content),
            ("manifest.json", &file.json),
            ("robots.txt", &file.txt),
            ("rss.xml", &file.rss),
            ("sitemap.xml", &file.sitemap),
        ];

        println!("\n {}", file_name.to_uppercase());
        for (file_name, content) in &file_paths {
            let file_path = dir_name.join(file_name);
            fs::write(&file_path, content)?;
            if file_name == &"index.html" {
                let minified_content = minify_html(&file_path)?;
                fs::write(&file_path, &minified_content)?;
            }
            println!("  - {}", file_path.display());
        }
    }

    Ok(())
}
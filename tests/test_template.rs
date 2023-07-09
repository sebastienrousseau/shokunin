// #[cfg(test)]
// mod tests {
//     use ssg::template::{render_page, render_template, PageOptions};
//     use std::{collections::HashMap, error::Error};

//     #[test]
//     fn test_render_template() -> Result<(), Box<dyn Error>> {
//         let template = "<html><head><title>{{title}}</title></head><body>{{content}}</body></html>";
//         let mut context = HashMap::new();
//         context.insert("title", "My Title");
//         context.insert("content", "My Content");
//         let result = render_template(template, &context)?;
//         assert_eq!(
//             result,
//             "<html><head><title>My Title</title></head><body>My Content</body></html>"
//         );
//         Ok(())
//     }

//     #[test]
//     fn test_render_template_unresolved_tags() {
//         let template = "<html><head><title>{{title}}</title></head><body>{{content}}</body></html>";
//         let mut context = HashMap::new();
//         context.insert("title", "My Title");
//         let result = render_template(template, &context);
//         assert_eq!(
//             result,
//             Err("Failed to render template, unresolved template tags: <html><head><title>My Title</title></head><body>{{content}}</body></html>".to_owned())
//         );
//     }

//     #[test]
//     fn test_render_page() -> Result<(), Box<dyn Error>> {
//         // Prepare the test data
//         let mut options = PageOptions::new();
//         options.set("author", "John Doe");
//         options.set("banner_alt", "A simple test page");
//         options.set("banner_height", "100");
//         options.set("banner_width", "100");
//         options.set("banner", "./images/banner.png");
//         options.set("charset", "utf-8");
//         options.set("cname", "example");
//         options.set("content", "Hello, world!");
//         options.set("copyright", "Copyright 2023");
//         options.set("css", "styles.css");
//         options.set("date", "2021-01-01");
//         options.set("description", "A simple test page");
//         options.set("generator", "SSG");
//         options.set("image", "./images/test.png");
//         options.set("keywords", "test, page");
//         options.set("language", "en");
//         options.set("layout", "page");
//         options.set("logo_alt", "A simple test page");
//         options.set("logo_height", "100");
//         options.set("logo_width", "100");
//         options.set("logo", "./images/logo.png");
//         options.set("meta", "");
//         options.set("msapplication_config", "/browserconfig.xml");
//         options.set("msapplication_tap_highlight", "no");
//         options.set("msapplication_tile_color", "#da532c");
//         options.set("msapplication_tile_image", "/mstile-144x144.png");
//         options.set("name", "My Site");
//         options.set("navigation", "<nav>Home</nav>");
//         options.set("og_description", "A simple test page");
//         options.set("og_image_alt", "A simple test page");
//         options.set("og_image", "./images/test.png");
//         options.set("og_locale", "en_US");
//         options.set("og_site_name", "My Site");
//         options.set("og_title", "Test Page");
//         options.set("og_type", "website");
//         options.set("og_url", "https//example.com");
//         options.set("robots", "index, follow");
//         options.set("subtitle", "A simple test page");
//         options.set("theme_color", "#ffffff");
//         options.set("title", "Test Page");
//         options.set("twitter_card", "summary");
//         options.set("twitter_creator", "johndoe");
//         options.set("twitter_description", "A simple test page");
//         options.set("twitter_image_alt", "A simple test page");
//         options.set("twitter_image", "./images/test.png");
//         options.set("twitter_site", "johndoe");
//         options.set("twitter_title", "Test Page");
//         options.set("twitter_url", "https://example.com");
//         options.set("url", "https://example.com");

//         let template_path = String::from("./template");
//         let layout = String::from("page");

//         // Create a temporary directory and copy the template file into it
//         let tempdir = tempfile::tempdir().map_err(|err| {
//             format!("Could not create temporary directory: {}", err)
//         })?;
//         let template_file_path = tempdir.path().join("template.html");
//         std::fs::copy("template/template.html", template_file_path)
//             .map_err(|err| {
//                 format!("Could not copy template file: {}", err)
//             })?;

//         // Call the render_page function
//         let result = render_page(&options, &template_path, &layout);
//         if let Err(e) = &result {
//             println!("Error: {:?}", e);
//         }
//         assert!(result.is_ok());

//         Ok(())
//     }
// }

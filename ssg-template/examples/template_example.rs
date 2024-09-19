// Copyright © 2023-2024 SSG Template Library. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// See LICENSE-APACHE.md and LICENSE-MIT.md in the repository root for full license information.

//! # SSG Template Library Usage Examples
//!
//! This program demonstrates the usage of the `ssg-template` library, covering:
//! - Context management (adding key-value pairs)
//! - Template rendering
//! - Error handling

#![allow(missing_docs)]

use ssg_template::{Context, Engine, TemplateError};
use std::{
    collections::HashMap,
    fs::{self, File},
    io::Write,
};
use tempfile::tempdir;

/// Entry point for the `ssg-template` library usage examples.
fn main() -> Result<(), TemplateError> {
    println!(
        "🦀 Comprehensive SSG Template Library Usage Examples 🦀\n"
    );

    demonstrates_context_management_example()?;
    demonstrates_template_rendering_example()?;
    demonstrates_error_handling_example()?;
    demonstrates_template_with_loop_example()?;
    demonstrates_template_with_conditionals_example()?;
    demonstrates_template_with_partials_example()?;
    demonstrates_template_with_inheritance_example()?;
    demonstrates_multiple_templates_example()?;
    demonstrates_large_context_example()?;
    demonstrates_dynamic_content_example()?;
    demonstrates_nested_context_example()?;
    demonstrates_template_caching_example()?;
    demonstrates_template_debugging_example()?;
    demonstrates_form_template_example()?;
    demonstrates_custom_error_handling_example()?;
    demonstrates_pagination_example()?;
    demonstrates_internationalization_example()?;
    demonstrates_template_fragments_example()?;
    demonstrates_markdown_to_html_example()?;
    demonstrates_template_versioning_example()?;
    demonstrates_ajax_template_example()?;

    println!("\n🎉 All examples completed successfully!");

    Ok(())
}

/// Demonstrates basic usage of the `Context` struct for template rendering.
fn demonstrates_context_management_example() -> Result<(), TemplateError>
{
    println!("🦀 Context Management Examples 🦀");

    let mut context = Context::new();
    context.set("title".to_string(), "Welcome to SSG".to_string());
    context.set("author".to_string(), "Alice".to_string());

    // Retrieve and display context values
    if let Some(title) = context.get("title") {
        println!("Context title: ✅ {}", title);
    } else {
        println!("Failed to retrieve title from context");
    }

    if let Some(author) = context.get("author") {
        println!("Context author: ✅ {}", author);
    } else {
        println!("Failed to retrieve author from context");
    }

    Ok(())
}

/// Demonstrates rendering a basic HTML template using the `Engine` struct.
fn demonstrates_template_rendering_example() -> Result<(), TemplateError>
{
    println!("\n🦀 Template Rendering Example 🦀");

    // Step 1: Create a temporary directory for the template
    let dir = tempdir()?;
    let base_template_path = dir.path(); // Using the base directory, no subfolder

    // Step 2: Create the 'index.html' template file directly in the base directory
    let template_path = base_template_path.join("index.html");
    let mut template_file = File::create(&template_path)?;
    let template_content = r#"
        <html>
        <head><title>{{title}}</title></head>
        <body>{{body}}</body>
        </html>
    "#;
    template_file.write_all(template_content.as_bytes())?;

    // Step 3: Create the context with key-value pairs
    let mut context = Context::new();
    context.set("title".to_string(), "Hello World".to_string());
    context.set(
        "body".to_string(),
        "This is a rendered page.".to_string(),
    );

    // Step 4: Initialize the Engine with the base directory where the template is located
    let engine = Engine::new(base_template_path.to_str().unwrap()); // Provide the dynamic base directory

    // Step 5: Render the template by specifying the filename without the extension (just "index")
    let rendered_template = engine.render_page(&context, "index")?;
    println!("Rendered Template: ✅ \n{}", rendered_template);

    Ok(())
}

/// Demonstrates error handling in the `ssg-template` library.
fn demonstrates_error_handling_example() -> Result<(), TemplateError> {
    println!("\n🦀 Error Handling Examples 🦀");

    let mut context = Context::new();
    context.set("title".to_string(), "Error Example".to_string());

    // Create a temporary directory for the template
    let dir = tempdir()?;
    let template_dir_path = dir.path();

    // Create an invalid HTML template file inside the temporary directory
    let template_path = template_dir_path.join("invalid.html");
    let mut template_file = File::create(&template_path)?;
    let template_content = r#"
        <html>
        <head><title>{{title}}</title></head> <!-- Missing closing curly brace -->
        <body>{{body}}</body>
    "#;
    template_file.write_all(template_content.as_bytes())?;

    let engine = Engine::new(template_dir_path.to_str().unwrap());

    match engine.render_page(&context, "invalid") {
        // Pass only "invalid", not "invalid.html"
        Ok(rendered) => println!("Unexpected success: \n{}", rendered),
        Err(e) => println!("Expected error occurred: {}", e),
    }

    Ok(())
}

/// Demonstrates rendering a template with a loop using the `Engine` struct.
fn demonstrates_template_with_loop_example() -> Result<(), TemplateError>
{
    println!("\n🦀 Template with Loop Example 🦀");

    // Create the context with a manually generated list of items
    let mut context = Context::new();
    context.set("title".to_string(), "Items List".to_string());

    // Manually process the loop in Rust before rendering
    let items = vec!["Item 1", "Item 2", "Item 3"];
    let mut items_html = String::new();
    for item in items {
        items_html.push_str(&format!("<li>{}</li>", item));
    }
    context.set("items".to_string(), items_html); // Add the generated HTML to the context

    // Create a temporary directory for the template
    let dir = tempdir()?;
    let template_dir_path = dir.path();

    // Create a loop HTML template file inside the temporary directory
    let template_path = template_dir_path.join("items.html");
    let mut template_file = File::create(&template_path)?;
    let template_content = r#"
        <html>
        <head><title>{{title}}</title></head>
        <body>
            <ul>{{items}}</ul> <!-- Render the pre-generated list here -->
        </body>
        </html>
    "#;
    template_file.write_all(template_content.as_bytes())?;

    let engine = Engine::new(template_dir_path.to_str().unwrap());

    let rendered_template = engine.render_page(&context, "items")?;
    println!("Rendered Template with Loop: ✅ \n{}", rendered_template);

    Ok(())
}

/// Demonstrates rendering a template with conditional logic using the `Engine` struct.
fn demonstrates_template_with_conditionals_example(
) -> Result<(), TemplateError> {
    println!("\n🦀 Template with Conditionals Example 🦀");

    let mut context = Context::new();
    context
        .set("title".to_string(), "Conditional Rendering".to_string());

    // Handle the conditional logic in Rust before rendering
    let logged_in = true; // Simulate logged-in user
    let body_content = if logged_in {
        "<p>Welcome back, user!</p>"
    } else {
        "<p>Please log in.</p>"
    };
    context.set("body".to_string(), body_content.to_string());

    // Create a temporary directory for the template
    let dir = tempdir()?;
    let template_dir_path = dir.path();

    // Create a conditional HTML template file inside the temporary directory
    let template_path = template_dir_path.join("conditional.html");
    let mut template_file = File::create(&template_path)?;
    let template_content = r#"
        <html>
        <head><title>{{title}}</title></head>
        <body>{{body}}</body>
        </html>
    "#;
    template_file.write_all(template_content.as_bytes())?;

    let engine = Engine::new(template_dir_path.to_str().unwrap());

    let rendered_template =
        engine.render_page(&context, "conditional")?;
    println!(
        "Rendered Template with Conditionals: ✅ \n{}",
        rendered_template
    );

    Ok(())
}

/// Demonstrates rendering a template with nested context values using the `Engine` struct.
fn demonstrates_nested_context_example() -> Result<(), TemplateError> {
    println!("\n🦀 Nested Context Example 🦀");

    let mut context = Context::new();
    context
        .set("title".to_string(), "Nested Context Example".to_string());
    context.set("user.name".to_string(), "John Doe".to_string()); // Nested context for user details
    context.set("user.age".to_string(), "30".to_string());

    // Create a temporary directory for the template
    let dir = tempdir()?;
    let template_dir_path = dir.path();

    // Create a nested context HTML template file inside the temporary directory
    let template_path = template_dir_path.join("user_profile.html");
    let mut template_file = File::create(&template_path)?;
    let template_content = r#"
        <html>
        <head><title>{{title}}</title></head>
        <body>
            <p>Name: {{user.name}}</p>
            <p>Age: {{user.age}}</p>
        </body>
        </html>
    "#;
    template_file.write_all(template_content.as_bytes())?;

    let engine = Engine::new(template_dir_path.to_str().unwrap());

    let rendered_template =
        engine.render_page(&context, "user_profile")?; // Pass only "user_profile", not "user_profile.html"

    println!(
        "Rendered Template with Nested Context: ✅ \n{}",
        rendered_template
    );

    Ok(())
}

/// This function demonstrates rendering a template with partials using the `Engine` struct.
fn demonstrates_template_with_partials_example(
) -> Result<(), TemplateError> {
    println!("\n🦀 Template with Partials Example 🦀");

    // Create the context for the main page
    let mut context = Context::new();
    context.set("title".to_string(), "Page with Partials".to_string());
    context.set(
        "body".to_string(),
        "This is the main content.".to_string(),
    );

    // Create a temporary directory for the template
    let dir = tempdir()?;
    let template_dir_path = dir.path();

    // Create the header and footer partials
    let header_path = template_dir_path.join("header.html");
    let mut header_file = File::create(&header_path)?;
    header_file
        .write_all(b"<header><h1>Welcome to {{title}}</h1></header>")?;

    let footer_path = template_dir_path.join("footer.html");
    let mut footer_file = File::create(&footer_path)?;
    footer_file.write_all(b"<footer>Footer Content</footer>")?;

    // Create the main template that includes the partials
    let main_template_path =
        template_dir_path.join("page_with_partials.html");
    let mut main_template_file = File::create(&main_template_path)?;
    let main_template_content = r#"
        {{header}}
        <main>{{body}}</main>
        {{footer}}
    "#;
    main_template_file.write_all(main_template_content.as_bytes())?;

    let engine = Engine::new(template_dir_path.to_str().unwrap());

    // Render the header and footer templates separately and inject the results into the main template
    let rendered_header = engine.render_page(&context, "header")?;
    context.set("header".to_string(), rendered_header);

    let rendered_footer = engine.render_page(&context, "footer")?;
    context.set("footer".to_string(), rendered_footer);

    // Render the main template
    let rendered_template =
        engine.render_page(&context, "page_with_partials")?;
    println!(
        "Rendered Template with Partials: ✅ \n{}",
        rendered_template
    );

    Ok(())
}

/// Demonstrates template inheritance using a base layout and extending templates.
fn demonstrates_template_with_inheritance_example(
) -> Result<(), TemplateError> {
    println!("\n🦀 Template with Inheritance Example (Manual) 🦀");

    // Create a temporary directory for the templates
    let dir = tempdir()?;
    let template_dir_path = dir.path();

    // Create a base layout template (layout.html)
    let layout_path = template_dir_path.join("layout.html");
    let mut layout_file = File::create(&layout_path)?;
    let layout_content = r#"
        <html>
        <head><title>{{title}}</title></head>
        <body>
            <header>{{header}}</header>
            <main>{{content}}</main>
            <footer>{{footer}}</footer>
        </body>
        </html>
    "#;
    layout_file.write_all(layout_content.as_bytes())?;

    // Create a page content that will be injected into the layout (about.html)
    let about_path = template_dir_path.join("about.html");
    let mut about_file = File::create(&about_path)?;
    let about_content = r#"
        <p>This is the about page.</p>
    "#;
    about_file.write_all(about_content.as_bytes())?;

    // Set up the context for layout template
    let mut context = Context::new();
    context.set("title".to_string(), "About Us".to_string());
    context.set(
        "header".to_string(),
        "Welcome to the About Page".to_string(),
    );
    context.set("footer".to_string(), "Footer Content".to_string());

    // Initialize the Engine with the template directory
    let engine = Engine::new(template_dir_path.to_str().unwrap());

    // Render the 'about' content separately
    let about_content = fs::read_to_string(about_path)?;
    context.set("content".to_string(), about_content);

    // Render the layout and inject the 'about' content
    let rendered_template = engine.render_page(&context, "layout")?;
    println!(
        "Rendered Template with Inheritance (Manual): ✅ \n{}",
        rendered_template
    );

    Ok(())
}

/// Demonstrates rendering multiple different pages using different templates.
fn demonstrates_multiple_templates_example() -> Result<(), TemplateError>
{
    println!("\n🦀 Multiple Templates Example 🦀");

    // Create a temporary directory for the templates
    let dir = tempdir()?;
    let template_dir_path = dir.path();

    // Create a home template (home.html)
    let home_path = template_dir_path.join("home.html");
    let mut home_file = File::create(&home_path)?;
    let home_content = r#"
        <html>
        <head><title>{{title}}</title></head>
        <body>
            <h1>Welcome to {{title}}</h1>
            <p>This is the homepage.</p>
        </body>
        </html>
    "#;
    home_file.write_all(home_content.as_bytes())?;

    // Create a contact template (contact.html)
    let contact_path = template_dir_path.join("contact.html");
    let mut contact_file = File::create(&contact_path)?;
    let contact_content = r#"
        <html>
        <head><title>{{title}}</title></head>
        <body>
            <h1>{{title}}</h1>
            <p>Contact us at {{email}}</p>
        </body>
        </html>
    "#;
    contact_file.write_all(contact_content.as_bytes())?;

    // Set up the context for the homepage
    let mut context_home = Context::new();
    context_home.set("title".to_string(), "Home Page".to_string());

    // Set up the context for the contact page
    let mut context_contact = Context::new();
    context_contact.set("title".to_string(), "Contact Us".to_string());
    context_contact
        .set("email".to_string(), "contact@example.com".to_string());

    // Initialize the Engine with the template directory
    let engine = Engine::new(template_dir_path.to_str().unwrap());

    // Render the home and contact pages independently
    let rendered_home = engine.render_page(&context_home, "home")?;
    println!("Rendered Home Page: ✅ \n{}", rendered_home);

    let rendered_contact =
        engine.render_page(&context_contact, "contact")?;
    println!("Rendered Contact Page: ✅ \n{}", rendered_contact);

    Ok(())
}

/// Demonstrates handling a large context with many key-value pairs.
fn demonstrates_large_context_example() -> Result<(), TemplateError> {
    println!("\n🦀 Large Context Example 🦀");

    let mut context = Context::new();

    // Add many key-value pairs to the context
    for i in 1..101 {
        context.set(format!("key{}", i), format!("Value {}", i));
    }

    // Create a temporary directory for the template
    let dir = tempdir()?;
    let template_dir_path = dir.path();

    // Create a template that uses multiple keys from the context
    let template_path = template_dir_path.join("large_context.html");
    let mut template_file = File::create(&template_path)?;
    let mut template_content = String::from("<html><body>\n");
    for i in 1..101 {
        // Use the correct placeholder format: {{key1}}, {{key2}}, etc.
        template_content
            .push_str(&format!("<p>{{{{key{}}}}}</p>\n", i));
    }
    template_content.push_str("</body></html>");
    template_file.write_all(template_content.as_bytes())?;

    // Initialize the Engine with the template directory
    let engine = Engine::new(template_dir_path.to_str().unwrap());

    // Render the template with the large context
    let rendered_template =
        engine.render_page(&context, "large_context")?;
    println!(
        "Rendered Template with Large Context: ✅ \n{}",
        rendered_template
    );

    Ok(())
}

/// Demonstrates dynamic content rendering based on context values.
fn demonstrates_dynamic_content_example() -> Result<(), TemplateError> {
    println!("\n🦀 Dynamic Content Example 🦀");

    // Create a context where the content changes dynamically
    let mut context = Context::new();
    let user_language = "es"; // Example language setting: "es" for Spanish, "en" for English

    if user_language == "es" {
        context.set("greeting".to_string(), "Hola Mundo".to_string());
    } else {
        context.set("greeting".to_string(), "Hello World".to_string());
    }

    // Create a temporary directory for the template
    let dir = tempdir()?;
    let template_dir_path = dir.path();

    // Create a simple template that displays the dynamic greeting
    let template_path = template_dir_path.join("dynamic_content.html");
    let mut template_file = File::create(&template_path)?;
    let template_content = r#"
        <html>
        <head><title>{{greeting}}</title></head>
        <body>
            <h1>{{greeting}}</h1>
        </body>
        </html>
    "#;
    template_file.write_all(template_content.as_bytes())?;

    let engine = Engine::new(template_dir_path.to_str().unwrap());

    // Render the template with the dynamic content
    let rendered_template =
        engine.render_page(&context, "dynamic_content")?;
    println!(
        "Rendered Template with Dynamic Content: ✅ \n{}",
        rendered_template
    );

    Ok(())
}

/// Demonstrates caching frequently used templates to improve rendering performance.
fn demonstrates_template_caching_example() -> Result<(), TemplateError>
{
    println!("\n🦀 Template Caching Example 🦀");

    // Create a temporary directory for the template
    let dir = tempdir()?;
    let template_dir_path = dir.path();

    // Create a simple template
    let template_path = template_dir_path.join("cached_template.html");
    let mut template_file = File::create(&template_path)?;
    let template_content = r#"
        <html>
        <head><title>{{title}}</title></head>
        <body>{{body}}</body>
        </html>
    "#;
    template_file.write_all(template_content.as_bytes())?;

    // Initialize the Engine with the template directory
    let engine = Engine::new(template_dir_path.to_str().unwrap());

    // Create a context for rendering
    let mut context = Context::new();
    context.set("title".to_string(), "Cached Template".to_string());
    context.set("body".to_string(), "This page is cached.".to_string());

    // Cache templates to avoid reloading them multiple times
    let mut template_cache: HashMap<String, String> = HashMap::new();
    if let Some(cached_template) = template_cache.get("cached_template")
    {
        println!("Using cached template: ✅ \n{}", cached_template);
    } else {
        let rendered_template =
            engine.render_page(&context, "cached_template")?;
        template_cache.insert(
            "cached_template".to_string(),
            rendered_template.clone(),
        );
        println!(
            "Rendered and cached template: ✅ \n{}",
            rendered_template
        );
    }

    Ok(())
}

/// Demonstrates debugging a template by identifying missing or undefined variables.
fn demonstrates_template_debugging_example() -> Result<(), TemplateError>
{
    println!("\n🦀 Template Debugging Example 🦀");

    let mut context = Context::new();
    context.set("title".to_string(), "Debugging Template".to_string());

    // Intentionally leaving "body" out to trigger a debugging scenario

    // Create a temporary directory for the template
    let dir = tempdir()?;
    let template_dir_path = dir.path();

    // Create a simple template that expects "body" in the context
    let template_path = template_dir_path.join("debug_template.html");
    let mut template_file = File::create(&template_path)?;
    let template_content = r#"
        <html>
        <head><title>{{title}}</title></head>
        <body>{{body}}</body>
        </html>
    "#;
    template_file.write_all(template_content.as_bytes())?;

    let engine = Engine::new(template_dir_path.to_str().unwrap());

    // Try to render the template and catch any missing context variables
    match engine.render_page(&context, "debug_template") {
        Ok(rendered) => {
            println!("Rendered Template: ✅ \n{}", rendered)
        }
        Err(e) => println!("Error during rendering: ❌ \n{}", e),
    }

    Ok(())
}

/// Demonstrates rendering a form with dynamic fields.
fn demonstrates_form_template_example() -> Result<(), TemplateError> {
    println!("\n🦀 Form Template Example 🦀");

    // Create a temporary directory for the template
    let dir = tempdir()?;
    let template_dir_path = dir.path();

    // Create a form template with placeholders for form fields
    let template_path = template_dir_path.join("form_template.html");
    let mut template_file = File::create(&template_path)?;
    let template_content = r#"
        <html>
        <head><title>{{title}}</title></head>
        <body>
            <form action="/submit" method="POST">
                <label for="name">Name:</label>
                <input type="text" id="name" name="name" value="{{name}}">
                <br>
                <label for="email">Email:</label>
                <input type="email" id="email" name="email" value="{{email}}">
                <br>
                <input type="submit" value="Submit">
            </form>
        </body>
        </html>
    "#;
    template_file.write_all(template_content.as_bytes())?;

    // Create the context for the form
    let mut context = Context::new();
    context.set("title".to_string(), "User Form".to_string());
    context.set("name".to_string(), "John Doe".to_string());
    context.set("email".to_string(), "john@example.com".to_string());

    // Initialize the Engine with the template directory
    let engine = Engine::new(template_dir_path.to_str().unwrap());

    // Render the form template
    let rendered_template =
        engine.render_page(&context, "form_template")?;
    println!("Rendered Form Template: ✅ \n{}", rendered_template);

    Ok(())
}

/// Demonstrates custom error handling for missing or invalid templates.
fn demonstrates_custom_error_handling_example(
) -> Result<(), TemplateError> {
    println!("\n🦀 Custom Error Handling Example 🦀");

    let mut context = Context::new();
    context
        .set("title".to_string(), "Custom Error Example".to_string());

    // Try to render a non-existent template to trigger a custom error
    let engine = Engine::new("path/to/nonexistent/templates");

    match engine.render_page(&context, "nonexistent_template") {
        Ok(rendered) => println!("Unexpected success: \n{}", rendered),
        Err(e) => match e {
            TemplateError::Io(_) => {
                println!("Custom Error: Template file not found! ❌")
            }
            TemplateError::RenderError(msg) => {
                println!("Custom Render Error: {} ❌", msg)
            }
            _ => println!("General Error: {} ❌", e),
        },
    }

    Ok(())
}

/// Demonstrates rendering paginated content.
fn demonstrates_pagination_example() -> Result<(), TemplateError> {
    println!("\n🦀 Pagination Example 🦀");

    // Create a temporary directory for the template
    let dir = tempdir()?;
    let template_dir_path = dir.path();

    // Create a paginated template without using unsupported `{% for %}` loop
    let template_path =
        template_dir_path.join("pagination_template.html");
    let mut template_file = File::create(&template_path)?;
    let template_content = r#"
        <html>
        <head><title>{{title}}</title></head>
        <body>
            <ul>{{posts}}</ul> <!-- Render pre-processed posts here -->
            <p>Page {{page}} of {{total_pages}}</p>
        </body>
        </html>
    "#;
    template_file.write_all(template_content.as_bytes())?;

    // Create the context for pagination
    let mut context = Context::new();
    context.set("title".to_string(), "Paginated Content".to_string());

    // Manually process the list of posts in Rust before rendering
    let posts = vec!["Post 1", "Post 2", "Post 3"];
    let mut posts_html = String::new();
    for post in posts {
        posts_html.push_str(&format!("<li>{}</li>", post));
    }
    context.set("posts".to_string(), posts_html); // Add pre-generated HTML to the context
    context.set("page".to_string(), "1".to_string());
    context.set("total_pages".to_string(), "3".to_string());

    // Initialize the Engine with the template directory
    let engine = Engine::new(template_dir_path.to_str().unwrap());

    // Render the paginated template with the pre-processed posts
    let rendered_template =
        engine.render_page(&context, "pagination_template")?;
    println!("Rendered Paginated Template: ✅ \n{}", rendered_template);

    Ok(())
}

/// Demonstrates internationalization by rendering language-specific templates.
fn demonstrates_internationalization_example(
) -> Result<(), TemplateError> {
    println!("\n🦀 Internationalization (i18n) Example 🦀");

    // Create a context with translations
    let mut context = Context::new();
    let lang = "fr"; // Assume we choose "fr" for French

    if lang == "fr" {
        context.set(
            "greeting".to_string(),
            "Bonjour le monde".to_string(),
        );
    } else {
        context.set("greeting".to_string(), "Hello World".to_string());
    }

    // Create a temporary directory for the template
    let dir = tempdir()?;
    let template_dir_path = dir.path();

    // Create a simple template for internationalization
    let template_path = template_dir_path.join("i18n_template.html");
    let mut template_file = File::create(&template_path)?;
    let template_content = r#"
        <html>
        <head><title>{{greeting}}</title></head>
        <body>
            <h1>{{greeting}}</h1>
        </body>
        </html>
    "#;
    template_file.write_all(template_content.as_bytes())?;

    let engine = Engine::new(template_dir_path.to_str().unwrap());

    // Render the template with the language-based greeting
    let rendered_template =
        engine.render_page(&context, "i18n_template")?;
    println!("Rendered Template with i18n: ✅ \n{}", rendered_template);

    Ok(())
}

/// Demonstrates template fragments by loading and rendering multiple template parts.
fn demonstrates_template_fragments_example() -> Result<(), TemplateError>
{
    println!("\n🦀 Template Fragments Example 🦀");

    // Create a temporary directory for the template
    let dir = tempdir()?;
    let template_dir_path = dir.path();

    // Create a fragment for the header
    let header_path = template_dir_path.join("header.html");
    let mut header_file = File::create(&header_path)?;
    header_file.write_all(b"<header><h1>Site Header</h1></header>")?;

    // Create a fragment for the footer
    let footer_path = template_dir_path.join("footer.html");
    let mut footer_file = File::create(&footer_path)?;
    footer_file.write_all(b"<footer>Site Footer</footer>")?;

    // Create a main template that includes fragments
    let main_template_path =
        template_dir_path.join("page_with_fragments.html");
    let mut main_template_file = File::create(&main_template_path)?;
    let main_template_content = r#"
        {{header}}
        <main>This is the page content.</main>
        {{footer}}
    "#;
    main_template_file.write_all(main_template_content.as_bytes())?;

    let engine = Engine::new(template_dir_path.to_str().unwrap());

    // Load and render fragments
    let rendered_header =
        engine.render_page(&Context::new(), "header")?;
    let rendered_footer =
        engine.render_page(&Context::new(), "footer")?;

    let mut context = Context::new();
    context.set("header".to_string(), rendered_header);
    context.set("footer".to_string(), rendered_footer);

    // Render the main template with fragments
    let rendered_template =
        engine.render_page(&context, "page_with_fragments")?;
    println!(
        "Rendered Template with Fragments: ✅ \n{}",
        rendered_template
    );

    Ok(())
}

/// Demonstrates converting markdown to HTML manually in a template.
fn demonstrates_markdown_to_html_example() -> Result<(), TemplateError>
{
    println!("\n🦀 Markdown to HTML Example (Manual Conversion) 🦀");

    // Step 1: Create a context
    let mut context = Context::new();

    // Step 2: Simulate simple markdown input
    let markdown_content =
        "# Hello World\nThis is a **markdown** example.";

    // Step 3: Perform a very basic manual markdown-to-HTML conversion
    let html_content = markdown_content
        .replace("# ", "<h1>") // Convert # header to <h1>
        .replace("\n", "</h1>\n") // Close the header tag with newline
        .replace("**", "<strong>") // Simulate bold text (start)
        .replace("**", "</strong>"); // Simulate bold text (end)

    // Set the converted HTML content into the context
    context.set("content".to_string(), html_content);

    // Step 4: Create a temporary directory for the template
    let dir = tempdir()?;
    let template_dir_path = dir.path();

    // Step 5: Create a simple template to display the HTML content
    let template_path =
        template_dir_path.join("markdown_template.html");
    let mut template_file = File::create(&template_path)?;
    let template_content = r#"
        <html>
        <head><title>Markdown to HTML</title></head>
        <body>{{content}}</body>
        </html>
    "#;
    template_file.write_all(template_content.as_bytes())?;

    // Step 6: Initialize the Engine with the template directory
    let engine = Engine::new(template_dir_path.to_str().unwrap());

    // Step 7: Render the template with the manually converted markdown content
    let rendered_template =
        engine.render_page(&context, "markdown_template")?;
    println!(
        "Rendered Markdown to HTML (Manual Conversion): ✅ \n{}",
        rendered_template
    );

    Ok(())
}

/// Demonstrates template versioning by switching between multiple versions of a template.
fn demonstrates_template_versioning_example(
) -> Result<(), TemplateError> {
    println!("\n🦀 Template Versioning Example 🦀");

    // Create a temporary directory for the templates
    let dir = tempdir()?;
    let template_dir_path = dir.path();

    // Create two versions of a template (v1.html and v2.html)
    let v1_path = template_dir_path.join("v1.html");
    let mut v1_file = File::create(&v1_path)?;
    v1_file.write_all(b"<h1>Version 1</h1>")?;

    let v2_path = template_dir_path.join("v2.html");
    let mut v2_file = File::create(&v2_path)?;
    v2_file.write_all(b"<h1>Version 2</h1>")?;

    // Create the context
    let context = Context::new();

    // Simulate template versioning by choosing v1 or v2 based on a condition
    let client_version = 2; // Let's assume this is the client version
    let template_version =
        if client_version == 1 { "v1" } else { "v2" };

    // Initialize the Engine with the template directory
    let engine = Engine::new(template_dir_path.to_str().unwrap());

    // Render the appropriate version of the template
    let rendered_template =
        engine.render_page(&context, template_version)?;
    println!(
        "Rendered Template Version {}: ✅ \n{}",
        client_version, rendered_template
    );

    Ok(())
}

/// Demonstrates rendering a template that is dynamically loaded via AJAX.
fn demonstrates_ajax_template_example() -> Result<(), TemplateError> {
    println!("\n🦀 AJAX Template Example 🦀");

    // Create the context with some data for dynamic update
    let mut context = Context::new();
    context.set(
        "message".to_string(),
        "This content is dynamically loaded.".to_string(),
    );

    // Create a temporary directory for the template
    let dir = tempdir()?;
    let template_dir_path = dir.path();

    // Create a template that is meant to be loaded via AJAX
    let template_path = template_dir_path.join("ajax_template.html");
    let mut template_file = File::create(&template_path)?;
    let template_content = r#"
        <div id="ajax-content">{{message}}</div>
        <script>
            fetch('/ajax-endpoint')
                .then(response => response.text())
                .then(data => document.getElementById('ajax-content').innerHTML = data);
        </script>
    "#;
    template_file.write_all(template_content.as_bytes())?;

    let engine = Engine::new(template_dir_path.to_str().unwrap());

    // Render the template meant for AJAX
    let rendered_template =
        engine.render_page(&context, "ajax_template")?;
    println!("Rendered AJAX Template: ✅ \n{}", rendered_template);

    Ok(())
}

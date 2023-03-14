use std::collections::HashMap;
use std::fs;

pub(crate) fn render_template(
    template: &str,
    context: &HashMap<&str, &str>,
) -> Result<String, String> {
    let mut output = template.to_owned();
    for (key, value) in context {
        output = output.replace(&format!("{{{{{}}}}}", key), value);
    }
    // println!("output: {}", output);
    if output.contains("{{") {
        Err(format!(
            "Failed to render template, unresolved template tags: {}",
            output
        ))
    } else {
        Ok(output)
    }
}

pub(crate) fn render_page(
    title: &str,
    description: &str,
    keywords: &str,
    meta: &str,
    css: &str,
    content: &str,
    copyright: &str,
) -> Result<String, String> {
    let mut context = HashMap::new();
    context.insert("title", title);
    context.insert("description", description);
    context.insert("keywords", keywords);
    context.insert("meta", meta);
    context.insert("css", css);
    context.insert("content", content);
    context.insert("copyright", copyright);

    let template = fs::read_to_string("./template/template.html").map_err(|e| format!("{}", e))?;

    render_template(&template, &context)
}

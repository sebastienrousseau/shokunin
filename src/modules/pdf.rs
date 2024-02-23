use regex::Regex;
use std::error::Error;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;
use printpdf::{BuiltinFont, Mm, PdfDocument};

/// Generate a PDF document from plain text content and save it to the specified output path.
///
/// # Arguments
///
/// * `plain_text` - A string slice that holds the plain text content to be converted to PDF.
/// * `output_dir` - A string slice that specifies the output directory for the generated PDF.
/// * `filename` - A string slice that specifies the filename of the generated PDF.
///
/// # Returns
///
/// A `Result` indicating success or failure.
///
pub fn generate_pdf(
    plain_text: &str,
    output_dir: &str,
    filename: &str,
) -> Result<(), Box<dyn Error>> {
    std::fs::create_dir_all(output_dir)?;
    let pdf_file_path = Path::new(output_dir).join(format!("{}.pdf", filename));
    let (doc, page1, layer1) = PdfDocument::new("PDF Document", Mm(210.0), Mm(297.0), "Layer 1");
    let mut current_layer = doc.get_page(page1).get_layer(layer1);

    let margin = Mm(20.0);
    let page_width = Mm(210.0) - (margin * 2.0);
    let page_height = Mm(297.0) - (margin * 2.0);
    let font = doc.add_builtin_font(BuiltinFont::Helvetica)?;
    let font_size = 13.0;
    let line_height = Mm(6.0);

    let cleaned_content = remove_css_classes(plain_text)?;
    let mut remaining_text = cleaned_content.trim_start();
    let mut rect_y = page_height - margin;

    while !remaining_text.is_empty() {
        let mut line_width = 0.0;
        let mut line_end = remaining_text.len();
        let char_height = line_height.0 * 1.4618;

        for (i, c) in remaining_text.char_indices() {
            let char_width = font_size / 6.1618;
            if line_width + char_width > page_width.0 {
                line_end = i;
                break;
            }
            line_width += char_width;
            if c == '\n' {
                line_end = i + 1;
                break;
            }
        }

        let (line, rest) = remaining_text.split_at(line_end);
        remaining_text = rest.trim_start();

        let start_x = margin.0;

        current_layer.begin_text_section();
        current_layer.set_font(&font, font_size);
        current_layer.set_character_spacing(0.0618);
        current_layer.set_line_height(line_height.0);
        current_layer.set_text_cursor(Mm(start_x), rect_y);
        current_layer.write_text(line, &font);
        current_layer.end_text_section();

        rect_y -= Mm(char_height);

        if rect_y.0 < margin.0 {
            let (page, layer) = doc.add_page(Mm(210.0), Mm(297.0), "Next Page");
            rect_y = page_height - margin;
            current_layer = doc.get_page(page).get_layer(layer);
        }
    }

    let file = File::create(pdf_file_path)?;
    let mut buf_writer = BufWriter::new(file);
    doc.save(&mut buf_writer)?;

    Ok(())
}

/// Removes CSS class attributes from the content.
///
/// # Arguments
///
/// * `content` - A string slice that holds the content with potential CSS class attributes.
///
/// # Returns
///
/// Returns a `Result<String, Box<dyn Error>>`, where `Ok(String)` is the cleaned content,
/// and `Err` contains error information if regex compilation or replacement fails.
fn remove_css_classes(content: &str) -> Result<String, Box<dyn Error>> {
    let class_regex = Regex::new(
        r#"(?x)  # Enable comment mode for whitespace and comments in the regex
        \.class\s*=\s*"[^"]*"  # Match .class="..." attributes
        |  # OR
        \:\ # Match any other class attributes with a space
        |  # OR
        !\w+\s+\w+ # Match any other class attributes with a space
        "#,
    )?;

    let cleaned_content = class_regex.replace_all(content, "");
    Ok(cleaned_content.into_owned())
}
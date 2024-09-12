use crate::models::Metadata;
use anyhow::Result;

/// Processes the extracted metadata.
pub fn process_metadata(metadata: &Metadata) -> Result<Metadata> {
    let mut processed = metadata.clone();

    // Convert dates to a standard format
    if let Some(date) = processed.get_mut("date") {
        *date = standardize_date(date);
    }

    // Ensure required fields are present
    ensure_required_fields(&mut processed)?;

    // Generate derived fields
    generate_derived_fields(&mut processed);

    Ok(processed)
}

fn standardize_date(date: &str) -> String {
    // Implement date standardization logic here
    // For example, convert various date formats to ISO 8601
    // This is a placeholder implementation
    date.to_string()
}

fn ensure_required_fields(metadata: &mut Metadata) -> Result<()> {
    let required_fields = vec!["title", "date"];

    for field in required_fields {
        if !metadata.contains_key(field) {
            anyhow::bail!("Missing required metadata field: {}", field);
        }
    }

    Ok(())
}

fn generate_derived_fields(metadata: &mut Metadata) {
    // Generate a URL slug from the title if not present
    if !metadata.contains_key("slug") {
        if let Some(title) = metadata.get("title") {
            let slug = generate_slug(title);
            metadata.insert("slug".to_string(), slug);
        }
    }
}

fn generate_slug(title: &str) -> String {
    // Implement slug generation logic here
    // This is a simple placeholder implementation
    title.to_lowercase().replace(' ', "-")
}

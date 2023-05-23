use ssg::compiler::compile;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let src_dir = Path::new("examples/kaishi.one/content");
    let out_dir = Path::new("examples/kaishi.one/public");
    let site_name = String::from("kaishi.one");
    let binding = String::from("examples/kaishi.one/templates");
    let template_path = Some(&binding);
    compile(src_dir, out_dir, template_path, site_name)?;

    Ok(())
}

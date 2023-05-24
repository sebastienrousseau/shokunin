use ssg::compiler::compile;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let src_dir = Path::new("examples/pain001.com/content");
    let out_dir = Path::new("examples/pain001.com/public");
    let site_name = String::from("pain001.com");
    let binding = String::from("examples/pain001.com/templates");
    let template_path = Some(&binding);
    compile(
        src_dir,
        out_dir,
        template_path.map(|x| x.as_str()),
        &site_name,
    )?;

    Ok(())
}

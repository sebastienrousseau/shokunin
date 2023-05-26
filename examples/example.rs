use ssg::compiler::compile;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Define the paths to the build, site, source and template directories.
    let build_path = Path::new("examples/example.com/build");
    let site_path = Path::new("examples/example.com/public");
    let content_path = Path::new("examples/example.com/contents");
    let template_path = Path::new("examples/example.com/templates");

    compile(build_path, content_path, site_path, template_path)?;

    Ok(())
}

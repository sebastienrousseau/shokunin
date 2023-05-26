use ssg::compiler::compile;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Define the paths to the build, site, source and template directories.
    let build_path = Path::new("examples/shokunin.one/build");
    let site_path = Path::new("examples/shokunin.one/public");
    let content_path = Path::new("examples/shokunin.one/contents");
    let template_path = Path::new("examples/shokunin.one/templates");

    compile(build_path, content_path, site_path, template_path)?;

    Ok(())
}

/// This is the main entry point for the shokunin application.
fn main() {
    // Call the `run()` function from the `shokunin` module.
    if let Err(err) = ssg::run() {
        eprintln!("Error running shokunin (ssg): {}", err);
        std::process::exit(1);
    }
}

/// ## Function: `main` - The main function of the program
///
/// The main function of the program. Calls the `run()` function from
/// the `ssg` module to run the static site generator.
///
/// If an error occurs while running the `run()` function, the function
/// prints an error message to standard error and exits the program with
/// a status code of 1.
///
fn main() {
    // Call the `run()` function from the `shokunin` module.
    if let Err(err) = ssg::run() {
        eprintln!("Error running shokunin (ssg): {}", err);
        std::process::exit(1);
    }
}

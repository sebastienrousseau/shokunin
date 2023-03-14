use clap::ArgMatches;

use crate::create_new_project;

/// Processes the command line arguments provided to the program.
///
/// # Arguments
///
/// * `matches` - An instance of `clap::ArgMatches` containing the parsed command line arguments.
///
pub fn process_arguments(matches: &ArgMatches) {
    if let Some(arg_new) = matches.get_one::<String>("new") {
        println!("📝 Creating a new project... {}", arg_new);
        let new_project = create_new_project();
        match new_project {
            Ok(_) => println!("✅ Done."),
            Err(e) => println!("❌ Error: {}", e),
        }
    } else {
        println!("❌ No arguments provided. Please provide the required arguments to generate your site.");
    }
}

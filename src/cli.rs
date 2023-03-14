use clap::{Arg, ArgMatches, Command, Error};

/// Builds and returns a set of command-line arguments using the Clap
/// library.
///
/// # Arguments
///
/// None
///
/// # Returns
///
/// * `Result<ArgMatches, Error>` - A struct containing the parsed
/// command-line arguments and their values, or an error if the
/// arguments could not be parsed.
///
/// # Examples
///
/// ```
/// use ssg::cli;
/// let matches = cli::build_cli().unwrap();
/// ```
pub fn build_cli() -> Result<ArgMatches, Error> {
    let matches = Command::new("Shokunin (職人) 🦀")
        .author("Sebastien Rousseau")
        .about("")
        .version("0.0.1")
        .arg(
            Arg::new("new")
                .help("Create a new project.")
                .long("new")
                .short('n')
                .value_name("NEW"),
        )
        .after_help(
            "\x1b[1;4mDocumentation:\x1b[0m\n\n  https://shokunin.one\n\n\x1b[1;4mLicense:\x1b[0m\n  The project is licensed under the terms of both the MIT license and the Apache License (Version 2.0).",
        )
        .get_matches();

    Ok(matches)
}

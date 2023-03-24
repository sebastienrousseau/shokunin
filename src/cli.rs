use clap::{Arg, ArgMatches, Command, Error};

/// ## Function: build - returns a Result containing the parsed input options
///
/// Builds a command-line interface (CLI) using the `clap` crate and
/// returns the command-line arguments passed to the CLI as an
/// `ArgMatches` object.
///
/// This function creates a CLI, and adds the following arguments to it:
///
/// - `--new` or `-n`: Creates a new project.
/// - `--content` or `-c`: Specifies the location of the content directory.
/// - `--output` or `-o`: Specifies the location of the output directory.
///
/// If the CLI is successfully built and the command-line arguments are
/// parsed correctly, the function returns an `Ok` result containing the
/// `ArgMatches` object. If an error occurs while building or parsing
/// the CLI, an `Err` result containing the error message is returned.
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
/// let matches = cli::build().unwrap();
///
/// ```
pub fn build() -> Result<ArgMatches, Error> {
    let matches = Command::new("Shokunin (職人) 🦀")
        .author("Sebastien Rousseau")
        .about("")
        .version("0.0.8")
        .arg(
            Arg::new("new")
                .help("Create a new project.")
                .long("new")
                .short('n')
                .value_name("NEW"),
        )
        .arg(
            Arg::new("content")
                .help("Location of the content directory.")
                .long("content")
                .short('c')
                .value_name("CONTENT"),
        )
        .arg(
            Arg::new("output")
                .help("Location of the output directory.")
                .long("output")
                .short('o')
                .value_name("OUTPUT"),
        )
        .arg(
            Arg::new("template")
                .help("Location of the template directory.")
                .long("template")
                .short('t')
                .value_name("TEMPLATE"),
        )

        .after_help(
            "\x1b[1;4mDocumentation:\x1b[0m\n\n  https://shokunin.one\n\n\x1b[1;4mLicense:\x1b[0m\n  The project is licensed under the terms of both the MIT license and the Apache License (Version 2.0).",
        )
        .get_matches();

    // println!("Matches: {:?}", matches);

    Ok(matches)
}

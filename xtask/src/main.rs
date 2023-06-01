//! xtaskops entrypointFunction: `main` - Main function for xtaskops
fn main() -> Result<(), anyhow::Error> {
    // Run the xtaskops main function
    xtaskops::tasks::main()
}

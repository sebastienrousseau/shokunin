// Copyright © 2024 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

// #[cfg(test)]
// mod tests {
//     use staticrux::cli::build;

//     #[test]
//     // Test that the arguments for the build CLI are correctly set
//     fn test_build_args() {
//         // Define the expected argument values
//         let arg_specs = [
//             ("content", None),
//             ("output", None),
//             ("help", None),
//             ("version", None),
//         ];

//         // Call the build function to get the command-line arguments
//         let args = build().unwrap();

//         // Iterate through the expected argument values
//         for (arg_name, expected_value) in arg_specs.iter() {
//             // Get the actual value for the argument
//             let arg_value: Option<&String> = args.get_one(arg_name);

//             // Compare the actual and expected values
//             assert_eq!(arg_value, *expected_value);
//         }
//     }
// }

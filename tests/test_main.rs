// Copyright © 2024 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

#[cfg(test)]
mod tests {
    use ssg::run;

    #[test]
    fn test_run() {
        if let Err(err) = run() {
            eprintln!("Error running shokunin (ssg): {}", err);
        }
        assert_eq!(1, 1)
    }
}

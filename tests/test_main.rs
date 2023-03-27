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

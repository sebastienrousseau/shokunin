#[cfg(test)]
mod tests {

    #[test]
    fn test_shokunin() {
        let shokunin = ssg::new();
        assert_eq!(shokunin, ssg::default());
    }
}

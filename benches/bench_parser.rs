use clap::{Arg, Command};
use criterion::{
    black_box, criterion_group, criterion_main, Criterion,
};
use ssg::parser::args;

#[cfg(test)]
pub fn criterion_benchmark(c: &mut Criterion) {
    // Test required arguments present
    let matches = Command::new("test")
        .arg(Arg::new("new"))
        .arg(Arg::new("content"))
        .arg(Arg::new("output"))
        .get_matches_from(vec!["test_name", "test_content", "output"]);

    c.bench_function("parse command line arguments", |b| {
        b.iter(|| {
            let result = args(black_box(&matches));
            assert!(result.is_ok());
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

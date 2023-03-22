use criterion::{
    black_box, criterion_group, criterion_main, Criterion,
};
use ssg::file::add;
use std::path::PathBuf;
#[cfg(test)]
fn add_benchmark(c: &mut Criterion) {
    let path = PathBuf::from("path/to/directory");
    c.bench_function("add function", |b| {
        b.iter(|| {
            let result = add(black_box(&path));
            assert!(result.is_ok());
        })
    });
}

criterion_group!(benches, add_benchmark);
criterion_main!(benches);

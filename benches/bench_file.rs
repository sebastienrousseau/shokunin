use criterion::{
    black_box, criterion_group, criterion_main, Criterion,
};
use ssg::file::add;
use std::path::PathBuf;

#[cfg(test)]
fn add_benchmark(c: &mut Criterion) {
    let path = PathBuf::from("content");
    c.bench_function("add function", |b| {
        b.iter(|| {
            let result = add(&path);
            if let Err(e) = result {
                eprintln!("Error: {}", e);
            } else {
                black_box(result.unwrap());
            }
        })
    });
}

criterion_group!(benches, add_benchmark);
criterion_main!(benches);

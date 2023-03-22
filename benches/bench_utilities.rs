use criterion::{
    black_box, criterion_group, criterion_main, Criterion,
};
use ssg::utilities::directory;
use tempfile::TempDir;

fn directory_benchmark(c: &mut Criterion) {
    let tempdir = TempDir::new().unwrap();
    let dir = tempdir.path().join("test_dir");

    c.bench_function("create directory", |b| {
        b.iter(|| {
            let result =
                directory(black_box(&dir), black_box("test_dir"));
            assert!(result.is_ok());
        })
    });

    c.bench_function("check if directory exists", |b| {
        b.iter(|| {
            let result = dir.exists();
            assert!(result);
        })
    });

    c.bench_function("check if directory is a directory", |b| {
        b.iter(|| {
            let result = dir.is_dir();
            assert!(result);
        })
    });

    c.bench_function(
        "check if non-existent directory does not exist",
        |b| {
            let non_existent_dir = tempdir.path().join("non-existent");
            b.iter(|| {
                let result = non_existent_dir.exists();
                assert!(!result);
            })
        },
    );
}

criterion_group!(benches, directory_benchmark);
criterion_main!(benches);

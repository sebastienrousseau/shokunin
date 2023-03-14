extern crate criterion;

use criterion::{criterion_group, criterion_main, Criterion};
use ssg::run;

fn shokunin_benchmark(c: &mut Criterion) {
    c.bench_function("shokunin", |b| {
        b.iter(|| {
            for _ in 0..1000 {
                match run() {
                    Ok(_) => {}
                    Err(e) => {
                        eprintln!("Error running shokunin: {:?}", e);
                    }
                }
            }
        })
    });
}

criterion_group!(benches, shokunin_benchmark);
criterion_main!(benches);

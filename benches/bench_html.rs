use criterion::{
    black_box, criterion_group, criterion_main, Criterion,
};
use ssg::html::generate_html;

#[cfg(test)]
pub fn criterion_benchmark(c: &mut Criterion) {
    let content = "---\n\
                   title: My Title\n\
                   description: My Description\n\
                   keywords: foo, bar, baz\n\
                   permalink: /my-permalink\n\
                   ---\n\
                   My content";
    c.bench_function("generate HTML", |b| {
        b.iter(|| {
            let result = generate_html(
                black_box(content),
                black_box("My Title"),
                black_box("My Description"),
            );
            assert!(result.contains("<h1>My Title</h1>"));
            assert!(result.contains("<h2>My Description</h2>"));
            assert!(result.contains("<p>My content</p>"));
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

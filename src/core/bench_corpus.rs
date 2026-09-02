// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Deterministic synthetic corpora for benchmarking.
//!
//! Every benchmark that needed pages used to generate them inline, so the
//! numbers were only comparable within a single bench file: `bench_scalability`
//! and `incremental_1000_pages` wrote different front matter and different body
//! lengths, then reported timings as though they measured the same work. A
//! published figure has to be reproducible by whoever reads it, which means one
//! generator, one shape, and no hidden inputs.
//!
//! # Determinism
//!
//! Content is derived from a seed and the page index through a small
//! [xorshift] generator rather than the `rand` crate: the corpus must be
//! byte-identical across machines, architectures and toolchain versions, and
//! `rand`'s output is explicitly not stable across releases. The same seed
//! therefore yields the same corpus in a year's time, which is the property
//! that lets a benchmark number be checked rather than believed.
//!
//! Body length and tag selection vary per page — a corpus of identical pages
//! measures the cache, not the compiler — but vary *predictably*.
//!
//! [xorshift]: https://en.wikipedia.org/wiki/Xorshift
//!
//! # Examples
//!
//! ```rust
//! use ssg::bench_corpus::{generate_corpus, CorpusSpec};
//! let dir = tempfile::tempdir().unwrap();
//! let spec = CorpusSpec::new(64);
//! let written = generate_corpus(dir.path(), &spec).unwrap();
//! assert_eq!(written, 64);
//! ```

use std::fs;
use std::io;
use std::path::Path;

/// Words drawn on to build page bodies.
///
/// A fixed vocabulary keeps the compressed size of the corpus stable, which
/// matters because the page-weight gate measures compressed bytes.
const LEXICON: &[&str] = &[
    "compiler",
    "pipeline",
    "markdown",
    "template",
    "static",
    "render",
    "accessible",
    "contrast",
    "locale",
    "sitemap",
    "canonical",
    "manifest",
    "fingerprint",
    "integrity",
    "streaming",
    "incremental",
    "corpus",
    "benchmark",
    "throughput",
    "latency",
    "deterministic",
    "artefact",
];

/// Tags assigned to pages, so taxonomy generation has real work to do.
const TAGS: &[&str] = &[
    "architecture",
    "performance",
    "accessibility",
    "security",
    "tooling",
];

/// Shape of a synthetic corpus.
///
/// # Examples
///
/// ```rust
/// use ssg::bench_corpus::CorpusSpec;
/// // The published sizes: 1K, 10K, 100K.
/// let spec = CorpusSpec::new(1_000);
/// assert_eq!(spec.pages, 1_000);
/// assert_eq!(spec.seed, CorpusSpec::DEFAULT_SEED);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CorpusSpec {
    /// Number of Markdown pages to write.
    pub pages: usize,
    /// Seed for the content generator. Fixed by default so two runs of the
    /// same size produce byte-identical input.
    pub seed: u64,
    /// Approximate words per page body.
    pub words_per_page: usize,
}

impl CorpusSpec {
    /// The seed used for published figures. Changing it invalidates
    /// comparison with every previously published number, so it is a
    /// constant rather than a parameter with a default.
    pub const DEFAULT_SEED: u64 = 0x5353_4720_4265_6e63; // "SSG Benc"

    /// A corpus of `pages` pages at the published seed and body length.
    #[must_use]
    pub const fn new(pages: usize) -> Self {
        Self {
            pages,
            seed: Self::DEFAULT_SEED,
            words_per_page: 220,
        }
    }

    /// Overrides the seed, for tests that need two distinguishable corpora.
    #[must_use]
    pub const fn with_seed(mut self, seed: u64) -> Self {
        self.seed = seed;
        self
    }
}

/// A tiny xorshift64* PRNG.
///
/// Chosen over `rand` because the corpus must be reproducible across releases;
/// `rand` makes no such guarantee, and a benchmark whose input silently changes
/// with a dependency bump reports drift as a regression.
struct Rng(u64);

impl Rng {
    const fn new(seed: u64) -> Self {
        // A zero state is a fixed point for xorshift, so it is never allowed.
        Self(if seed == 0 {
            0x9E37_79B9_7F4A_7C15
        } else {
            seed
        })
    }

    const fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    const fn pick<'a, T>(&mut self, items: &'a [T]) -> &'a T {
        let idx = (self.next() % items.len() as u64) as usize;
        &items[idx]
    }
}

/// Writes `spec.pages` Markdown files into `dir`, returning the count written.
///
/// The directory is created if absent. Existing files are overwritten, so a
/// re-run refreshes the corpus in place rather than accumulating.
///
/// # Errors
///
/// Returns any I/O error from creating the directory or writing a page.
///
/// # Examples
///
/// ```rust
/// use ssg::bench_corpus::{generate_corpus, CorpusSpec};
/// let dir = tempfile::tempdir().unwrap();
/// generate_corpus(dir.path(), &CorpusSpec::new(4)).unwrap();
/// assert!(dir.path().join("page-0000.md").is_file());
/// ```
pub fn generate_corpus(dir: &Path, spec: &CorpusSpec) -> io::Result<usize> {
    fs::create_dir_all(dir)?;

    for i in 0..spec.pages {
        // Seeding per page rather than streaming one sequence means page N is
        // identical whether the corpus holds 1K pages or 100K — so the 1K and
        // 10K runs share a prefix and are genuinely comparable.
        let mut rng =
            Rng::new(spec.seed ^ (i as u64).wrapping_mul(0x9E37_79B9));

        let words: Vec<&str> = (0..spec.words_per_page)
            .map(|_| *rng.pick(LEXICON))
            .collect();
        let tag_a = rng.pick(TAGS);
        let tag_b = rng.pick(TAGS);

        // Two paragraphs and a heading, so the Markdown parser and the HTML
        // rewriter both see structure rather than one long text run.
        let half = words.len() / 2;
        let body = format!(
            "## Section {i}\n\n{}\n\n### Detail\n\n{}\n",
            words[..half].join(" "),
            words[half..].join(" "),
        );

        let page = format!(
            "---\n\
             title: \"Benchmark page {i}\"\n\
             description: \"Synthetic page {i} from the SSG benchmark corpus.\"\n\
             date: \"2026-01-15T09:00:00+00:00\"\n\
             language: \"en-GB\"\n\
             layout: \"page\"\n\
             permalink: \"https://example.com/page-{i}\"\n\
             author: \"bench@example.com\"\n\
             tags: \"{tag_a}, {tag_b}\"\n\
             ---\n\n{body}"
        );

        // Zero-padded so lexical order matches numeric order; a directory
        // listing that jumps 1, 10, 100 makes a partial run hard to read.
        fs::write(dir.join(format!("page-{i:04}.md")), page)?;
    }

    Ok(spec.pages)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn corpus_is_byte_identical_across_runs() {
        // The whole point: a published number is only checkable if the input
        // can be regenerated exactly.
        let a = tempfile::tempdir().unwrap();
        let b = tempfile::tempdir().unwrap();
        let spec = CorpusSpec::new(16);

        let _written = generate_corpus(a.path(), &spec).unwrap();
        let _written = generate_corpus(b.path(), &spec).unwrap();

        for i in 0..16 {
            let name = format!("page-{i:04}.md");
            assert_eq!(
                fs::read(a.path().join(&name)).unwrap(),
                fs::read(b.path().join(&name)).unwrap(),
                "{name} differs between runs"
            );
        }
    }

    #[test]
    fn page_content_does_not_depend_on_corpus_size() {
        // A 1K run and a 10K run must share their first 1K pages, or the two
        // published figures measure different inputs.
        let small = tempfile::tempdir().unwrap();
        let large = tempfile::tempdir().unwrap();
        let _written =
            generate_corpus(small.path(), &CorpusSpec::new(8)).unwrap();
        let _written =
            generate_corpus(large.path(), &CorpusSpec::new(64)).unwrap();

        for i in 0..8 {
            let name = format!("page-{i:04}.md");
            assert_eq!(
                fs::read(small.path().join(&name)).unwrap(),
                fs::read(large.path().join(&name)).unwrap(),
                "{name} differs between corpus sizes"
            );
        }
    }

    #[test]
    fn different_seeds_produce_different_corpora() {
        let a = tempfile::tempdir().unwrap();
        let b = tempfile::tempdir().unwrap();
        let _written = generate_corpus(a.path(), &CorpusSpec::new(8)).unwrap();
        let _written =
            generate_corpus(b.path(), &CorpusSpec::new(8).with_seed(1))
                .unwrap();

        let name = "page-0000.md";
        assert_ne!(
            fs::read(a.path().join(name)).unwrap(),
            fs::read(b.path().join(name)).unwrap()
        );
    }

    #[test]
    fn pages_carry_frontmatter_and_structure() {
        let dir = tempfile::tempdir().unwrap();
        let _written =
            generate_corpus(dir.path(), &CorpusSpec::new(1)).unwrap();
        let page = fs::read_to_string(dir.path().join("page-0000.md")).unwrap();

        assert!(page.starts_with("---\n"), "missing front matter");
        assert!(page.contains("permalink: \"https://example.com/page-0\""));
        assert!(page.contains("tags: \""), "taxonomy needs tags");
        assert!(page.contains("## Section 0"), "missing heading");
        assert!(page.contains("### Detail"), "missing subheading");
    }

    #[test]
    fn zero_seed_does_not_collapse_the_generator() {
        // Xorshift has a fixed point at zero; an unguarded seed of 0 would
        // emit the same word for every position.
        let dir = tempfile::tempdir().unwrap();
        let _written =
            generate_corpus(dir.path(), &CorpusSpec::new(1).with_seed(0))
                .unwrap();
        let page = fs::read_to_string(dir.path().join("page-0000.md")).unwrap();

        let body: Vec<&str> = page
            .split("---\n")
            .nth(2)
            .unwrap_or_default()
            .split_whitespace()
            .collect();
        let distinct: std::collections::BTreeSet<_> = body.iter().collect();
        assert!(
            distinct.len() > 5,
            "seed 0 collapsed the generator: {} distinct words",
            distinct.len()
        );
    }
}

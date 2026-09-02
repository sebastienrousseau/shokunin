//! Guards the two ways the benchmark wiring fails without saying so.
//!
//! `benches/bench.rs` is an umbrella harness: it `mod`s the per-area bench
//! files and lists their Criterion groups in a single `criterion_main!`.
//! That arrangement has two failure modes, and both are silent.
//!
//! # Trap 1 — a `[[bench]]` section for a file the umbrella already owns
//!
//! `Cargo.toml` says so in prose: such a section "would collide with
//! bench.rs's `criterion_main!`". Prose in a manifest is not a gate, and the
//! collision surfaces as a confusing link error rather than a clear one.
//!
//! # Trap 2 — `required-features` on a target CI runs without those features
//!
//! This is the worse of the two, because it does not fail at all. Cargo
//! *skips* a bench target whose `required-features` are not enabled, silently
//! and with a zero exit code. `scheduled.yml` runs
//! `cargo bench --bench bench -- scalability` and `make bench` runs
//! `cargo bench --bench bench`, neither with extra features — so putting
//! `required-features` on the umbrella would stop the scheduled scalability
//! run happening, and nothing anywhere would go red.
//!
//! A benchmark that stops running looks exactly like a benchmark that is
//! passing.

use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn manifest() -> String {
    fs::read_to_string(root().join("Cargo.toml")).expect("Cargo.toml readable")
}

/// Bench files the umbrella harness compiles into itself.
fn umbrella_modules() -> BTreeSet<String> {
    let src = fs::read_to_string(root().join("benches/bench.rs"))
        .expect("benches/bench.rs readable");
    src.lines()
        .filter_map(|l| {
            l.trim()
                .strip_prefix("mod ")
                .and_then(|r| r.strip_suffix(';'))
                .map(str::to_string)
        })
        .collect()
}

/// `[[bench]]` sections as `(name, has_required_features)`.
fn declared_benches() -> Vec<(String, bool)> {
    let m = manifest();
    let mut out = Vec::new();
    let mut in_block = false;
    let mut name: Option<String> = None;
    let mut req = false;

    let flush = |out: &mut Vec<(String, bool)>,
                 name: &mut Option<String>,
                 req: &mut bool| {
        if let Some(n) = name.take() {
            out.push((n, *req));
        }
        *req = false;
    };

    for line in m.lines() {
        let t = line.trim();
        if t == "[[bench]]" {
            flush(&mut out, &mut name, &mut req);
            in_block = true;
            continue;
        }
        if t.starts_with('[') && t != "[[bench]]" {
            flush(&mut out, &mut name, &mut req);
            in_block = false;
            continue;
        }
        if in_block {
            if let Some(n) = t
                .strip_prefix("name = \"")
                .and_then(|r| r.strip_suffix('"'))
            {
                name = Some(n.to_string());
            }
            if t.starts_with("required-features") {
                req = true;
            }
        }
    }
    flush(&mut out, &mut name, &mut req);
    out
}

#[test]
fn no_bench_section_collides_with_the_umbrella_harness() {
    let umbrella = umbrella_modules();
    assert!(
        !umbrella.is_empty(),
        "parsed no `mod` lines from benches/bench.rs"
    );

    let colliding: Vec<String> = declared_benches()
        .into_iter()
        .map(|(n, _)| n)
        .filter(|n| umbrella.contains(n))
        .collect();

    assert!(
        colliding.is_empty(),
        "these files are compiled into benches/bench.rs and must not also have \
         their own [[bench]] section — the two `criterion_main!`s collide:\n  {}",
        colliding.join("\n  ")
    );
}

#[test]
fn benches_ci_runs_unconditionally_have_no_required_features() {
    // Targets invoked without extra features anywhere in the repo. Cargo
    // skips a target whose required-features are unmet *without failing*, so
    // adding them here would silently retire the run.
    let mut invoked_plainly = BTreeSet::new();

    let mut sources =
        vec![fs::read_to_string(root().join("Makefile")).unwrap_or_default()];
    if let Ok(dir) = fs::read_dir(root().join(".github/workflows")) {
        for e in dir.flatten() {
            if e.path().extension().is_some_and(|x| x == "yml") {
                sources.push(fs::read_to_string(e.path()).unwrap_or_default());
            }
        }
    }

    for src in &sources {
        for line in src.lines() {
            if !line.contains("cargo bench") || line.contains("--features") {
                continue;
            }
            // `--bench <name>` names the target; a bare `cargo bench` runs all.
            if let Some(rest) = line.split("--bench ").nth(1) {
                if let Some(name) = rest.split_whitespace().next() {
                    let _ = invoked_plainly.insert(name.to_string());
                }
            }
        }
    }

    assert!(
        !invoked_plainly.is_empty(),
        "no plain `cargo bench --bench <name>` invocation found; this test \
         assumes at least one exists (scheduled.yml and the Makefile both had \
         one when it was written)"
    );

    let broken: Vec<String> = declared_benches()
        .into_iter()
        .filter(|(n, req)| *req && invoked_plainly.contains(n))
        .map(|(n, _)| n)
        .collect();

    assert!(
        broken.is_empty(),
        "these bench targets declare `required-features` but are invoked \
         without them:\n  {}\n\
         Cargo skips such a target silently, so the benchmark would stop \
         running and nothing would fail.",
        broken.join("\n  ")
    );
}

#[test]
fn every_umbrella_module_has_a_matching_bench_file() {
    for m in umbrella_modules() {
        let path = root().join(format!("benches/{m}.rs"));
        assert!(
            path.is_file(),
            "benches/bench.rs declares `mod {m};` but {} does not exist",
            path.display()
        );
    }
}

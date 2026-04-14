<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# `docs/`

This directory is a **build target**, not a source tree.

It is listed in the repository's `.gitignore` (`/docs/`) so anything
written here is local to your workstation and never committed.

## What lives here

- Output of `cargo doc --no-deps --open`, mirrored when the
  `docs.yml` workflow runs in CI on `main`.
- Generated artefacts from any of the bundled examples that point at
  `./docs/` as their `--output` flag.

## Where to find the published docs

- **API reference:** https://docs.rs/ssg
- **Source of the landing page:** the crate root uses
  `#![doc = include_str!("../README.md")]` in `src/lib.rs`, so the
  project's top-level `README.md` *is* the docs.rs landing page. Edit
  the README, not a separate landing file, when changing the public
  documentation.

## Regenerating locally

```bash
cargo doc --no-deps --open
```

That command writes to `target/doc/`, **not** here. The `docs/`
directory exists only for ad-hoc generators that need a stable
relative path.

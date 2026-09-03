<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# `docs/`

This directory is **both** a build target and a source tree, which is
the thing to understand before adding a file here.

`.gitignore` denies the whole directory with `/docs/*` and then
re-admits the committed subtrees one by one:

```gitignore
/docs/*
!/docs/README.md
!/docs/ARCHITECTURE.md
!/docs/adr/
...
```

**A new file or directory under `docs/` is invisible to git until it is
added to that allowlist**, and nothing tells you: `git status` does not
list ignored files, so the working tree looks correct and clean while the
content is absent from every commit. Renaming a listed directory has the
same effect, because the negation stops matching — that is how a
`docs/adrs` → `docs/adr` rename once produced a commit that deleted nine
ADRs and added none back, on a branch whose tooling all reported green.

So when you add something here, add its allowlist entry in the same
change and verify with the index rather than the filesystem:

```sh
git add --dry-run docs/<new-path>      # must print "add '<path>'"
git ls-tree -r HEAD -- docs/           # must list it after committing
```

## What lives here

- Output of `cargo doc --no-deps --open`, mirrored when the
  `docs.yml` workflow runs in CI on `main`.
- Generated artefacts from any of the bundled examples that point at
  `./docs/` as their `--output` flag.

## Where to find the published docs

- **API reference:** <https://docs.rs/ssg>
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

<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# Security Policy

## Reporting a Vulnerability

If you have discovered a security vulnerability in SSG, **do not open a
public issue**. Email the maintainer at
[sebastian.rousseau@gmail.com](mailto:sebastian.rousseau@gmail.com)
with:

- A description of the vulnerability and its impact
- Steps to reproduce
- The version of SSG affected
- Any proof-of-concept code (gist or attachment, never inline)

You should receive an acknowledgement within 72 hours. Disclosure
follows the coordinated disclosure principle — a fix is developed and
released before details are made public, and you receive credit in
the advisory unless you prefer otherwise.

## Supported Versions

| Version | Supported |
|---|---|
| 0.0.x (latest) | ✅ Security fixes |
| < 0.0.30 | ❌ Upgrade required |

`0.0.x` is pre-1.0; only the latest minor version receives security
patches. Once `1.0.0` ships, the supported range will widen per the
SemVer-aligned policy in `CHANGELOG.md`.

## Security Posture

SSG is a build-time tool that processes untrusted input (Markdown,
YAML frontmatter, templates) into static HTML. The threat model
focuses on:

- **Local privilege elevation** — a malicious site source must not
  exfiltrate secrets from the build environment or escape the output
  directory via path traversal.
- **Supply-chain integrity** — every dependency is pinned and audited
  via `cargo deny check` on every CI run; GitHub Actions are pinned
  to commit SHAs (not tags).
- **Output integrity** — generated HTML must not echo unsanitised
  user content into HTML attributes or JavaScript contexts.

Out of scope:

- Runtime attacks against deployed static sites (those depend on the
  hosting provider's controls).
- Cryptographic identity of the operator (key management is delegated
  to git signing and GPG-signed releases).

## Security-Relevant Defaults

| Control | Default | Configuration |
|---|---|---|
| `unsafe` Rust | `#![forbid(unsafe_code)]` at every crate root | not configurable |
| Path traversal | `is_safe_path()` checks every output path against site root | not configurable |
| CSP | Inline `<style>`/`<script>` extracted to external files with SRI hashes; `unsafe-inline` not emitted | `CspPlugin` |
| HSTS | `Strict-Transport-Security: max-age=31536000` in deploy configs | `DeployPlugin` |
| Dependency advisories | `cargo deny check` runs on every CI build; allow-list is documented in `deny.toml` with rationale | `deny.toml` |
| Action pinning | All GitHub Actions pinned by 40-char commit SHA, not tag/branch | workflow files |

## Reproducible Builds

CI verifies that `cargo build --release --locked -p ssg` produces a
byte-identical binary across two consecutive runs on the same commit.
The job is in
[`.github/workflows/scheduled.yml`](.github/workflows/scheduled.yml)
under the `reproducible` job and runs weekly + on tag push.

The verification process:

1. Both builds set `SOURCE_DATE_EPOCH=1700000000` to pin
   timestamp-derived metadata.
2. `RUSTFLAGS="--remap-path-prefix=${GITHUB_WORKSPACE}=/build"` strips
   absolute checkout paths from the binary.
3. `cargo build --release --locked -p ssg` runs twice, with
   `cargo clean -p ssg` between runs.
4. SHA-256 of `target/release/ssg` is captured after each build.
5. The job fails if the two hashes differ.

To verify locally:

```sh
SOURCE_DATE_EPOCH=1700000000 \
  RUSTFLAGS="--remap-path-prefix=$(pwd)=/build" \
  cargo build --release --locked -p ssg
sha256sum target/release/ssg
cargo clean -p ssg
SOURCE_DATE_EPOCH=1700000000 \
  RUSTFLAGS="--remap-path-prefix=$(pwd)=/build" \
  cargo build --release --locked -p ssg
sha256sum target/release/ssg
```

The two sums must match.

### Non-determinism sources investigated

If the CI job fails after a change to the build, candidate causes:

- **Embedded paths** — anything that reads `file!()` or `env!("CARGO_MANIFEST_DIR")`
  into the binary; mitigation is `--remap-path-prefix`.
- **`build.rs` non-determinism** — generators that read clock, hostname,
  or randomness; check `build.rs` for `SystemTime::now()` or
  `gethostname` use.
- **Parallel codegen ordering** — `codegen-units > 1` with non-frozen
  symbol ordering. Release profile already pins `codegen-units = 1`.
- **Dependency churn** — `cargo build --locked` enforces the lockfile;
  if a transitive crate changed between the two runs, the cache was
  poisoned.
- **`include_bytes!` of mutable files** — anything generated outside
  the lockfile's reach.

The reproducible-build hash is **not** currently part of the release
attestation — that step is tracked in #424's deferred follow-up. Once
the verification has been green for ≥ 4 consecutive weekly runs, the
hash will be embedded in the SLSA provenance attestation alongside
the existing CycloneDX SBOM.

## Build Provenance

Every release tag triggers
[`.github/workflows/release.yml`](.github/workflows/release.yml),
which produces:

- Multi-platform binaries (Linux glibc + musl, macOS arm64 + x86_64,
  Windows MSVC) with SHA-256 checksums.
- Detached GPG signatures (when `GPG_PRIVATE_KEY` secret is configured).
- A SLSA build provenance attestation via
  `actions/attest-build-provenance` (signed via Sigstore).
- A CycloneDX SBOM (`scheduled.yml` `sbom` job).

To verify a release binary:

```sh
# Verify checksum
sha256sum -c ssg-vX.Y.Z-x86_64-unknown-linux-gnu.tar.gz.sha256

# Verify GPG signature
gpg --verify ssg-vX.Y.Z-x86_64-unknown-linux-gnu.tar.gz.asc \
             ssg-vX.Y.Z-x86_64-unknown-linux-gnu.tar.gz

# Verify Sigstore provenance
gh attestation verify ssg-vX.Y.Z-x86_64-unknown-linux-gnu.tar.gz \
                      --owner sebastienrousseau
```

## Cryptographic Material

SSG itself does not generate or store cryptographic keys at runtime.
The release pipeline uses:

- **GPG** — for detached signatures over release artifacts. The public
  key is published at the project URL and rotated per the standard
  GPG hygiene cycle.
- **Sigstore** — for keyless build provenance attestation; uses the
  GitHub OIDC token at workflow runtime.
- **SHA-256** — for content-integrity checksums.
- **SHA-384** — for Subresource Integrity hashes embedded in HTML
  by `CspPlugin`.

No long-term secrets are stored in the repository.

## Hardening Roadmap

Tracked in the milestone "Compliance, Security & Observability":

- **Reproducible builds across all release platforms** — currently
  Linux-only; macOS and Windows reproducibility is platform-dependent
  and tracked separately.
- **Post-quantum content provenance** — embedding ML-DSA (FIPS 204)
  signatures in `<meta name="content-signature">` tags. Tracked in
  issue #420; needs a design RFC before implementation.
- **Transitive dependency rotation** — six unmaintained crates remain
  in the allow-list (`yaml-rust`, `paste`, `fxhash`, `number_prefix`,
  `bincode`, `rand 0.8.5`). Tracked in issue #464.
- **OpenTelemetry build traces** — feature-gated observability for
  the plugin pipeline. Tracked in issue #422.

---

*Last reviewed: 2026-05-10. This file lives at repository root and
is the canonical security policy for the SSG project.*

<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# Supply-chain attestation

This directory holds the `cargo-vet` attestation state for the `ssg`
workspace. `cargo-vet` complements `cargo-deny` (license + CVE
checking) with **per-crate audit attestation**: every transitive
dependency must either be (1) audited by us, (2) imported from a
trust set we recognise, or (3) explicitly exempted with rationale.

## Files

| File | Purpose | Edited by |
|---|---|---|
| `audits.toml` | Our own audit certificates. Add an entry here when *we* have read the source of a crate version and certify it `safe-to-deploy` or `safe-to-run`. | `cargo vet certify` |
| `config.toml` | Workspace policy: which trust sets we import, per-crate policies, and the exemption list (crates we have not yet audited but accept for now). | `cargo vet init` / `cargo vet exemption add` |
| `imports.lock` | Pinned hashes of imported trust-set audit files, so we detect tampering on `cargo vet` runs. | `cargo vet` |

## Trust sets imported

We trust the following organisations' published audit decisions:

- **Mozilla Firefox** — `https://raw.githubusercontent.com/mozilla-firefox/firefox/main/supply-chain/audits.toml`
- **Bytecode Alliance (Wasmtime)** — `https://raw.githubusercontent.com/bytecodealliance/wasmtime/main/supply-chain/audits.toml`
- **Google** — `https://raw.githubusercontent.com/google/supply-chain/main/audits.toml`

Rationale: these organisations audit Rust crates as part of shipping
production code (Firefox, Wasmtime, ChromeOS) — their certificates
are reviewed by paid security teams. Adopting the union of their
trust sets transfers 90+ direct attestations to us at v0.0.45
bootstrap.

To refresh imported certificates: `cargo vet`. To rotate trust sets:
edit the `[imports.*]` block in `config.toml` and re-run `cargo vet`.

## Exemption policy

v0.0.45 bootstrap inherits **544 exemptions** — every transitive
crate not covered by an imported trust set is auto-exempted at
`safe-to-deploy` so `cargo vet` is green from day one. This is the
*starting line*, not the finish line.

Reduction path:
1. **Quarterly trust-set refresh.** Run `cargo vet` and review the
   delta — Mozilla/Google/Bytecodealliance add new audits monthly,
   so passive refresh drops our count steadily.
2. **Targeted audits for high-blast-radius crates.** When a crate
   appears in the build pipeline's critical path (parsers,
   network, FFI), we should audit it ourselves via
   `cargo vet certify <crate> <version>`.
3. **Drop entries on minor-bump churn.** Each crate version is its
   own exemption entry; bumping a dep often *adds* an entry rather
   than replacing one. Pruning is a chore: `cargo vet prune`.

Target trajectory: 544 → 350 by v0.1.0, → 100 by v1.0.0. Tracked as
a recurring v0.0.* milestone task (no per-release issue — the bar
moves down quarterly).

## CI integration

`.github/workflows/ci.yml` runs `cargo vet --locked` in a dedicated
`vet` job (parallel with `cargo-deny`). The build fails if:

- An imported trust set returns a tampered audit file (detected via
  `imports.lock`).
- A new transitive dep appears that is neither audited, trust-set
  vouched, nor exempted.
- A pinned audit file URL is unreachable.

Adding a new dep that triggers a vet failure: run
`cargo vet suggest` for guidance, then either `cargo vet certify` or
`cargo vet exemption add` with a rationale.

## See also

- `deny.toml` — license + CVE gating (`cargo deny check`)
- `docs/adrs/0006-cyclonedx-over-spdx.md` — SBOM format decision
- `SECURITY.md` — overall threat model and security defaults

<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# ADR-0006: CycloneDX 1.5 as the primary SBOM format

- **Date:** 2026-06-26
- **Status:** Accepted

## Context

Software Bills of Materials (SBOMs) are a regulatory requirement under
EU CRA (Cyber Resilience Act) Article 11, US Executive Order 14028,
and a procurement table-stake for any sale into financial services
under DORA.

Two formats dominate:

- **CycloneDX** (OWASP). Component-centric, optimised for vulnerability
  intelligence. Native VEX (Vulnerability Exploitability eXchange)
  support since 1.4. JSON, XML, and Protobuf encodings.
- **SPDX** (Linux Foundation). License-centric, optimised for IP
  compliance. Strong in legal-software-clearance workflows. Tag-value
  - JSON encodings.

The two are not mutually exclusive — CycloneDX components can be
SPDX-licensed, SPDX docs can reference CycloneDX VEX statements.
The question is which to emit *primarily* from `ssg build`, and which
(if any) to emit as a secondary artefact.

For the audiences `ssg` targets — Tier-1 enterprise and financial
services platform engineering — vulnerability response is the
dominant SBOM use case, not license clearance. The procurement teams
ingest SBOMs into Snyk, Sonatype Nexus IQ, Black Duck, or open-source
GUAC — all of which natively consume CycloneDX.

## Decision

**Primary SBOM format: CycloneDX 1.5 JSON.** Emitted as
`sbom.cdx.json` at the site root on every `ssg build`.

SPDX 3.0 emission is planned for v0.0.50 as a secondary artefact
(`sbom.spdx.json`) but not as the primary. Tracked as a follow-up to
v0.0.50 #580 (SLSA L3 migration).

VEX statements (CycloneDX 1.5 `vulnerabilities[]`) are included where
the build crate has applied a documented mitigation; otherwise
omitted to avoid false-positive churn in downstream scanners.

## Consequences

**Positive.**

- Native interop with Snyk, Sonatype, Black Duck, GUAC, Dependency-Track
  without conversion shims.
- VEX support — we can attach exploitability assertions to specific
  CVEs (e.g., "CVE-2024-XYZ in `foo` is unreachable from `ssg` because
  the affected code path is gated behind `feature = "bar"` which is
  off by default").
- JSON encoding is grep-able; one-line `jq` queries answer "which
  components are MIT-licensed?" or "which components include a VEX
  not_affected statement?"

**Negative.**

- License-compliance pipelines that prefer SPDX (notably the
  ScanCode Toolkit and FOSSology default flows) need a conversion
  step. We mitigate by emitting SPDX as a v0.0.50 follow-up.
- CycloneDX 1.5 → 1.6 schema migration is on the horizon (Q3 2026
  per OWASP); we will follow but not lead.

## Alternatives Considered

- **SPDX 2.3 as primary.** Rejected: VEX support is bolt-on (SPDX
  Security Profile is 3.0+); ingestion in vulnerability platforms is
  weaker.
- **Emit both with equal weight.** Rejected for v0.0.45 — duplicates
  the maintenance burden and forces a decision-by-default on
  downstream consumers about which to trust. Picks happen post-1.0.
- **Skip in-tree SBOM, rely on downstream tooling.** Rejected:
  procurement evidence requires SBOM provenance bound to the build,
  not generated externally after the fact.

## Status

Accepted. CycloneDX 1.5 emission already shipped via the v0.0.44
SBOM plugin (`src/plugins/sbom.rs`). SPDX 3.0 emission deferred to a
post-v0.0.50 minor.

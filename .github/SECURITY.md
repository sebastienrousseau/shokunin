<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# Security

We take the security of our software products and services seriously, which includes all source code repositories managed through our GitHub repositories.

## Contact Information

To report a security vulnerability, please use the following email address: <contact@static-site-generator.one>.

We accept reports in the following languages:English or French.

## Reporting Security Issues

When reporting a security issue, please include as much of the following information as possible to help us understand the nature and scope of the possible issue:

- Type of issue (e.g., buffer overflow, SQL injection, cross-site scripting, etc.)
- Full paths of source file(s) related to the manifestation of the issue
- The location of the affected source code (tag/branch/commit or direct URL)
- Any special configuration required to reproduce the issue
- Step-by-step instructions to reproduce the issue
- Proof-of-concept or exploit code (if possible)
- Impact of the issue, including how an attacker might exploit it
- This information will help us triage your report more quickly.

## Response Time

We aim to acknowledge receipt of your vulnerability report within 48 hours and will strive to keep you informed of the progress we're making toward resolving the issue.

## Disclosure Policy

Once we've resolved a reported security issue, we may disclose it publicly. We will coordinate the disclosure with the person who reported the issue to ensure that they are credited for their discovery.

## Acknowledgments

We will publicly thank security researchers who follow this responsible disclosure policy, recognizing their contributions in our 'Hall of Fame' or 'Thank You' page.

## Supply-Chain Verification

This project takes the following steps to protect its supply chain:

### Pinned CI dependencies

All GitHub Actions in `.github/workflows/` are pinned to full commit SHAs
rather than mutable tags. Each pinned reference includes a comment with the
human-readable tag for auditability (e.g.,
`actions/checkout@<sha> # v4`).

### SBOM generation

Every push to `main` and every release tag generates a
[CycloneDX](https://cyclonedx.org/) Software Bill of Materials (SBOM) via
`cargo-cyclonedx`. The SBOM is uploaded as a CI artifact and, on tagged
releases, attested with GitHub's built-in build provenance.

### How to verify a release

1. **Download the SBOM artifact** from the GitHub Actions run for the
   release tag (artifact name: `sbom-cyclonedx`).
2. **Verify attestation** using the GitHub CLI:
   ```bash
   gh attestation verify <sbom-file> --repo sebastienrousseau/static-site-generator
   ```
3. **Audit dependencies** locally:
   ```bash
   cargo install cargo-deny cargo-cyclonedx
   cargo deny check
   cargo cyclonedx --format json --output-cdx
   ```
4. **Compare SBOMs** -- diff the local SBOM against the CI-generated one
   to confirm they list the same dependency set.

### Dependency review

Pull requests are automatically scanned by `actions/dependency-review-action`
(via the shared `security.yml` pipeline) to flag new dependencies with
known vulnerabilities at the `moderate` severity threshold or above.

## Safe Harbour

We promise not to initiate legal action against researchers for disclosing vulnerabilities as long as they adhere to responsible disclosure guidelines, which includes reporting it to us and not publicly disclosing the issue until we've had a reasonable time to address it.

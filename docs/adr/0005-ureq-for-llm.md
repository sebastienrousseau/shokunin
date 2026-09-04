<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# ADR-0005: `ureq` for the local-LLM HTTP path

- **Date:** 2026-06-26
- **Status:** Accepted

## Context

`LlmPlugin` calls a local Ollama / llama.cpp endpoint over HTTP to
generate alt text, meta descriptions, and translations. The pre-v0.0.44
implementation shelled out to the host's `curl` binary via
`std::process::Command::new("curl")`. That approach failed three
constraints:

1. **Cross-platform.** Windows runners without `curl.exe` on PATH
   (a real CI scenario into 2024) broke silently.
2. **Shell-injection surface.** User-supplied prompts flowed into
   argv; while we escaped, the surface existed.
3. **Typed errors.** Network failures surfaced as stderr strings.
   Callers could not distinguish "endpoint down" from "endpoint slow"
   from "endpoint returned malformed JSON."

The fix was to use a real Rust HTTP client. The constraint set was:

- Synchronous, blocking — ADR-0001 forbids tokio, and the LLM call
  runs inside a Rayon worker that already owns its thread.
- Pure Rust, rustls-only — no OpenSSL link surface.
- Small dep graph — we are not building a web framework.
- Mature TLS — local Ollama is HTTP, but org-policy proxies may force
  HTTPS.

## Decision

**`ureq` is the LLM transport.** Configured `default-features = false`
with `features = ["json", "tls"]` — `tls` selects `ureq`'s rustls
backend.

Closes the v0.0.44 #520 shellout port. The v0.0.45 #558 lint
(`tools/lint-no-shellout.sh`) prevents regression.

## Consequences

**Positive.**

- One syscall per LLM call. No subprocess spawn, no PATH lookup, no
  Windows `cmd.exe` quoting surface.
- Typed errors: `SsgError::LlmEndpointUnreachable`,
  `SsgError::LlmTimeout`, `SsgError::LlmInvalidResponse`. Callers
  branch cleanly.
- Pure rustls — no native OpenSSL link, preserves the
  `cargo install ssg` UX on musl and minimal Linux images.
- Synchronous fits naturally inside a Rayon worker; no executor
  acrobatics.

**Negative.**

- `ureq` 2.x's blocking model means one in-flight LLM call holds a
  Rayon worker for the duration of the call. The v0.0.49 #575
  `LlmCache` mitigates this by caching deterministically; concurrent
  cache misses still serialise per worker.
- `ureq` does not support HTTP/2 or HTTP/3. Local Ollama is HTTP/1.1,
  so this does not bind today; if an org-policy proxy demands HTTP/2,
  the LlmPlugin will need a separate transport adapter.

## Alternatives Considered

- **`reqwest`.** Pulls tokio transitively even in blocking mode.
  Forbidden by ADR-0001.
- **`hyper` directly.** Same transitive tokio dependency post-1.0.
- **`isahc`.** Pulls libcurl (C). Loses the rustls-only narrative.
- **Continued `curl` shellout.** Rejected for the three reasons in
  Context.
- **Hand-rolled HTTP/1.1 over `std::net::TcpStream`.** Rejected: TLS
  is a multi-thousand-LOC commitment we should not own.

## Status

Accepted. `ureq = { version = "2", default-features = false, features = ["json", "tls"] }`
is unconditional in `Cargo.toml` (it's used by the plugin module even
when the `ai` feature is off; the module compiles in either case).

<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# ADR-0004: Sync `tungstenite` for HMR fan-out

- **Date:** 2026-06-26
- **Status:** Accepted — supersession planned in v0.0.48 (#571)

## Context

The dev-server Hot Module Replacement (HMR) broadcaster pushes
`hmr-css`, `hmr-html`, `hmr-markdown`, and `reload` frames to every
connected browser tab. The connection pattern is:

- 1–5 concurrent subscribers (a developer rarely has dozens of tabs).
- Small frames (≤ 4 KB JSON).
- Burst rate ≤ 10 frames/s on aggressive edits.

This is the smallest-possible WebSocket workload. ADR-0001 forbids
tokio; the choice was between:

1. Sync `tungstenite` with a `std::thread::spawn` accept loop and
   `std::sync::mpsc` fan-out.
2. `async-tungstenite` over `smol` (single new executor, async fan-out).

Option 1 is the simplest possible architecture. The downside is
**head-of-line blocking**: one slow subscriber's TCP write blocks the
broadcaster loop until that write returns.

For a 1–5-subscriber dev workload that downside is theoretical, not
operational. We shipped option 1 in v0.0.44.

## Decision

**For v0.0.44 through v0.0.47 we use sync `tungstenite` driven from
a dedicated `std::thread::spawn` loop reading frames from a
`std::sync::mpsc::Receiver`.**

The broadcaster fans out by iterating subscribers and writing to each
WebSocket in order.

## Consequences

**Positive.**

- Minimal code surface: ~250 LOC for the broadcaster + accept loop.
- Trivial reasoning: it's an `mpsc` loop; what you see is what you
  get.
- No second executor, preserves ADR-0001 cleanly.

**Negative.**

- One slow subscriber blocks the whole loop. This has been observed
  during local testing only when a Chrome tab is paused at a
  breakpoint; the failure mode is "other tabs stop receiving HMR
  frames until the paused tab disconnects."
- The architecture does not scale beyond ~10 subscribers. Acceptable
  for a dev-server; would be wrong for any production fan-out.

## Alternatives Considered

- **`async-tungstenite` over `smol`.** Holds for v0.0.48 #571
  migration. We deferred it to v0.0.44 because the head-of-line risk
  was acceptable for the dev workload, and shipping the simpler
  architecture first let us measure the actual problem rather than
  pre-optimise.
- **`async-tungstenite` over `tokio`.** Forbidden by ADR-0001.
- **Per-subscriber `std::thread::spawn`.** Rejected: leaking one
  thread per browser tab is a footgun and complicates clean shutdown
  on Ctrl-C.

## Status

**Accepted — supersession planned.** Tracked in v0.0.48 #571. The
migration is motivated by HMR-to-many-subscribers cases that the
v0.0.46 #564 Loom model surfaces as theoretically broken, not by an
observed production failure. A successor ADR-0008 will document the
final smol-based design once #571 ships.

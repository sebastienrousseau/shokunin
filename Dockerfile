# SPDX-License-Identifier: Apache-2.0 OR MIT
#
# Two-stage build: cargo-chef for cached dependency layer, then
# debian-slim runtime with the static-site-generator binary.
#
# Image: ghcr.io/sebastienrousseau/static-site-generator
# Built and pushed by .github/workflows/release.yml on every `v*` tag.

# ── Stage 1: build ─────────────────────────────────────────────────
FROM rust:1.88-slim AS builder

WORKDIR /usr/src/ssg
# hadolint ignore=DL3008
RUN apt-get update \
 && apt-get install -y --no-install-recommends pkg-config libssl-dev \
 && rm -rf /var/lib/apt/lists/*

# Copy manifests first so the dependency layer caches across changes
# to source files only.
COPY Cargo.toml Cargo.lock build.rs rust-toolchain.toml ./
COPY crates/serde_yml/Cargo.toml crates/serde_yml/Cargo.toml
RUN mkdir -p src crates/serde_yml/src \
 && echo "fn main() {}" > src/main.rs \
 && echo "pub fn _stub() {}" > crates/serde_yml/src/lib.rs \
 && cargo build --release --locked --bin ssg \
 && rm -rf src crates/serde_yml/src

# Now copy the real source and rebuild only the bin target.
COPY src ./src
COPY crates ./crates
COPY benches ./benches
COPY templates ./templates
COPY themes ./themes
RUN touch src/main.rs && cargo build --release --locked --bin ssg

# ── Stage 2: runtime ───────────────────────────────────────────────
FROM debian:bookworm-slim AS runtime

LABEL org.opencontainers.image.title="static-site-generator"
LABEL org.opencontainers.image.description="A Content-First Open Source Static Site Generator (SSG) crafted in Rust"
LABEL org.opencontainers.image.source="https://github.com/sebastienrousseau/static-site-generator"
LABEL org.opencontainers.image.licenses="MIT OR Apache-2.0"
LABEL org.opencontainers.image.authors="Sebastien Rousseau <contact@sebastienrousseau.com>"

RUN apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates libssl3 \
 && rm -rf /var/lib/apt/lists/* \
 && useradd --create-home --shell /sbin/nologin --uid 10001 ssg

COPY --from=builder /usr/src/ssg/target/release/ssg /usr/local/bin/ssg

USER ssg
WORKDIR /workspace
EXPOSE 8000

ENTRYPOINT ["/usr/local/bin/ssg"]
CMD ["--help"]

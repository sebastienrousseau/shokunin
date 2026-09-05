# SPDX-License-Identifier: Apache-2.0 OR MIT
#
# Build image for ClusterFuzzLite and OSS-Fuzz.
# Pinned by digest, not just by tag: `:v1` is mutable, so the image this
# builds against can change under us without any commit here. Flagged by
# Scorecard's Pinned-Dependencies check.
#
# The cost is that base updates -- including security fixes to the builder
# itself -- now arrive only when this line is bumped. The tag is kept
# alongside the digest so it is obvious which upstream tag to re-resolve:
#
#   docker buildx imagetools inspect gcr.io/oss-fuzz-base/base-builder-rust:v1
FROM gcr.io/oss-fuzz-base/base-builder-rust:v1@sha256:433319ea1a6bbea6c1db1915a5d385021908cc62026d9f0efda2270ef6060de3

# The fuzz build script enumerates targets from fuzz/Cargo.toml, so no target
# list is duplicated here.
COPY . $SRC/static-site-generator
WORKDIR $SRC/static-site-generator
COPY .clusterfuzzlite/build.sh $SRC/build.sh

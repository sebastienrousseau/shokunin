# SPDX-License-Identifier: Apache-2.0 OR MIT
#
# Build image for ClusterFuzzLite and OSS-Fuzz.
FROM gcr.io/oss-fuzz-base/base-builder-rust:v1

# The fuzz build script enumerates targets from fuzz/Cargo.toml, so no target
# list is duplicated here.
COPY . $SRC/static-site-generator
WORKDIR $SRC/static-site-generator
COPY .clusterfuzzlite/build.sh $SRC/build.sh

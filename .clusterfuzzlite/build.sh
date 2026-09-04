#!/bin/bash -eu
# SPDX-License-Identifier: Apache-2.0 OR MIT
#
# Thin shim. The real logic lives in fuzz/oss-fuzz-build.sh so that a
# developer can run exactly what CI runs, without a container.
exec "$SRC/static-site-generator/fuzz/oss-fuzz-build.sh"

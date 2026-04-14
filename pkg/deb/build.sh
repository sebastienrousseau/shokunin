#!/bin/sh
# Build a .deb package from the release binary
set -eu
VERSION="${1:?Usage: build.sh <version>}"
ARCH="${2:-amd64}"
PKG="ssg_${VERSION}_${ARCH}"

mkdir -p "$PKG/DEBIAN" "$PKG/usr/bin" "$PKG/usr/share/doc/ssg"
cp target/release/ssg "$PKG/usr/bin/"
strip "$PKG/usr/bin/ssg"
cp LICENSE-MIT LICENSE-APACHE "$PKG/usr/share/doc/ssg/"

cat > "$PKG/DEBIAN/control" << EOF
Package: ssg
Version: ${VERSION}
Section: web
Priority: optional
Architecture: ${ARCH}
Depends: libc6 (>= 2.31)
Maintainer: Sebastien Rousseau <contact@sebastienrousseau.com>
Description: Fast, SEO-optimised static site generator
 SSG generates static websites from Markdown content with built-in
 SEO, accessibility compliance, and i18n support. Built in Rust.
Homepage: https://github.com/sebastienrousseau/shokunin
EOF

dpkg-deb --build "$PKG"
echo "Built $PKG.deb"

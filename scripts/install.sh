#!/bin/sh
# SPDX-License-Identifier: Apache-2.0 OR MIT
#
# SSG Installer — downloads the latest release binary for your platform.
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/sebastienrousseau/static-site-generator/main/scripts/install.sh | sh
#
# Environment variables:
#   SSG_VERSION   — specific version to install (default: latest)
#   SSG_INSTALL   — install directory (default: ~/.local/bin)

set -eu

REPO="sebastienrousseau/shokunin"
BINARY="ssg"
INSTALL_DIR="${SSG_INSTALL:-$HOME/.local/bin}"

# ---------------------------------------------------------------------------
# Detect platform
# ---------------------------------------------------------------------------
detect_platform() {
  OS="$(uname -s)"
  ARCH="$(uname -m)"

  case "$OS" in
    Linux)
      case "$ARCH" in
        x86_64)  TARGET="x86_64-unknown-linux-musl" ;;
        aarch64) TARGET="aarch64-unknown-linux-gnu" ;;
        *)       error "Unsupported Linux architecture: $ARCH" ;;
      esac
      EXT="tar.gz"
      ;;
    Darwin)
      case "$ARCH" in
        x86_64)  TARGET="x86_64-apple-darwin" ;;
        arm64)   TARGET="aarch64-apple-darwin" ;;
        *)       error "Unsupported macOS architecture: $ARCH" ;;
      esac
      EXT="tar.gz"
      ;;
    MINGW*|MSYS*|CYGWIN*)
      TARGET="x86_64-pc-windows-msvc"
      EXT="zip"
      ;;
    *)
      error "Unsupported operating system: $OS"
      ;;
  esac
}

# ---------------------------------------------------------------------------
# Resolve version
# ---------------------------------------------------------------------------
resolve_version() {
  if [ -n "${SSG_VERSION:-}" ]; then
    VERSION="$SSG_VERSION"
  else
    VERSION="$(curl -fsSL "https://api.github.com/repos/$REPO/releases/latest" \
      | grep '"tag_name"' | head -1 | cut -d'"' -f4)"
    if [ -z "$VERSION" ]; then
      error "Failed to fetch latest version from GitHub"
    fi
  fi
}

# ---------------------------------------------------------------------------
# Download, verify, install
# ---------------------------------------------------------------------------
install() {
  ARCHIVE="${BINARY}-${VERSION}-${TARGET}.${EXT}"
  URL="https://github.com/$REPO/releases/download/${VERSION}/${ARCHIVE}"
  CHECKSUM_URL="${URL}.sha256"

  TMPDIR="$(mktemp -d)"
  trap 'rm -rf "$TMPDIR"' EXIT

  info "Downloading $BINARY $VERSION for $TARGET..."
  curl -fsSL "$URL" -o "$TMPDIR/$ARCHIVE"
  curl -fsSL "$CHECKSUM_URL" -o "$TMPDIR/$ARCHIVE.sha256" 2>/dev/null || true

  # Verify checksum if available
  if [ -f "$TMPDIR/$ARCHIVE.sha256" ]; then
    info "Verifying checksum..."
    cd "$TMPDIR"
    if command -v sha256sum >/dev/null 2>&1; then
      sha256sum -c "$ARCHIVE.sha256"
    elif command -v shasum >/dev/null 2>&1; then
      shasum -a 256 -c "$ARCHIVE.sha256"
    else
      warn "No checksum tool found — skipping verification"
    fi
    cd - >/dev/null
  fi

  # Extract
  info "Extracting..."
  case "$EXT" in
    tar.gz) tar xzf "$TMPDIR/$ARCHIVE" -C "$TMPDIR" ;;
    zip)    unzip -qo "$TMPDIR/$ARCHIVE" -d "$TMPDIR" ;;
  esac

  # Install
  mkdir -p "$INSTALL_DIR"
  if [ "$EXT" = "zip" ]; then
    cp "$TMPDIR/${BINARY}.exe" "$INSTALL_DIR/"
  else
    cp "$TMPDIR/$BINARY" "$INSTALL_DIR/"
    chmod +x "$INSTALL_DIR/$BINARY"
  fi

  success "Installed $BINARY $VERSION to $INSTALL_DIR/$BINARY"

  # Check PATH
  case ":$PATH:" in
    *":$INSTALL_DIR:"*) ;;
    *)
      warn "$INSTALL_DIR is not in your PATH."
      echo "  Add it with:  export PATH=\"$INSTALL_DIR:\$PATH\""
      echo "  Or add to your shell profile (~/.bashrc, ~/.zshrc, etc.)"
      ;;
  esac
}

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------
info()    { printf '\033[1;34m==>\033[0m %s\n' "$*"; }
success() { printf '\033[1;32m==>\033[0m %s\n' "$*"; }
warn()    { printf '\033[1;33mwarning:\033[0m %s\n' "$*"; }
error()   { printf '\033[1;31merror:\033[0m %s\n' "$*" >&2; exit 1; }

# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------
main() {
  info "SSG Installer"
  detect_platform
  resolve_version
  install
}

main "$@"

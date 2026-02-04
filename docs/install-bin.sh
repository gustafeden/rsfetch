#!/bin/sh
set -e

# blaeckfetch installer bootstrap (Rust binary installer)
# Downloads the blaeck-powered Rust installer binary and runs it.
# Usage: curl -fsSL https://gustafeden.github.io/blaeckfetch/install-bin.sh | sh

REPO="gustafeden/blaeckfetch"
INSTALLER_NAME="blaeckfetch-installer"

say() {
    printf '%s\n' "$1"
}

err() {
    say "error: $1" >&2
    exit 1
}

need() {
    command -v "$1" >/dev/null 2>&1 || err "need '$1' (command not found)"
}

# --- Detect platform ---

detect_platform() {
    OS="$(uname -s)"
    ARCH="$(uname -m)"

    case "$OS" in
        Darwin) OS_LABEL="macOS" ;;
        Linux)  OS_LABEL="Linux" ;;
        *)      err "unsupported OS: $OS" ;;
    esac

    case "$ARCH" in
        x86_64)  ARCH_LABEL="x86_64" ;;
        aarch64) ARCH_LABEL="aarch64" ;;
        arm64)   ARCH="aarch64"; ARCH_LABEL="arm64" ;;
        *)       err "unsupported architecture: $ARCH" ;;
    esac

    case "${OS}-${ARCH}" in
        Darwin-x86_64)  TARGET="x86_64-apple-darwin" ;;
        Darwin-aarch64) TARGET="aarch64-apple-darwin" ;;
        Linux-x86_64)   TARGET="x86_64-unknown-linux-gnu" ;;
        Linux-aarch64)  TARGET="aarch64-unknown-linux-gnu" ;;
        *)              err "unsupported platform: ${OS} ${ARCH}" ;;
    esac
}

# --- Resolve version ---

get_version() {
    if [ -n "${RSFETCH_VERSION:-}" ]; then
        VERSION="$RSFETCH_VERSION"
        return
    fi

    need curl
    VERSION="$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" \
        | grep '"tag_name"' \
        | head -1 \
        | sed 's/.*"tag_name": *"v\{0,1\}\([^"]*\)".*/\1/')"

    [ -n "$VERSION" ] || err "could not determine latest version"
}

# --- Main ---

main() {
    need curl
    need tar

    detect_platform
    get_version

    say "blaeckfetch installer — ${OS_LABEL} ${ARCH_LABEL}"
    say ""

    URL="https://github.com/${REPO}/releases/download/v${VERSION}/${INSTALLER_NAME}-${TARGET}.tar.gz"
    TMPDIR="$(mktemp -d)"
    trap 'rm -rf "$TMPDIR"' EXIT

    say "downloading ${INSTALLER_NAME} v${VERSION}..."
    curl -fsSL "$URL" | tar xz -C "$TMPDIR"

    # Remove macOS quarantine attribute
    case "$OS" in
        Darwin)
            xattr -d com.apple.quarantine "$TMPDIR/${INSTALLER_NAME}" 2>/dev/null || true
            ;;
    esac

    chmod +x "$TMPDIR/${INSTALLER_NAME}"
    "$TMPDIR/${INSTALLER_NAME}"
}

main "$@"

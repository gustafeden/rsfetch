#!/bin/sh
set -e

# Build
cargo build --release

# Find install dir: first writable dir in PATH under $HOME, fallback to ~/.local/bin
INSTALL_DIR=""
for dir in $(echo "$PATH" | tr ':' '\n'); do
    case "$dir" in
        "$HOME"*)
            if [ -d "$dir" ] && [ -w "$dir" ]; then
                INSTALL_DIR="$dir"
                break
            fi
            ;;
    esac
done

if [ -z "$INSTALL_DIR" ]; then
    INSTALL_DIR="$HOME/.local/bin"
    mkdir -p "$INSTALL_DIR"
fi

cp target/release/rsfetch "$INSTALL_DIR/rsfetch"
echo "installed rsfetch to $INSTALL_DIR/rsfetch"

# Verify
if command -v rsfetch >/dev/null 2>&1; then
    echo ""
    rsfetch
else
    echo ""
    echo "note: $INSTALL_DIR may not be in your PATH"
    echo "add this to your shell config:"
    echo "  export PATH=\"$INSTALL_DIR:\$PATH\""
fi

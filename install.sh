#!/usr/bin/env bash
set -eo pipefail

# blaeckfetch installer (blaeck-sh)
# Usage: curl -fsSL https://gustafeden.github.io/blaeckfetch/install.sh | bash

REPO="gustafeden/blaeckfetch"
BLAECK_SH_URL="https://gustafeden.github.io/blaeck-sh/blaeck.sh"

# ---------------------------------------------------------------------------
# Load blaeck-sh
# ---------------------------------------------------------------------------
_blaeck_src="$(curl -fsSL "$BLAECK_SH_URL" 2>/dev/null)" || {
    echo "error: could not download blaeck.sh" >&2
    exit 1
}
eval "$_blaeck_src"

# ---------------------------------------------------------------------------
# Detect platform
# ---------------------------------------------------------------------------
detect_platform() {
    local os arch
    os="$(uname -s)"
    arch="$(uname -m)"

    case "$os" in
        Darwin) OS_LABEL="macOS" ;;
        Linux)  OS_LABEL="Linux" ;;
        *)      echo "error: unsupported OS: $os" >&2; exit 1 ;;
    esac

    case "$arch" in
        x86_64)           ARCH_LABEL="x86_64" ;;
        aarch64)          ARCH_LABEL="aarch64" ;;
        arm64) arch="aarch64"; ARCH_LABEL="arm64" ;;
        *)      echo "error: unsupported architecture: $arch" >&2; exit 1 ;;
    esac

    case "${os}-${arch}" in
        Darwin-x86_64)  TARGET="x86_64-apple-darwin" ;;
        Darwin-aarch64) TARGET="aarch64-apple-darwin" ;;
        Linux-x86_64)   TARGET="x86_64-unknown-linux-gnu" ;;
        Linux-aarch64)  TARGET="aarch64-unknown-linux-gnu" ;;
        *)              echo "error: unsupported platform: ${os} ${arch}" >&2; exit 1 ;;
    esac
}

# ---------------------------------------------------------------------------
# Resolve version
# ---------------------------------------------------------------------------
get_version() {
    if [[ -n "${RSFETCH_VERSION:-}" ]]; then
        VERSION="$RSFETCH_VERSION"
        return
    fi

    VERSION="$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" 2>/dev/null \
        | grep '"tag_name"' \
        | head -1 \
        | sed 's/.*"tag_name": *"v\{0,1\}\([^"]*\)".*/\1/')"

    [[ -n "$VERSION" ]] || { echo "error: could not determine latest version" >&2; exit 1; }
}

# ---------------------------------------------------------------------------
# Resolve install dir
# ---------------------------------------------------------------------------
resolve_install_dir() {
    local dir
    while IFS=: read -rd: dir || [[ -n "$dir" ]]; do
        if [[ "$dir" == "$HOME"* && -d "$dir" && -w "$dir" ]]; then
            INSTALL_DIR="$dir"
            return
        fi
    done <<< "$PATH:"

    INSTALL_DIR="$HOME/.local/bin"
    mkdir -p "$INSTALL_DIR"
}

# ---------------------------------------------------------------------------
# Detect existing version
# ---------------------------------------------------------------------------
detect_existing() {
    EXISTING_VERSION=""
    if command -v blaeckfetch &>/dev/null; then
        EXISTING_VERSION="$(blaeckfetch --version 2>/dev/null || true)"
    fi
}

# ---------------------------------------------------------------------------
# Detect shell RC file
# ---------------------------------------------------------------------------
detect_shell_rc() {
    local shell="${SHELL:-}"
    case "$shell" in
        */zsh)  SHELL_RC="~/.zshrc" ;;
        */bash)
            if [[ -f "$HOME/.bash_profile" ]]; then
                SHELL_RC="~/.bash_profile"
            else
                SHELL_RC="~/.bashrc"
            fi
            ;;
        */fish) SHELL_RC="fish" ;;
        *)      SHELL_RC="~/.profile" ;;
    esac
}

# ---------------------------------------------------------------------------
# Display path with ~ for HOME
# ---------------------------------------------------------------------------
display_path() {
    local p="$1"
    if [[ -n "$HOME" && "$p" == "$HOME"* ]]; then
        echo "~${p#"$HOME"}"
    else
        echo "$p"
    fi
}

# ---------------------------------------------------------------------------
# Render installer UI
# ---------------------------------------------------------------------------
render_ui() {
    local -a lines=()

    # Header
    lines+=("$(bk_style "  blaeckfetch installer v${VERSION}" --bold --color white)")
    lines+=("$(bk_dim "  ────────────────────")")
    lines+=("")

    # Steps
    local i
    for i in "${!STEP_NAMES[@]}"; do
        local name="${STEP_NAMES[$i]}"
        local status="${STEP_STATUS[$i]}"
        local detail="${STEP_DETAIL[$i]}"

        case "$status" in
            pending)
                lines+=("$(bk_status pending "$name" "$detail")" ) ;;
            active)
                local frames_var
                frames_var=$(_bk_spin_frames "circle")
                eval "local frame_count=\${#${frames_var}[@]}"
                local frame_idx=$(( (_bk_spin_idx) % frame_count ))
                eval "local spin_char=\${${frames_var}[$frame_idx]}"
                lines+=("$(printf '%s%s%s %-12s %s' "$_BK_CYAN" "$spin_char" "$_BK_RESET" "$name" "$detail")")
                ;;
            done)
                lines+=("$(bk_status ok "$name" "$detail")") ;;
            fail)
                lines+=("$(bk_status fail "$name" "$detail")") ;;
        esac
    done

    # Error
    if [[ -n "${ERROR_MSG:-}" ]]; then
        lines+=("")
        lines+=("$(bk_red "  Error: $ERROR_MSG")")
    fi

    # Success section
    if [[ "${FINISHED:-0}" -eq 1 && -z "${ERROR_MSG:-}" ]]; then
        local scenario="fresh"
        if [[ -n "$EXISTING_VERSION" ]]; then
            if [[ "$EXISTING_VERSION" == "blaeckfetch $VERSION" ]]; then
                scenario="reinstall"
            else
                scenario="update"
            fi
        fi

        lines+=("")

        case "$scenario" in
            fresh)
                local box_line1
                box_line1="$(bk_green "✓") blaeckfetch v${VERSION} installed"
                lines+=("$(bk_box --style round --color green "$box_line1")")
                ;;
            update)
                local box_line1 box_line2
                box_line1="$(bk_green "✓") blaeckfetch updated"
                box_line2="  ${EXISTING_VERSION} → v${VERSION}"
                lines+=("$(bk_box --style round --color green "$box_line1" "$box_line2")")
                ;;
            reinstall)
                local box_line1
                box_line1="$(bk_green "✓") blaeckfetch v${VERSION} reinstalled"
                lines+=("$(bk_box --style round --color green "$box_line1")")
                ;;
        esac

        # PATH warning
        local install_in_path=0
        local dir
        while IFS=: read -rd: dir || [[ -n "$dir" ]]; do
            [[ "$dir" == "$INSTALL_DIR" ]] && install_in_path=1
        done <<< "$PATH:"

        if [[ $install_in_path -eq 0 ]]; then
            detect_shell_rc
            local dir_display
            dir_display="$(display_path "$INSTALL_DIR")"
            lines+=("")
            lines+=("$(bk_yellow "  Add to your PATH (run once):")")
            if [[ "$SHELL_RC" == "fish" ]]; then
                lines+=("    set -Ux fish_user_paths $dir_display \$fish_user_paths")
            else
                lines+=("    echo 'export PATH=\"$dir_display:\$PATH\"' >> $SHELL_RC")
                lines+=("    source $SHELL_RC")
            fi
        fi

        # Fresh install: get started
        if [[ "$scenario" == "fresh" ]]; then
            lines+=("")
            lines+=("$(bk_bold "  Get started:")")
            lines+=("    blaeckfetch$(bk_dim "                     run it")")
            lines+=("    blaeckfetch -c cyan$(bk_dim "             try a color theme")")
            lines+=("    blaeckfetch --help$(bk_dim "              see all options")")
            lines+=("")
            lines+=("$(bk_dim "  Config: blaeckfetch --print-config > ~/.config/blaeckfetch/config.toml")")
            lines+=("$(bk_dim "  More:   ")$(bk_cyan "https://github.com/gustafeden/blaeckfetch")")
        fi

        # Update: what's new
        if [[ "$scenario" == "update" ]]; then
            lines+=("")
            lines+=("$(bk_dim "  What's new: ")$(bk_cyan "https://github.com/gustafeden/blaeckfetch/releases/tag/v${VERSION}")")
        fi

        lines+=("")
    fi

    bk_render "${lines[@]}"
}

# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------
main() {
    detect_platform
    get_version
    resolve_install_dir
    detect_existing

    # Step state
    STEP_NAMES=("Platform" "Install to" "Downloading" "Extracting" "Verifying")
    STEP_STATUS=("pending" "pending" "pending" "pending" "pending")
    STEP_DETAIL=("" "" "" "" "")
    ERROR_MSG=""
    FINISHED=0

    bk_render_init 8  # header(3) + steps(5)
    trap 'bk_render_done' EXIT

    # Step 0: Platform
    STEP_STATUS[0]="done"
    STEP_DETAIL[0]="$OS_LABEL $ARCH_LABEL"
    render_ui

    # Step 1: Install dir
    STEP_STATUS[1]="done"
    STEP_DETAIL[1]="$(display_path "$INSTALL_DIR")"
    render_ui

    # Step 2: Download
    STEP_STATUS[2]="active"
    local url="https://github.com/${REPO}/releases/download/v${VERSION}/blaeckfetch-${TARGET}.tar.gz"
    local tmpdir
    tmpdir="$(mktemp -d)"
    trap 'rm -rf "$tmpdir"; bk_render_done' EXIT

    # Download with spinner
    local download_err=""
    curl -fsSL "$url" -o "$tmpdir/blaeckfetch.tar.gz" 2>/dev/null &
    local dl_pid=$!

    while kill -0 "$dl_pid" 2>/dev/null; do
        _bk_spin_idx=$(( (_bk_spin_idx + 1) % 4 ))
        render_ui
        sleep 0.08
    done

    if ! wait "$dl_pid"; then
        STEP_STATUS[2]="fail"
        STEP_DETAIL[2]="download failed"
        ERROR_MSG="could not download blaeckfetch v${VERSION} for ${TARGET}"
        render_ui
        exit 1
    fi

    local size_bytes size_mb
    size_bytes="$(wc -c < "$tmpdir/blaeckfetch.tar.gz" | tr -d ' ')"
    size_mb="$(awk "BEGIN { printf \"%.1f\", $size_bytes / 1048576 }")"
    STEP_STATUS[2]="done"
    STEP_DETAIL[2]="blaeckfetch v${VERSION} (${size_mb} MB)"

    render_ui

    # Step 3: Extract
    STEP_STATUS[3]="active"
    render_ui

    tar xzf "$tmpdir/blaeckfetch.tar.gz" -C "$tmpdir"
    if [[ ! -f "$tmpdir/blaeckfetch" ]]; then
        STEP_STATUS[3]="fail"
        STEP_DETAIL[3]="binary not found in archive"
        ERROR_MSG="blaeckfetch binary not found in archive"
        render_ui
        exit 1
    fi

    chmod +x "$tmpdir/blaeckfetch"
    cp "$tmpdir/blaeckfetch" "$INSTALL_DIR/blaeckfetch"
    STEP_STATUS[3]="done"
    STEP_DETAIL[3]="$(display_path "$INSTALL_DIR/blaeckfetch")"
    render_ui

    # Step 4: Verify
    STEP_STATUS[4]="active"
    render_ui

    local verify_out
    if verify_out="$("$INSTALL_DIR/blaeckfetch" --version 2>&1)"; then
        STEP_STATUS[4]="done"
        STEP_DETAIL[4]="$verify_out"
    else
        STEP_STATUS[4]="fail"
        STEP_DETAIL[4]="verification failed"
        ERROR_MSG="blaeckfetch --version returned non-zero"
        render_ui
        exit 1
    fi

    FINISHED=1

    # Resize render block for post-install content
    local final_lines=14  # base: header(3) + steps(5) + box(4) + padding(2)
    if [[ -z "$EXISTING_VERSION" ]]; then
        final_lines=24  # fresh install: get started + config + more
    elif [[ "$EXISTING_VERSION" != "blaeckfetch $VERSION" ]]; then
        final_lines=17  # update: what's new link
    fi
    local in_path=0
    local d
    while IFS=: read -rd: d || [[ -n "$d" ]]; do
        [[ "$d" == "$INSTALL_DIR" ]] && in_path=1
    done <<< "$PATH:"
    [[ $in_path -eq 0 ]] && (( final_lines += 4 ))

    bk_render_resize "$final_lines"
    render_ui
}

main "$@"

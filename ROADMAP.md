# rsfetch roadmap

## Animated logo on terminal startup

Make the ASCII logo animate (e.g. spinning, color cycling, pulsing) when
the terminal opens, then freeze the moment the user starts interacting.

### Approach A — Foreground blocking (recommended)

```zsh
# .zshrc
rsfetch --animate   # blocks until keypress, then exits
```

rsfetch owns the terminal during animation. It renders the full fetch
display and loops the logo animation using blaeck's re-rendering (cursor
positioning to overwrite just the logo region). It reads `/dev/tty` in raw
mode (no echo, no line buffering) waiting for any keypress. On first
keypress it renders the final static frame, restores the terminal, and
exits. The shell prompt appears after exit.

**Pros:**
- Simple — no background process, no stdin juggling, no orphaned PIDs
- No cursor coordinate issues since rsfetch owns the whole screen
- Clean exit, shell takes over naturally
- Works on every terminal

**Cons:**
- Shell prompt doesn't appear until animation stops
- Can't type commands while the logo is still animating

**Implementation:**
1. Add `--animate` flag (clap)
2. After initial render, enter animation loop:
   - Advance logo frame (rotate, spin, color shift, etc.)
   - Re-render only the logo region using cursor positioning
   - Poll `/dev/tty` with `termios` raw mode, ~80ms frame interval
   - On any keypress or timeout: break
3. Render final static frame
4. Restore terminal (`termios` original settings)
5. Exit

### Approach B — Background process (experimental / fragile)

```zsh
# .zshrc
rsfetch --animate --background &
RSFETCH_PID=$!

preexec() {
    [[ -n "$RSFETCH_PID" ]] && kill -USR1 $RSFETCH_PID 2>/dev/null
    unset RSFETCH_PID
}
```

rsfetch forks to background immediately, shell prompt appears alongside
the animated logo. Animation runs behind the prompt. zsh `preexec` hook
signals rsfetch to stop before any command executes.

**Pros:**
- Prompt is available immediately while logo animates
- Feels more "alive" — animation and prompt coexist

**Cons:**
- Scrolling invalidates saved cursor coordinates
- Cursor flicker from jumping between prompt and logo area
- Race condition: typing during mid-frame-redraw causes garbled output
- Terminal resize breaks coordinate math
- Multiple tabs = multiple competing background processes
- `TIOCSTI` (inject keypress back) is deprecated on newer kernels

**Implementation:**
1. Add `--background` flag
2. Fork: parent exits immediately, child continues
3. Save screen region coordinates (row/col of logo area)
4. Animation loop:
   - Save cursor position (`\033[s`)
   - Move to logo area (`\033[<row>;<col>H`)
   - Redraw logo frame
   - Restore cursor position (`\033[u`)
   - Sleep ~80ms
5. Install `SIGUSR1` handler: render final frame, exit
6. Also support timeout (e.g. 30s auto-stop)

### Logo animation types

Ideas for what "animated" means:

- **Rotation/spin** — cycle through rotated versions of the logo
- **Color cycling** — shift hue across the logo over time
- **Pulse/breathe** — dim ↔ bright oscillation
- **Typewriter reveal** — draw the logo character by character on first display
- **Glitch** — random character substitution that settles into the final logo
- **Matrix rain** — characters rain down and form the logo

### Prerequisites

- Animation frame data: either procedurally generated (color shift, pulse)
  or pre-rendered frame sets for each logo
- Terminal capability detection: ensure raw mode and cursor positioning work
- Graceful degradation: `--animate` with an unsupported terminal falls back
  to static display

---

## Animated inline images (terminal image protocols)

For terminals that support inline image rendering (iTerm2, kitty, WezTerm,
Ghostty, etc.), display an animated GIF as the logo instead of ASCII art.
The terminal handles the animation loop natively — rsfetch exits and the
GIF keeps playing in the scrollback.

### Supported protocols

| Terminal   | Protocol                          | Animated GIF support |
|------------|-----------------------------------|----------------------|
| iTerm2     | `\033]1337;File=inline=1;...`     | Yes                  |
| Kitty      | Kitty graphics protocol           | Yes (frame-based)    |
| WezTerm    | iTerm2 protocol (compatible)      | Yes                  |
| Ghostty    | Kitty graphics protocol           | Yes                  |
| Sixel      | Various (mlterm, foot, etc.)      | Partial              |

### Implementation

1. Detect terminal (check `$TERM_PROGRAM`, `$TERM`, kitty detection)
2. Select protocol (iTerm2 vs kitty vs sixel vs none)
3. If supported: encode animated GIF with the appropriate escape sequence,
   emit it in the logo position. rsfetch exits, terminal keeps animating.
4. If not supported: fall back to static ASCII art

### Logo sources

- Ship small animated GIFs for built-in logos (Apple spin, Tux wave, etc.)
- `--logo-gif <path>` flag for custom animated GIFs
- `logo_gif` config option

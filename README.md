# blaeckfetch

A fast, minimalist system fetch for your terminal. Written in Rust. Powered by [blaeck](https://github.com/gustafeden/blaeck) for rendering.


![blaeckfetch default](https://gustafeden.github.io/blaeckfetch/demo/default.gif)

Blaeckfetch default logo (moon) with essential system info. Customize as you like:

```toml
logo = "arch"
fields = ["OS", "CPU", "GPU", "Memory", "Disk (/)", "Local IP"]
color = "cyan"
palette = true

[colors]
title = [255, 165, 0]
label = "cyan"
logo = "magenta"
```

## Splash mode

Animated sequence inspired by retro SEGA intro, background image, blinking footer, and collapse exit. Use a custom PNG/JPEG or it will fallback to an ASCII procedural starfield. Press any key to dismiss. Add `blaeckfetch --splash` to your shell RC for a startup splash.


![splash — background image](https://gustafeden.github.io/blaeckfetch/demo/splash-image.gif)

> More modes to come in the future.

## Performance

| Tool | Time |
|------|------|
| **blaeckfetch** | **~7ms** |
| neofetch | ~400ms |

~57x faster than neofetch. Boot-cycle cache keeps static fields instant after the first run.

## Terminal compatibility

Works in any terminal that supports ANSI escape codes. Truecolor terminals (iTerm2, WezTerm, Kitty, Ghostty, Alacritty) get full 24-bit RGB. Terminals without truecolor (macOS Terminal.app) automatically fall back to 256-color approximation. VHS recordings are supported with full animation.

## Install

### macOS

```sh
brew tap gustafeden/tap && brew install blaeckfetch
```

### Arch Linux

```sh
yay -S blaeckfetch-bin
```

### Linux / macOS (installer script)

```sh
curl -fsSL https://gustafeden.github.io/blaeckfetch/install.sh | bash
```

### From source

```sh
cargo install --git https://github.com/gustafeden/blaeckfetch blaeckfetch
```

## Usage

```sh
blaeckfetch                  # default mode
blaeckfetch -c cyan          # color theme
blaeckfetch --logo arch      # override logo
blaeckfetch --splash         # splash screen
blaeckfetch --neofetch       # neofetch layout
blaeckfetch --json           # JSON output
```

Generate a config file:

```sh
blaeckfetch --print-config > ~/.config/blaeckfetch/config.toml
```

## Documentation

Full docs — configuration reference, splash mode options, field list, and more:

**[gustafeden.github.io/blaeckfetch](https://gustafeden.github.io/blaeckfetch/)**

## License

MIT

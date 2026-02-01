# rsfetch

A fast system fetch display for your terminal, written in Rust. Powered by [blaeck](https://github.com/gustafeden/blaeck) for rendering.

## Performance

| Tool | Time |
|------|------|
| **rsfetch** | **~7ms** |
| neofetch | ~400ms |

~57x faster than neofetch. Uses a boot-cycle cache for static fields — first run after reboot is ~40ms, every run after that is ~7ms.

## Output

```
                    'c.          gustafeden@hostname
                 ,xNMM.          --------------------------
               .OMMMMo           OS: MacOS 15.5 arm64
               OMMM0,            Host: Mac15,6
     .;loddo:' loolloddol;.      Kernel: 24.5.0
   cKMMMMMMMMMMNWMMMMMMMMMM0:    Uptime: 32 days, 17 hours
 .KMMMMMMMMMMMMMMMMMMMMMMMWd.    Packages: 214 (brew)
 XMMMMMMMMMMMMMMMMMMMMMMMX.      Shell: zsh
;MMMMMMMMMMMMMMMMMMMMMMMM:       Resolution: 1800x1169, 2560x1440
:MMMMMMMMMMMMMMMMMMMMMMMM:       DE: Aqua
.MMMMMMMMMMMMMMMMMMMMMMMMX.      WM: Quartz Compositor
 kMMMMMMMMMMMMMMMMMMMMMMMMWd.    WM Theme: Blue (Dark)
 .XMMMMMMMMMMMMMMMMMMMMMMMMMMk   Terminal: WezTerm
  .XMMMMMMMMMMMMMMMMMMMMMMMMK.   CPU: Apple M3 Pro (11)
    kMMMMMMMMMMMMMMMMMMMMMMd     GPU: Apple M3 Pro
     ;KMMMMMMMWXXWMMMMMMMk.      Memory: 7083MiB / 36864MiB
       .cooc,.    .,coo:.        Disk (/): 753GiB / 926GiB (81%)
                                 Local IP: 192.168.1.100
```

## Install

No Rust required — prebuilt binaries for macOS and Linux:

```sh
curl -fsSL https://gustafeden.github.io/rsfetch/install.sh | bash
```

Supports macOS (Intel + Apple Silicon) and Linux (x86_64 + aarch64). Downloads a prebuilt binary to `~/.local/bin` (or the first writable directory in your `$PATH` under `$HOME`).

To install a specific version:

```sh
RSFETCH_VERSION=0.1.0 curl -fsSL https://gustafeden.github.io/rsfetch/install.sh | bash
```

Or with cargo (requires Rust):

```sh
cargo install --git https://github.com/gustafeden/rsfetch
```

Or build from source:

```sh
git clone https://github.com/gustafeden/rsfetch
cd rsfetch
cargo build --release
./target/release/rsfetch
```

## Usage

```
rsfetch [OPTIONS]

Options:
  -c, --color <COLOR>    Color theme (green, cyan, red, magenta, yellow, blue, mono)
  -l, --logo <LOGO>      Logo to display (apple, linux, ubuntu, arch, debian, fedora, none)
      --logo-file <PATH> Path to custom ASCII art file
      --no-logo          Hide the logo
      --json             Output as JSON
      --clear-cache      Clear the cache and re-gather all info
      --config <PATH>    Path to config file
      --print-config     Print default config to stdout
      --boot             Boot sequence mode (retro console animation)
      --left             Boot screen alignment: left (default)
      --center           Boot screen alignment: center
      --right            Boot screen alignment: right
      --animate          Animate the logo (color cycling)
  -h, --help             Print help
  -V, --version          Print version
```

CLI flags override config file values.

### Color themes

`green` (default), `cyan`, `red`, `magenta`, `yellow`, `blue`, `mono`

```sh
rsfetch -c cyan
rsfetch -c red
```

### Logos

Auto-detected by OS. Override with `--logo`:

`apple`, `linux`, `ubuntu`, `arch`, `debian`, `fedora`, `none`

```sh
rsfetch --logo arch
rsfetch --no-logo
```

### Custom ASCII art

Use any text file as a logo:

```sh
rsfetch --logo-file ~/my-logo.txt
```

Or set it in the config:

```toml
logo_file = "~/.config/rsfetch/logo.txt"
```

### Boot mode

A retro console-inspired boot animation. Shows a starfield with earth and moon by default, or a custom background image converted to half-block ASCII art.

```sh
rsfetch --boot
rsfetch --boot --center
```

Add it to your shell RC file (`.zshrc`, `.bashrc`) for a startup splash:

```sh
rsfetch --boot
```

Press any key to dismiss, or it auto-closes after the configured timeout.

#### Background image

Point `[boot] image` at a PNG or JPEG to use it as the background. The image is converted to half-block characters with full RGB color. Stars in the image (isolated bright pixels against dark space) will twinkle.

```toml
[boot]
image = "~/.config/rsfetch/space.png"
stretch = "fill"        # fill, fit, or crop
transparency = 0        # 0 = black bg, 1-255 = dark pixels transparent
```

Without an image, rsfetch renders a procedural starfield with earth and moon.

#### Boot config reference

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `image` | string | — | Path to background image (PNG/JPEG) |
| `stretch` | string | `"fill"` | `fill` (stretch), `fit` (letterbox), `crop` |
| `transparency` | int | `0` | Dark pixel threshold: 0 = opaque black, 1-255 = transparent |
| `width` | int | auto/68 | Canvas width in columns |
| `height` | int | auto/23 | Canvas height in rows |
| `timeout` | int | `120` | Seconds before auto-close |
| `star_brightness` | int | `30` | Star detection threshold (0-255) |
| `render_mode` | string | `"auto"` | `auto` (detect terminal), `image` (force iTerm2), `ascii` (force half-block) |
| `align` | string | `"left"` | `left`, `center`, or `right` (CLI flags override) |
| `min_width` | int | `10` | Minimum canvas width in columns |
| `min_height` | int | `5` | Minimum canvas height in rows |
| `max_width` | int | — | Maximum canvas width in columns |
| `max_height` | int | — | Maximum canvas height in rows |

### JSON output

```sh
rsfetch --json
```

### Configuration

rsfetch loads config from `~/.config/rsfetch/config.toml`. Generate a default config:

```sh
rsfetch --print-config > ~/.config/rsfetch/config.toml
```

Example config:

```toml
color = "cyan"
logo = "arch"
palette = false
separator = "="

# Show only these fields, in this order
fields = [
    "OS",
    "CPU",
    "Memory",
    "Disk (/)",
]

# Rename fields
[labels]
"Disk (/)" = "Disk"
"Local IP" = "IP"

# Custom colors (named or RGB)
[colors]
title = [255, 165, 0]
label = "cyan"
```

Config options:

| Key | Type | Description |
|-----|------|-------------|
| `color` | string | Color theme name |
| `logo` | string | Logo name or `"auto"` |
| `logo_file` | string | Path to custom ASCII art file |
| `palette` | bool | Show color palette (default: true) |
| `separator` | string | Separator character (default: `"-"`) |
| `fields` | list | Fields to show, in order |
| `labels` | table | Rename any field label |
| `colors.title` | string/rgb | Title color |
| `colors.label` | string/rgb | Label color |
| `colors.separator` | string/rgb | Separator color |
| `colors.logo` | string/rgb | Logo color |

Color values can be a named color (`"cyan"`, `"light_red"`, etc.) or an RGB array (`[255, 165, 0]`).

## Fields

| Field | macOS | Linux |
|-------|-------|-------|
| OS | version + arch | distro + arch |
| Host | hardware model | DMI product name |
| Kernel | Darwin version | Linux version |
| Uptime | days, hours, mins | days, hours, mins |
| Packages | brew | dpkg, pacman, rpm, flatpak, snap |
| Shell | name + version | name + version |
| Resolution | CoreGraphics API | - |
| DE | Aqua | XDG_CURRENT_DESKTOP |
| WM | Quartz Compositor | XDG_SESSION_TYPE |
| WM Theme | accent color + dark/light | GTK theme |
| Terminal | TERM_PROGRAM | TERM_PROGRAM |
| CPU | model + core count | model + core count |
| GPU | SoC name | lspci VGA |
| Memory | used / total MiB | used / total MiB |
| Disk | root filesystem GiB | root filesystem GiB |
| Local IP | first non-loopback IPv4 | first non-loopback IPv4 |

## How it's fast

- Compiled Rust binary — no interpreter startup
- Direct syscalls for host model, disk stats, display resolution (no subprocess spawning)
- Boot-cycle cache (`~/.cache/rsfetch/cache`) for fields that don't change between reboots
- Only two subprocess calls on macOS (`defaults read` for theme), and those are cached
- sysinfo crate for memory/CPU instead of parsing command output

## Dependencies

- [blaeck](https://github.com/gustafeden/blaeck) — inline terminal UI framework
- [sysinfo](https://crates.io/crates/sysinfo) — cross-platform system information
- [clap](https://crates.io/crates/clap) — CLI argument parsing
- [image](https://crates.io/crates/image) — image loading and processing (boot mode backgrounds)

## Releasing

1. Bump `version` in `Cargo.toml` and `installer/Cargo.toml`
2. Commit: `git commit -am "v0.2.0"`
3. Tag and push: `git tag v0.2.0 && git push && git push origin v0.2.0`

GitHub Actions builds binaries for macOS/Linux (Intel + ARM) and creates the release.

## License

MIT

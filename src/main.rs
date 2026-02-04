mod animate;
mod boot;
mod cache;
mod color;
mod config;
mod info;
mod logo;
mod mode;
mod render;

use clap::Parser;
use color::Theme;
use config::Config;
use info::SystemInfo;
use mode::Mode;

/// A fast system fetch display, written in Rust.
#[derive(Parser)]
#[command(name = "blaeckfetch", version, about)]
struct Args {
    /// Color theme (green, cyan, red, magenta, yellow, blue, mono)
    #[arg(short, long)]
    color: Option<String>,

    /// Logo to display (apple, linux, ubuntu, arch, debian, fedora, none)
    #[arg(short, long)]
    logo: Option<String>,

    /// Path to custom ASCII art file
    #[arg(long)]
    logo_file: Option<String>,

    /// Hide the logo
    #[arg(long)]
    no_logo: bool,

    /// Output as JSON
    #[arg(long)]
    json: bool,

    /// Clear the cache and re-gather all info
    #[arg(long)]
    clear_cache: bool,

    /// Path to config file
    #[arg(long)]
    config: Option<String>,

    /// Print default config to stdout
    #[arg(long)]
    print_config: bool,

    /// Animate the logo (background color cycling)
    #[arg(long)]
    animate: bool,

    /// Display mode (default, neofetch, splash)
    #[arg(long)]
    mode: Option<String>,

    /// Use neofetch mode (auto OS logo + all fields)
    #[arg(long)]
    neofetch: bool,

    /// Splash mode (fullscreen retro console animation)
    #[arg(long)]
    splash: bool,

    /// Alias for --splash (hidden)
    #[arg(long, hide = true)]
    boot: bool,

    /// Splash screen alignment: left (default)
    #[arg(long)]
    left: bool,

    /// Splash screen alignment: center
    #[arg(long)]
    center: bool,

    /// Splash screen alignment: right
    #[arg(long)]
    right: bool,
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();

    if args.print_config {
        print!("{}", config::generate_default());
        return Ok(());
    }

    if args.clear_cache {
        cache::clear();
    }

    let cfg = Config::load(args.config.as_deref());

    let info = SystemInfo::gather();

    if args.json {
        println!("{}", info.to_json());
        return Ok(());
    }

    // CLI flags override config values
    let color_name = args
        .color
        .or(cfg.color.clone())
        .unwrap_or_else(|| "green".into());

    let mut theme = Theme::by_name(&color_name);

    // Apply custom color overrides from config
    if let Some(colors) = &cfg.colors {
        if let Some(c) = &colors.title {
            theme.title = c.to_color();
        }
        if let Some(c) = &colors.label {
            theme.label = c.to_color();
        }
        if let Some(c) = &colors.separator {
            theme.separator = c.to_color();
        }
        if let Some(c) = &colors.logo {
            theme.logo = c.to_color();
        }
    }

    // Resolve mode: shorthand flags > --mode > config > Default
    let mode = if args.neofetch {
        Mode::Neofetch
    } else if args.splash || args.boot {
        Mode::Splash
    } else if let Some(ref mode_str) = args.mode {
        Mode::from_str(mode_str).unwrap_or_else(|| {
            eprintln!("warning: unknown mode '{}', using 'default'", mode_str);
            Mode::Default
        })
    } else if let Some(ref mode_str) = cfg.mode {
        Mode::from_str(mode_str).unwrap_or(Mode::Default)
    } else {
        Mode::Default
    };

    // Resolve logo (mode-aware)
    let logo_file = args.logo_file.or(cfg.logo_file.clone());

    let logo_art = if args.no_logo {
        String::new()
    } else if let Some(path) = &logo_file {
        logo::from_file(path).unwrap_or_else(|e| {
            eprintln!("warning: could not read logo file {}: {}", path, e);
            logo::detect().art.to_string()
        })
    } else {
        let logo_name = args.logo.or(cfg.logo.clone());
        match logo_name.as_deref() {
            Some("none" | "off") => String::new(),
            Some("auto") => logo::detect().art.to_string(),
            Some(name) => logo::by_name(name).art.to_string(),
            None => match mode {
                Mode::Default => String::new(), // render.rs builds the colored moon element
                Mode::Neofetch => logo::detect().art.to_string(),
                Mode::Splash => String::new(), // splash has its own rendering
            },
        }
    };

    if mode == Mode::Splash {
        let boot_cfg = cfg.splash_config();

        // Resolve alignment: CLI flags win, then config, then default "left"
        let align = if args.center {
            "center"
        } else if args.left {
            "left"
        } else if args.right {
            "right"
        } else {
            boot_cfg.and_then(|b| b.align.as_deref()).unwrap_or("left")
        };
        let centered = align == "center";
        let right_aligned = align == "right";

        let cfg_w = boot_cfg.and_then(|b| b.width);
        let cfg_h = boot_cfg.and_then(|b| b.height);

        // Min/max constraints
        let min_w = boot_cfg.and_then(|b| b.min_width).unwrap_or(10);
        let min_h = boot_cfg.and_then(|b| b.min_height).unwrap_or(5);
        let max_w = boot_cfg.and_then(|b| b.max_width).unwrap_or(u16::MAX);
        let max_h = boot_cfg.and_then(|b| b.max_height).unwrap_or(u16::MAX);

        // Detect render mode
        let config_render_mode = boot_cfg.and_then(|b| b.render_mode.as_deref());
        let render_mode = boot::image_proto::detect(config_render_mode);

        let img_path_owned = boot_cfg.and_then(|b| b.image.clone());
        let stretch_owned = boot_cfg.and_then(|b| b.stretch.clone()).unwrap_or_else(|| "fill".into());

        let (bg, raw_image, boot_w, boot_h) = if let Some(ref img_path) = img_path_owned {
            let stretch = stretch_owned.as_str();
            if !matches!(stretch, "fill" | "fit" | "crop") {
                eprintln!("warning: unknown stretch mode '{}', using 'fill'", stretch);
            }

            // If no explicit size, derive from image aspect ratio
            let (w, h) = if cfg_w.is_some() || cfg_h.is_some() {
                (cfg_w.unwrap_or(68), cfg_h.unwrap_or(23))
            } else if render_mode == boot::image_proto::RenderMode::Image {
                // Use cell pixel ratio for accurate sizing in image mode
                // 2 border cols, 6 overhead rows (top/status/upper-sep/lower-sep/footer/bottom)
                let (cpw, cph) = boot::image_proto::cell_pixel_size().unwrap_or((8, 16));
                let (term_w, term_h) = boot::terminal_size();
                boot::background::image_cell_size_for_proto(img_path, cpw, cph)
                    .map(|(cw, ch)| boot::background::fit_to_terminal_with_chrome(cw, ch, term_w, term_h, 2, 6))
                    .unwrap_or((68, 23))
            } else if render_mode == boot::image_proto::RenderMode::Inline {
                // Inline mode (VHS): derive from image aspect ratio with a fixed virtual terminal
                // since terminal_size() is unreliable in non-interactive environments.
                // Use 80x24 as a conservative virtual terminal size.
                boot::background::image_cell_size(img_path)
                    .map(|(cw, ch)| boot::background::fit_to_terminal_with_chrome(cw, ch, 80, 24, 2, 6))
                    .unwrap_or((68, 23))
            } else {
                boot::background::image_dimensions(img_path).unwrap_or((68, 23))
            };

            // Apply min/max constraints
            let w = w.clamp(min_w, max_w);
            let h = h.clamp(min_h, max_h);

            if render_mode == boot::image_proto::RenderMode::Image {
                // Image mode: load raw PNG bytes sized for interior area
                // (rows 3..h-2, cols 1..w-1)
                let bh = h + 1; // matches the +1 in boot::run for image mode
                let interior_w = w.saturating_sub(2);
                let interior_h = bh.saturating_sub(6); // between separators
                // Resize to exact pixel dimensions matching the cell area
                let (cell_pw, cell_ph) = boot::image_proto::cell_pixel_size()
                    .unwrap_or((8, 16));
                let px_w = (interior_w as u32) * (cell_pw as u32);
                let px_h = (interior_h as u32) * (cell_ph as u32);
                let raw = boot::background::load_raw(img_path, px_w, px_h, stretch);
                (None, raw, Some(w), Some(h))
            } else {
                // ASCII mode: load half-block cells
                let transparency = boot_cfg.and_then(|b| b.transparency).unwrap_or(0);
                let star_brightness = boot_cfg.and_then(|b| b.star_brightness);
                let cells = boot::background::load(img_path, w, h, stretch, transparency, star_brightness);
                (cells, None, Some(w), Some(h))
            }
        } else {
            (None, None, cfg_w, cfg_h)
        };

        let timeout = boot_cfg.and_then(|b| b.timeout);
        let image_source = if render_mode == boot::image_proto::RenderMode::Image {
            img_path_owned.as_ref().map(|p| (p.as_str(), stretch_owned.as_str()))
        } else {
            None
        };
        let image_cell_size = if render_mode == boot::image_proto::RenderMode::Image {
            let (cpw, cph) = boot::image_proto::cell_pixel_size().unwrap_or((8, 16));
            img_path_owned.as_ref().and_then(|p| boot::background::image_cell_size_for_proto(p, cpw, cph))
        } else {
            None
        };
        let vhs_mode = render_mode == boot::image_proto::RenderMode::Inline;
        let entrance = boot_cfg.and_then(|b| b.entrance.as_deref()).unwrap_or("slow");
        let exit = boot_cfg.and_then(|b| b.exit.as_deref()).unwrap_or("slow");
        boot::run(&info, centered, right_aligned, bg, raw_image, render_mode, boot_w, boot_h, timeout, image_source, image_cell_size, (min_w, min_h), (max_w, max_h), entrance, exit, vhs_mode);
        return Ok(());
    }

    if args.animate {
        render::render_animated(&info, &logo_art, &theme, &cfg, mode)
    } else {
        render::render(&info, &logo_art, &theme, &cfg, mode)
    }
}

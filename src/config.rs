use blaeck::prelude::Color;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Deserialize, Default)]
#[serde(default)]
pub struct Config {
    pub mode: Option<String>,
    pub color: Option<String>,
    pub logo: Option<String>,
    pub logo_file: Option<String>,
    pub palette: Option<bool>,
    pub separator: Option<String>,
    pub label_align: Option<String>,
    pub label_position: Option<String>,
    pub value_align: Option<String>,
    pub field_separator: Option<String>,
    pub fields: Option<Vec<String>>,
    pub labels: Option<HashMap<String, String>>,
    pub colors: Option<ColorsConfig>,
    pub splash: Option<BootConfig>,
    pub boot: Option<BootConfig>,
}

#[derive(Deserialize, Default)]
#[serde(default)]
pub struct BootConfig {
    pub image: Option<String>,
    pub stretch: Option<String>,
    pub transparency: Option<u8>,
    pub width: Option<u16>,
    pub height: Option<u16>,
    pub timeout: Option<u64>,
    pub star_brightness: Option<u8>,
    pub render_mode: Option<String>,
    pub align: Option<String>,
    pub min_width: Option<u16>,
    pub min_height: Option<u16>,
    pub max_width: Option<u16>,
    pub max_height: Option<u16>,
    pub entrance: Option<String>,
    pub exit: Option<String>,
}

#[derive(Deserialize, Default)]
#[serde(default)]
pub struct ColorsConfig {
    pub title: Option<ColorValue>,
    pub label: Option<ColorValue>,
    pub separator: Option<ColorValue>,
    pub logo: Option<ColorValue>,
}

#[derive(Deserialize, Clone)]
#[serde(untagged)]
pub enum ColorValue {
    Named(String),
    Rgb([u8; 3]),
}

impl ColorValue {
    pub fn to_color(&self) -> Color {
        match self {
            ColorValue::Named(name) => match name.to_lowercase().as_str() {
                "black" => Color::Black,
                "red" => Color::Red,
                "green" => Color::Green,
                "yellow" => Color::Yellow,
                "blue" => Color::Blue,
                "magenta" => Color::Magenta,
                "cyan" => Color::Cyan,
                "white" => Color::White,
                "dark_gray" | "darkgray" => Color::DarkGray,
                "light_red" | "lightred" => Color::LightRed,
                "light_green" | "lightgreen" => Color::LightGreen,
                "light_yellow" | "lightyellow" => Color::LightYellow,
                "light_blue" | "lightblue" => Color::LightBlue,
                "light_magenta" | "lightmagenta" => Color::LightMagenta,
                "light_cyan" | "lightcyan" => Color::LightCyan,
                _ => Color::White,
            },
            ColorValue::Rgb(rgb) => Color::Rgb(rgb[0], rgb[1], rgb[2]),
        }
    }
}

impl Config {
    pub fn load(path: Option<&str>) -> Self {
        let config_path = match path {
            Some(p) => PathBuf::from(p),
            None => default_config_path(),
        };

        if !config_path.exists() {
            return Config::default();
        }

        match std::fs::read_to_string(&config_path) {
            Ok(content) => toml::from_str(&content).unwrap_or_else(|e| {
                eprintln!("warning: invalid config {}: {}", config_path.display(), e);
                Config::default()
            }),
            Err(_) => Config::default(),
        }
    }

    pub fn default_fields() -> Vec<String> {
        vec![
            "OS".into(),
            "Host".into(),
            "Kernel".into(),
            "Uptime".into(),
            "Packages".into(),
            "Shell".into(),
            "Resolution".into(),
            "DE".into(),
            "WM".into(),
            "WM Theme".into(),
            "Terminal".into(),
            "CPU".into(),
            "GPU".into(),
            "Memory".into(),
            "Disk (/)".into(),
            "Local IP".into(),
        ]
    }

    pub fn label_for(&self, key: &str) -> String {
        self.labels
            .as_ref()
            .and_then(|m| m.get(key).cloned())
            .unwrap_or_else(|| key.to_string())
    }

    /// Get splash config, checking both [splash] and [boot] sections.
    pub fn splash_config(&self) -> Option<&BootConfig> {
        self.splash.as_ref().or(self.boot.as_ref())
    }

    /// Get active fields for a given mode.
    pub fn active_fields_for_mode(&self, mode: crate::mode::Mode) -> Vec<String> {
        self.fields
            .clone()
            .unwrap_or_else(|| mode.default_fields())
    }

    /// Whether to show palette for a given mode.
    pub fn show_palette_for_mode(&self, mode: crate::mode::Mode) -> bool {
        self.palette.unwrap_or_else(|| mode.default_palette())
    }
}

pub fn default_config_path() -> PathBuf {
    dirs_config().join("config.toml")
}

fn dirs_config() -> PathBuf {
    if let Ok(home) = std::env::var("HOME") {
        PathBuf::from(home).join(".config").join("blaeckfetch")
    } else {
        PathBuf::from(".config/blaeckfetch")
    }
}

pub fn generate_default() -> String {
    r#"# blaeckfetch configuration
# Place this file at ~/.config/blaeckfetch/config.toml

# Display mode: default (moon + minimal), neofetch (classic layout), splash (animation)
# mode = "default"

# Color theme: green, cyan, red, magenta, yellow, blue, mono
# color = "green"

# Logo: moon, apple, linux, ubuntu, arch, debian, fedora, none, auto
# logo = "auto"

# Custom ASCII art file (overrides logo)
# logo_file = "~/.config/blaeckfetch/logo.txt"

# Show color palette at bottom
# palette = true

# Separator character
# separator = "-"

# Label alignment: left (default) or right (colons line up)
# label_align = "right"

# Label position: left (default) or right (value first, then label)
# label_position = "right"

# Value alignment: left (default) or right (values form a right-aligned column)
# value_align = "right"

# String between label and value (default: ": ")
# Use "fill" for a line that fills the gap: OS ──────── MacOS 15.5
# field_separator = ": "
# field_separator = " → "
# field_separator = "fill"

# Fields to display (in order). Remove or comment out entries to hide them.
# fields = [
#     "OS",
#     "Host",
#     "Kernel",
#     "Uptime",
#     "Packages",
#     "Shell",
#     "Resolution",
#     "DE",
#     "WM",
#     "WM Theme",
#     "Terminal",
#     "CPU",
#     "GPU",
#     "Memory",
#     "Disk (/)",
#     "Local IP",
# ]

# Custom field labels (rename any field)
# [labels]
# "Disk (/)" = "Disk"
# "Local IP" = "IP"

# Custom colors (override theme)
# Values: named color string or RGB array [r, g, b]
# Named colors: black, red, green, yellow, blue, magenta, cyan, white,
#               dark_gray, light_red, light_green, light_yellow,
#               light_blue, light_magenta, light_cyan
# [colors]
# title = "cyan"
# label = [100, 200, 255]
# separator = "dark_gray"
# logo = "green"

# Splash mode (retro console animation)
# Run with: blaeckfetch --splash
# Use --center or --left to control alignment.
# Without an image, shows a procedural starfield with earth and moon.
# [splash]
# image = "~/.config/blaeckfetch/space.png"   # Background image (PNG/JPEG)
# stretch = "fill"                         # fill (stretch), fit (letterbox), or crop
# transparency = 0                         # 0 = opaque black bg, 1-255 = dark pixels become transparent
# width = 68                               # Canvas width in columns (default: auto from image or 68)
# height = 23                              # Canvas height in rows (default: auto from image or 23)
# timeout = 120                            # Seconds before auto-closing (triggers collapse animation)
# star_brightness = 30                     # Star twinkle detection threshold (0-255)
# render_mode = "auto"                     # auto (detect terminal), image (force iTerm2), ascii (force half-block)
# align = "left"                           # left, center, or right (CLI --left/--center/--right override)
# min_width = 40                           # Minimum canvas width in columns
# min_height = 12                          # Minimum canvas height in rows
# max_width = 120                          # Maximum canvas width in columns
# max_height = 40                          # Maximum canvas height in rows
# entrance = "slow"                        # Entrance animation: slow (~1.2s), fast (~400ms), instant
# exit = "slow"                            # Exit animation: slow (~400ms), fast (~200ms), instant
"#
    .to_string()
}

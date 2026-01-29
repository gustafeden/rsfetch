use blaeck::prelude::Color;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Deserialize, Default)]
#[serde(default)]
pub struct Config {
    pub color: Option<String>,
    pub logo: Option<String>,
    pub logo_file: Option<String>,
    pub palette: Option<bool>,
    pub separator: Option<String>,
    pub fields: Option<Vec<String>>,
    pub labels: Option<HashMap<String, String>>,
    pub colors: Option<ColorsConfig>,
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

    pub fn show_palette(&self) -> bool {
        self.palette.unwrap_or(true)
    }

    pub fn separator_char(&self) -> &str {
        self.separator.as_deref().unwrap_or("-")
    }

    pub fn active_fields(&self) -> Vec<String> {
        self.fields
            .clone()
            .unwrap_or_else(Config::default_fields)
    }

    pub fn label_for(&self, key: &str) -> String {
        self.labels
            .as_ref()
            .and_then(|m| m.get(key).cloned())
            .unwrap_or_else(|| key.to_string())
    }
}

pub fn default_config_path() -> PathBuf {
    dirs_config().join("config.toml")
}

fn dirs_config() -> PathBuf {
    if let Ok(home) = std::env::var("HOME") {
        PathBuf::from(home).join(".config").join("rsfetch")
    } else {
        PathBuf::from(".config/rsfetch")
    }
}

pub fn generate_default() -> String {
    r#"# rsfetch configuration
# Place this file at ~/.config/rsfetch/config.toml

# Color theme: green, cyan, red, magenta, yellow, blue, mono
# color = "green"

# Logo: apple, linux, ubuntu, arch, debian, fedora, none, auto
# logo = "auto"

# Custom ASCII art file (overrides logo)
# logo_file = "~/.config/rsfetch/logo.txt"

# Show color palette at bottom
# palette = true

# Separator character
# separator = "-"

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
"#
    .to_string()
}

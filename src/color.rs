use blaeck::prelude::*;

#[derive(Clone)]
#[allow(dead_code)]
pub struct Theme {
    pub name: &'static str,
    pub title: Color,
    pub label: Color,
    pub separator: Color,
    pub logo: Color,
}

impl Theme {
    pub fn by_name(name: &str) -> Self {
        match name.to_lowercase().as_str() {
            "cyan" => Self::cyan(),
            "red" => Self::red(),
            "magenta" | "pink" => Self::magenta(),
            "yellow" => Self::yellow(),
            "blue" => Self::blue(),
            "white" | "mono" => Self::mono(),
            _ => Self::default_green(),
        }
    }

    pub fn default_green() -> Self {
        Self {
            name: "green",
            title: Color::Green,
            label: Color::Green,
            separator: Color::DarkGray,
            logo: Color::Green,
        }
    }

    pub fn cyan() -> Self {
        Self {
            name: "cyan",
            title: Color::Cyan,
            label: Color::Cyan,
            separator: Color::DarkGray,
            logo: Color::Cyan,
        }
    }

    pub fn red() -> Self {
        Self {
            name: "red",
            title: Color::Red,
            label: Color::Red,
            separator: Color::DarkGray,
            logo: Color::Red,
        }
    }

    pub fn magenta() -> Self {
        Self {
            name: "magenta",
            title: Color::Magenta,
            label: Color::Magenta,
            separator: Color::DarkGray,
            logo: Color::Magenta,
        }
    }

    pub fn yellow() -> Self {
        Self {
            name: "yellow",
            title: Color::Yellow,
            label: Color::Yellow,
            separator: Color::DarkGray,
            logo: Color::Yellow,
        }
    }

    pub fn blue() -> Self {
        Self {
            name: "blue",
            title: Color::Blue,
            label: Color::Blue,
            separator: Color::DarkGray,
            logo: Color::Blue,
        }
    }

    pub fn mono() -> Self {
        Self {
            name: "mono",
            title: Color::White,
            label: Color::White,
            separator: Color::DarkGray,
            logo: Color::White,
        }
    }

    #[allow(dead_code)]
    pub fn available() -> &'static [&'static str] {
        &["green", "cyan", "red", "magenta", "yellow", "blue", "mono"]
    }
}

/// Map a blaeck Color to a basic ANSI color number (0-7).
pub fn color_to_ansi(c: &Color) -> u8 {
    match c {
        Color::Black => 0,
        Color::Red => 1,
        Color::Green => 2,
        Color::Yellow => 3,
        Color::Blue => 4,
        Color::Magenta => 5,
        Color::Cyan => 6,
        Color::White => 7,
        Color::DarkGray => 0,
        Color::LightRed => 1,
        Color::LightGreen => 2,
        Color::LightYellow => 3,
        Color::LightBlue => 4,
        Color::LightMagenta => 5,
        Color::LightCyan => 6,
        _ => 2, // default to green
    }
}

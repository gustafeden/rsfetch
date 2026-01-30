use blaeck::prelude::*;
use blaeck::Blaeck;
use std::io;

use crate::animate;
use crate::color::{self, Theme};
use crate::config::Config;
use crate::info::SystemInfo;

pub fn render(info: &SystemInfo, logo: &str, theme: &Theme, cfg: &Config) -> io::Result<()> {
    let title = info.title();
    let sep_char = cfg.separator_char();
    let separator = sep_char.repeat(title.len());
    let all_fields = info.fields();
    let active = cfg.active_fields();

    let mut info_elements: Vec<Element> = Vec::new();

    // Title
    info_elements.push(element! {
        Text(content: title, color: theme.title, bold: true)
    });
    info_elements.push(element! {
        Text(content: separator, color: theme.separator)
    });

    // Info fields — filtered and ordered by config
    for key in &active {
        if let Some((_, value)) = all_fields.iter().find(|(k, _)| *k == key.as_str()) {
            let label = cfg.label_for(key);
            info_elements.push(element! {
                Box(flex_direction: FlexDirection::Row) {
                    Text(content: format!("{}: ", label), color: theme.label, bold: true)
                    Text(content: value.to_string())
                }
            });
        }
    }

    // Color palette
    if cfg.show_palette() {
        info_elements.push(element! { Text(content: "") });
        info_elements.push(element! {
            Box(flex_direction: FlexDirection::Row) {
                Text(content: "███", color: Color::Black)
                Text(content: "███", color: Color::Red)
                Text(content: "███", color: Color::Green)
                Text(content: "███", color: Color::Yellow)
                Text(content: "███", color: Color::Blue)
                Text(content: "███", color: Color::Magenta)
                Text(content: "███", color: Color::Cyan)
                Text(content: "███", color: Color::White)
            }
        });
        info_elements.push(element! {
            Box(flex_direction: FlexDirection::Row) {
                Text(content: "███", color: Color::DarkGray)
                Text(content: "███", color: Color::LightRed)
                Text(content: "███", color: Color::LightGreen)
                Text(content: "███", color: Color::LightYellow)
                Text(content: "███", color: Color::LightBlue)
                Text(content: "███", color: Color::LightMagenta)
                Text(content: "███", color: Color::LightCyan)
                Text(content: "███", color: Color::White)
            }
        });
    }

    let ui = Element::node::<Box>(
        BoxProps {
            flex_direction: FlexDirection::Row,
            ..Default::default()
        },
        vec![
            element! {
                Text(content: logo, color: theme.logo)
            },
            element! {
                Text(content: "   ")
            },
            Element::node::<Box>(
                BoxProps {
                    flex_direction: FlexDirection::Column,
                    ..Default::default()
                },
                info_elements,
            ),
        ],
    );

    let mut blaeck = Blaeck::new(io::stdout())?;
    blaeck.render(ui)?;
    blaeck.unmount()?;

    Ok(())
}

pub fn render_animated(info: &SystemInfo, logo: &str, theme: &Theme, cfg: &Config) -> io::Result<()> {
    // Normal render first
    render(info, logo, theme, cfg)?;

    if logo.is_empty() {
        return Ok(());
    }

    // Calculate geometry
    let logo_lines = logo.lines().count();
    let info_lines = {
        let active = cfg.active_fields();
        let mut count = 2; // title + separator
        count += active.len();
        if cfg.show_palette() {
            count += 3; // blank + 2 palette rows
        }
        count
    };
    let total_height = std::cmp::max(logo_lines, info_lines) as u16;

    let original_color = color::color_to_ansi(&theme.logo);

    // Animate in foreground — blocks until keypress, then exits
    animate::run_foreground(logo, total_height, original_color);

    Ok(())
}

pub mod background;
mod border;
mod canvas;
mod earth;
pub mod image_proto;
mod moon;
mod rng;
mod starfield;
mod status;
mod timeline;
use std::io::{self, Write};
use std::os::unix::io::FromRawFd;

use crate::info::SystemInfo;
use canvas::Canvas;
use timeline::{Phase, Timeline};

const DEFAULT_WIDTH: u16 = 68;
const DEFAULT_HEIGHT: u16 = 23;
const FPS: u64 = 30;
const PAD_LEFT: u16 = 1;
const PAD_TOP: u16 = 0;

// Colors
const BORDER_COLOR: (u8, u8, u8) = (100, 100, 120);
const STATUS_COLOR: (u8, u8, u8) = (140, 140, 140);
const FOOTER_COLOR: (u8, u8, u8) = (140, 140, 140);

pub fn run(
    info: &SystemInfo,
    centered: bool,
    right_aligned: bool,
    mut bg: Option<Vec<background::BgCell>>,
    mut raw_image: Option<Vec<u8>>,
    render_mode: image_proto::RenderMode,
    boot_w: Option<u16>,
    boot_h: Option<u16>,
    timeout_secs: Option<u64>,
    image_source: Option<(&str, &str)>, // (path, stretch) for re-rendering on resize
    image_cell_size: Option<(f32, f32)>, // natural cell dimensions for aspect ratio
    min_size: (u16, u16),
    max_size: (u16, u16),
    entrance: &str,
    exit: &str,
    vhs_mode: bool,
) {
    use std::os::unix::io::AsRawFd;

    // In VHS mode we skip /dev/tty and raw mode entirely — VHS handles ANSI
    // cursor positioning but can't do raw mode keypress detection.
    let fd: i32;
    let _tty_handle: Option<std::fs::File>;
    if !vhs_mode {
        let tty = match std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open("/dev/tty")
        {
            Ok(f) => f,
            Err(_) => return,
        };
        fd = tty.as_raw_fd();
        _tty_handle = Some(tty);
    } else {
        fd = -1;
        _tty_handle = None;
    }

    // Determine boot screen dimensions
    let base_bw = boot_w.unwrap_or(DEFAULT_WIDTH);
    let base_bh = boot_h.unwrap_or(DEFAULT_HEIGHT);
    let image_mode_flag = render_mode == image_proto::RenderMode::Image && raw_image.is_some();
    let image_extra_h: u16 = if image_mode_flag { 1 } else { 0 };
    let mut bw = base_bw;
    let mut bh = base_bh + image_extra_h;

    // Pre-build content
    let status_text = status::build_line(info);

    // Check terminal size
    let (term_width, term_height) = terminal_size();
    if !vhs_mode && term_width < bw {
        return; // terminal too narrow
    }

    let mut stdout = io::stdout();

    let mut origin_row: u16 = 1;
    let mut origin_col: u16 = 1;
    let mut last_term_width = term_width;
    let mut last_term_height = term_height;

    if vhs_mode {
        // VHS: start at top-left, no space reservation needed
        origin_row = 1;
        origin_col = 1;
    } else {
        // Reserve vertical space (content + top padding) by printing newlines.
        let total_height = PAD_TOP + bh;
        let reserve = total_height.min(term_height);
        for _ in 0..reserve.saturating_sub(1) {
            let _ = write!(stdout, "\n");
        }
        let _ = stdout.flush();

        // Query cursor row via DSR (requires temporary raw mode for reading response)
        let cursor_row = query_cursor_row(fd).unwrap_or(term_height);
        origin_row = if cursor_row >= reserve {
            cursor_row - reserve + 1 + PAD_TOP
        } else {
            1 + PAD_TOP
        };

        // Clear the reserved lines (removes any shell prompt text to the left)
        let clear_start = origin_row.saturating_sub(PAD_TOP);
        for r in clear_start..origin_row + bh {
            let _ = write!(stdout, "\x1b[{};1H\x1b[2K", r);
        }
        let _ = stdout.flush();
    }

    // Horizontal position (VHS: already set to 1 above)
    if !vhs_mode {
        origin_col = if centered {
            (term_width.saturating_sub(bw)) / 2 + 1
        } else if right_aligned {
            term_width.saturating_sub(bw).saturating_sub(PAD_LEFT) + 1
        } else {
            PAD_LEFT + 1
        };
    }

    // Enter raw mode (skip for VHS)
    let orig: libc::termios = if !vhs_mode {
        unsafe {
            let mut t: libc::termios = std::mem::zeroed();
            if libc::tcgetattr(fd, &mut t) != 0 {
                return;
            }
            let orig = t;
            t.c_lflag &= !(libc::ICANON | libc::ECHO);
            t.c_cc[libc::VMIN] = 0;
            t.c_cc[libc::VTIME] = 0;
            libc::tcsetattr(fd, libc::TCSANOW, &t);
            orig
        }
    } else {
        unsafe { std::mem::zeroed() }
    };

    // Hide cursor
    let _ = write!(stdout, "\x1b[?25l");
    let _ = stdout.flush();

    // --- Setup ---
    let mut canvas = Canvas::new(bw, bh);
    let image_mode = image_mode_flag;
    let mut image_emitted = false;

    let seed = std::process::id();
    let mut rng_state = rng::Rng::new(seed);

    // Starfield: fill the scene area
    let star_max_y = bh.saturating_sub(3).max(3);
    let mut stars = starfield::generate(1, 3, bw - 1, star_max_y, 0.02, &mut rng_state);

    // Earth: bottom-right of scene
    let earth_cx = bw as f32 - 14.0;
    let earth_cy = (bh as f32) * 0.6;
    let earth_radius = (bh as f32) * 0.26;

    // Moon: upper-left of scene
    let moon_cx = (bw as f32) * 0.15;
    let moon_cy = (bh as f32) * 0.26;
    let moon_radius = (bh as f32) * 0.11;

    let mut timeline = Timeline::new(entrance, exit);
    let timeout = std::time::Duration::from_secs(timeout_secs.unwrap_or(120));
    let start_time = std::time::Instant::now();
    let frame_duration = std::time::Duration::from_millis(1000 / FPS);

    // --- Frame loop ---
    let mut done_rendered = false;
    loop {
        let frame_start = std::time::Instant::now();

        if start_time.elapsed() > timeout {
            timeline.trigger_freeze();
        }
        if timeline.is_done() {
            if done_rendered {
                break;
            }
            done_rendered = true;
        }

        // Check for keypress (non-blocking) — skip in VHS mode
        if !vhs_mode {
            let mut buf = [0u8; 64];
            let n = unsafe { libc::read(fd, buf.as_mut_ptr() as *mut libc::c_void, buf.len()) };
            if n > 0 {
                timeline.trigger_freeze();
            }
        } else {
            // VHS mode: auto-trigger freeze after 3 seconds of blinking
            if matches!(timeline.phase, Phase::Alive) && start_time.elapsed() > std::time::Duration::from_secs(3) {
                timeline.trigger_freeze();
            }
        }

        // Handle terminal resize (skip in VHS — fixed size)
        let (cur_width, cur_height) = terminal_size();
        if !vhs_mode && (cur_width != last_term_width || cur_height != last_term_height) {
            // Wait for resize to settle (debounce)
            std::thread::sleep(std::time::Duration::from_millis(100));
            let (cur_width, cur_height) = terminal_size();

            // Clear entire visible area
            for row in 1..=cur_height {
                let _ = write!(stdout, "\x1b[{};1H\x1b[2K", row);
            }
            let _ = stdout.flush();
            last_term_width = cur_width;
            last_term_height = cur_height;

            // Recalculate box dimensions preserving aspect ratio
            if let Some((cw, ch)) = image_cell_size {
                let (new_w, new_h) = background::fit_to_terminal_with_chrome(cw, ch, cur_width, cur_height, 2, 6);
                bw = new_w;
                bh = new_h + image_extra_h;
            } else {
                bw = base_bw.min(cur_width.saturating_sub(1));
                bh = (base_bh + image_extra_h).min(cur_height.saturating_sub(1));
            }

            // Apply min/max constraints
            bw = bw.clamp(min_size.0, max_size.0);
            bh = bh.clamp(min_size.1, max_size.1);

            // Recalculate origin
            origin_row = 1 + PAD_TOP;
            if centered {
                origin_col = (cur_width.saturating_sub(bw)) / 2 + 1;
            } else if right_aligned {
                origin_col = cur_width.saturating_sub(bw).saturating_sub(PAD_LEFT) + 1;
            }

            // Rebuild canvas at new size
            canvas = Canvas::new(bw, bh);

            if image_mode {
                // Re-generate image for new dimensions
                if let Some((img_path, stretch)) = image_source {
                    let interior_w = bw.saturating_sub(2);
                    let interior_h = bh.saturating_sub(6);
                    let (cell_pw, cell_ph) = image_proto::cell_pixel_size()
                        .unwrap_or((8, 16));
                    let px_w = (interior_w as u32) * (cell_pw as u32);
                    let px_h = (interior_h as u32) * (cell_ph as u32);
                    if let Some(new_bytes) = background::load_raw(img_path, px_w, px_h, stretch) {
                        raw_image = Some(new_bytes);
                    }
                }
                image_emitted = false;
            }
        }

        // Clear canvas
        canvas.clear();

        let collapsing = timeline.is_collapsing() || timeline.is_done();

        if !collapsing {
            // --- Normal rendering (Entrance / Alive / Flash) ---

            // Emit inline image if in image mode
            if image_mode && !image_emitted {
                if let Some(ref raw_bytes) = raw_image {
                    // Interior area: same region as background::draw uses
                    // rows 3..bh-2, cols 1..bw-1
                    let img_x = 1u16;
                    let img_y = 3u16;
                    let img_w = bw.saturating_sub(2);
                    let img_h = bh.saturating_sub(6); // rows 3..bh-4 (between separators)
                    let mask_h = img_h;
                    canvas.set_image_mask(img_x, img_y, img_x + img_w, img_y + mask_h);
                    image_proto::emit(
                        &mut stdout,
                        raw_bytes,
                        img_w,
                        img_h,
                        origin_row + img_y,
                        origin_col + img_x,
                    );
                    image_emitted = true;
                }
            }

            // 1. Scene background (image or procedural)
            if image_mode {
                // Image mode: the inline image is already displayed, skip ASCII background
            } else if let Some(ref mut cells) = bg {
                background::twinkle(cells, &mut rng_state);
                background::draw(&mut canvas, cells, bw);
            } else {
                let star_vis = timeline.star_visibility();
                if star_vis > 0.0 {
                    if timeline.is_freeze_flash() {
                        starfield::flash_all(&mut stars);
                    } else if timeline.gradient_active() {
                        starfield::twinkle(&mut stars, &mut rng_state);
                    }
                    starfield::draw(&mut canvas, &stars, star_vis);
                }
                if timeline.celestial_visibility() > 0.3 {
                    earth::draw(&mut canvas, earth_cx, earth_cy, earth_radius);
                    moon::draw(&mut canvas, moon_cx, moon_cy, moon_radius);
                }
            }

            // 2. Border
            border::draw_border(&mut canvas, BORDER_COLOR, timeline.border_progress());

            // 3. Status bar
            let status_prog = timeline.status_progress();
            if status_prog > 0.0 {
                status::draw(&mut canvas, &status_text, 1, STATUS_COLOR, status_prog);
            }

            // 4. Footer
            if timeline.footer_visible() {
                let footer_text = if matches!(timeline.phase, Phase::Freeze) {
                    "ready"
                } else {
                    "press any key to start"
                };
                let footer_len = footer_text.chars().count() as u16;
                let footer_x = (bw - footer_len) / 2;
                canvas.put_str(
                    footer_x,
                    bh - 2,
                    footer_text,
                    FOOTER_COLOR,
                    None,
                );
            }
        } else {
            // --- Collapse: draw scene first, then overlay collapsing border on top ---
            let progress = timeline.collapse_progress();

            // Draw scene background
            if image_mode {
                // Image mode: unmask rows below the sliding border so spaces clear the image
                let max_travel = bh.saturating_sub(1).saturating_sub(2);
                let travel = (max_travel as f32 * progress) as u16;
                let bot = (bh - 1).saturating_sub(travel);
                if bot + 1 < bh {
                    canvas.unmask_region(0, bot + 1, bw, bh);
                }
            } else if let Some(ref mut cells) = bg {
                background::twinkle(cells, &mut rng_state);
                background::draw(&mut canvas, cells, bw);
            } else {
                starfield::draw(&mut canvas, &stars, 1.0);
                earth::draw(&mut canvas, earth_cx, earth_cy, earth_radius);
                moon::draw(&mut canvas, moon_cx, moon_cy, moon_radius);
            }

            // Draw collapsing border on top (clears everything below bottom border)
            draw_collapsed_frame(&mut canvas, &status_text, progress, bh);
        }

        // Render frame
        canvas.render(&mut stdout, origin_row, origin_col);

        // Advance timeline
        timeline.tick();

        // Frame timing
        let elapsed = frame_start.elapsed();
        if elapsed < frame_duration {
            std::thread::sleep(frame_duration - elapsed);
        }
    }

    // --- Cleanup ---
    if image_mode {
        canvas.clear_image_mask();
    }
    let final_box_rows = 3u16; // top border + status + bottom border
    for y in final_box_rows..bh {
        let _ = write!(
            stdout,
            "\x1b[{};{}H\x1b[2K",
            origin_row + y,
            1
        );
    }

    // Show cursor
    let _ = write!(stdout, "\x1b[?25h");
    let _ = stdout.flush();

    if !vhs_mode {
        unsafe {
            libc::tcsetattr(fd, libc::TCSANOW, &orig);
        }
    }

    // Position cursor right after the final box
    let _ = write!(stdout, "\x1b[{};1H", origin_row + final_box_rows);
    let _ = stdout.flush();
}

/// Draw the collapsing border frame.
fn draw_collapsed_frame(canvas: &mut Canvas, status_text: &str, progress: f32, bh: u16) {
    let w = canvas.width;
    let max_travel = bh - 1 - 2;
    let travel = (max_travel as f32 * progress) as u16;
    let bot = (bh - 1).saturating_sub(travel);

    // --- Top border ---
    canvas.set(0, 0, '╭', BORDER_COLOR, None);
    for x in 1..w - 1 {
        canvas.set(x, 0, '─', BORDER_COLOR, None);
    }
    canvas.set(w - 1, 0, '╮', BORDER_COLOR, None);

    // --- Status row with side borders ---
    canvas.set(0, 1, '│', BORDER_COLOR, None);
    canvas.set(w - 1, 1, '│', BORDER_COLOR, None);
    status::draw(canvas, status_text, 1, STATUS_COLOR, 1.0);

    if bot <= 2 {
        // Fully collapsed: 3 rows — top border, status, bottom border
        canvas.set(0, 2, '╰', BORDER_COLOR, None);
        for x in 1..w - 1 {
            canvas.set(x, 2, '─', BORDER_COLOR, None);
        }
        canvas.set(w - 1, 2, '╯', BORDER_COLOR, None);

        // Clear below
        for y in 3..bh {
            for x in 0..w {
                canvas.set(x, y, ' ', (0, 0, 0), None);
            }
        }
        return;
    }

    // --- Side borders between status and bottom ---
    for y in 2..bot {
        canvas.set(0, y, '│', BORDER_COLOR, None);
        canvas.set(w - 1, y, '│', BORDER_COLOR, None);
    }

    // --- Bottom border (slides up) ---
    canvas.set(0, bot, '╰', BORDER_COLOR, None);
    for x in 1..w - 1 {
        canvas.set(x, bot, '─', BORDER_COLOR, None);
    }
    canvas.set(w - 1, bot, '╯', BORDER_COLOR, None);

    // Clear everything below the bottom border (wipe scene content)
    for y in (bot + 1)..bh {
        for x in 0..w {
            canvas.set(x, y, ' ', (0, 0, 0), None);
        }
    }
}

/// Query current cursor row using DSR (Device Status Report).
fn query_cursor_row(fd: i32) -> Option<u16> {
    use std::io::Read;

    let orig = unsafe {
        let mut t: libc::termios = std::mem::zeroed();
        if libc::tcgetattr(fd, &mut t) != 0 {
            return None;
        }
        let orig = t;
        t.c_lflag &= !(libc::ICANON | libc::ECHO);
        t.c_cc[libc::VMIN] = 0;
        t.c_cc[libc::VTIME] = 1;
        libc::tcsetattr(fd, libc::TCSANOW, &t);
        orig
    };

    let mut stdout = io::stdout();
    let _ = write!(stdout, "\x1b[6n");
    let _ = stdout.flush();

    let mut buf = [0u8; 32];
    let mut len = 0usize;
    let mut tty_read = unsafe { std::fs::File::from_raw_fd(fd) };

    for _ in 0..32 {
        let mut byte = [0u8; 1];
        match tty_read.read(&mut byte) {
            Ok(1) => {
                buf[len] = byte[0];
                len += 1;
                if byte[0] == b'R' {
                    break;
                }
            }
            _ => break,
        }
    }

    std::mem::forget(tty_read);

    unsafe {
        libc::tcsetattr(fd, libc::TCSANOW, &orig);
    }

    let s = std::str::from_utf8(&buf[..len]).ok()?;
    let s = s.strip_prefix("\x1b[")?;
    let s = s.strip_suffix('R')?;
    let mut parts = s.split(';');
    let row: u16 = parts.next()?.parse().ok()?;
    Some(row)
}

pub fn terminal_size() -> (u16, u16) {
    unsafe {
        let mut ws: libc::winsize = std::mem::zeroed();
        if libc::ioctl(1, libc::TIOCGWINSZ, &mut ws) == 0 {
            (ws.ws_col, ws.ws_row)
        } else {
            (80, 24)
        }
    }
}

/// Render splash screen inline (no raw mode, no cursor control).
/// Used for terminals that don't support raw mode (VHS, dumb terminals).
/// Renders the full UI: border, status bar, background, footer.
pub fn run_inline(
    info: &SystemInfo,
    bg: Option<Vec<background::BgCell>>,
    w: u16,
    h: u16,
) {
    use std::io::{self, Write};

    let mut canvas = Canvas::new(w, h);

    // Draw background
    if let Some(ref cells) = bg {
        background::draw(&mut canvas, cells, w);
    }

    // Draw border (full progress)
    border::draw_border(&mut canvas, BORDER_COLOR, 1.0);

    // Draw status bar
    let status_text = status::build_line(info);
    status::draw(&mut canvas, &status_text, 1, STATUS_COLOR, 1.0);

    // Draw footer
    let footer_text = "press any key to start";
    let footer_len = footer_text.chars().count() as u16;
    let footer_x = (w - footer_len) / 2;
    canvas.put_str(footer_x, h - 2, footer_text, FOOTER_COLOR, None);

    // Render canvas inline to stdout
    canvas.render_inline(&mut io::stdout());
}

use std::io::{self, Write};

const CYCLE_COLORS: &[u8] = &[
    2, // green
    6, // cyan
    4, // blue
    5, // magenta
    1, // red
    3, // yellow
];

pub fn run_foreground(logo: &str, total_height: u16, original_color: u8) {
    use std::os::unix::io::AsRawFd;

    let tty = match std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open("/dev/tty")
    {
        Ok(f) => f,
        Err(_) => return,
    };
    let fd = tty.as_raw_fd();

    // Query cursor row (DSR)
    let cursor_row = match query_cursor_row(fd) {
        Some(r) => r,
        None => return,
    };
    let logo_start_row = cursor_row.saturating_sub(total_height);
    if logo_start_row == 0 {
        return;
    }

    // Enter raw mode
    let orig = unsafe {
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
    };

    let logo_lines: Vec<&str> = logo.lines().collect();
    let logo_width = logo_lines.iter().map(|l| l.len()).max().unwrap_or(0);
    let mut stdout = io::stdout();
    let mut frame: usize = 0;

    // Hide cursor during animation
    let _ = write!(stdout, "\x1b[?25l");
    let _ = stdout.flush();
    let timeout = std::time::Duration::from_secs(60);
    let start = std::time::Instant::now();

    loop {
        if start.elapsed() > timeout {
            break;
        }

        // Any keypress stops the animation (key is consumed)
        let mut buf = [0u8; 64];
        let n = unsafe { libc::read(fd, buf.as_mut_ptr() as *mut libc::c_void, buf.len()) };
        if n > 0 {
            break;
        }

        let color = CYCLE_COLORS[frame % CYCLE_COLORS.len()];

        for (i, line) in logo_lines.iter().enumerate() {
            let row = logo_start_row + i as u16;
            // Pad line to logo_width so we overwrite old content but don't touch the info area
            let _ = write!(
                stdout,
                "\x1b[{};1H\x1b[3{}m{:<width$}\x1b[0m",
                row, color, line, width = logo_width
            );
        }
        let _ = stdout.flush();

        frame += 1;
        std::thread::sleep(std::time::Duration::from_millis(150));
    }

    // Restore original logo color
    for (i, line) in logo_lines.iter().enumerate() {
        let row = logo_start_row + i as u16;
        let _ = write!(
            stdout,
            "\x1b[{};1H\x1b[3{}m{:<width$}\x1b[0m",
            row, original_color, line, width = logo_width
        );
    }

    // Show cursor again
    let _ = write!(stdout, "\x1b[?25h");
    let _ = stdout.flush();

    // Restore terminal mode
    unsafe {
        libc::tcsetattr(fd, libc::TCSANOW, &orig);
    }

    // Move cursor below the output and ensure line ends with newline
    // so zsh doesn't show PROMPT_EOL_MARK (%)
    let _ = write!(stdout, "\x1b[{};1H\r\n", cursor_row);
    let _ = stdout.flush();

}

fn query_cursor_row(fd: libc::c_int) -> Option<u16> {
    // Temporarily set raw mode for the query
    let orig = unsafe {
        let mut t: libc::termios = std::mem::zeroed();
        if libc::tcgetattr(fd, &mut t) != 0 {
            return None;
        }
        let orig = t;
        t.c_lflag &= !(libc::ICANON | libc::ECHO);
        t.c_cc[libc::VMIN] = 0;
        t.c_cc[libc::VTIME] = 1; // 100ms timeout
        libc::tcsetattr(fd, libc::TCSANOW, &t);
        orig
    };

    // Send DSR: \033[6n
    let query = b"\x1b[6n";
    unsafe {
        libc::write(fd, query.as_ptr() as *const libc::c_void, query.len());
    }

    // Read response: \033[<row>;<col>R
    let mut buf = [0u8; 32];
    let mut pos = 0;
    loop {
        let n = unsafe { libc::read(fd, buf[pos..].as_mut_ptr() as *mut libc::c_void, 1) };
        if n <= 0 {
            break;
        }
        pos += 1;
        if buf[pos - 1] == b'R' || pos >= buf.len() {
            break;
        }
    }

    // Restore
    unsafe {
        libc::tcsetattr(fd, libc::TCSANOW, &orig);
    }

    let s = std::str::from_utf8(&buf[..pos]).ok()?;
    let inner = s.strip_prefix("\x1b[")?.strip_suffix('R')?;
    let row: u16 = inner.split(';').next()?.parse().ok()?;
    Some(row)
}

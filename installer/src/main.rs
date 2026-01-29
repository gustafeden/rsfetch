use blaeck::prelude::*;
use blaeck::Blaeck;
use flate2::read::GzDecoder;
use std::fs;
use std::io::{self, Read};
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::Command;
use std::thread;
use std::time::{Duration, Instant};
use tar::Archive;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const SPINNERS: [char; 4] = ['\u{25d0}', '\u{25d3}', '\u{25d1}', '\u{25d2}'];

fn target_triple() -> &'static str {
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    { "aarch64-apple-darwin" }
    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    { "x86_64-apple-darwin" }
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    { "x86_64-unknown-linux-gnu" }
    #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
    { "aarch64-unknown-linux-gnu" }
}

fn platform_label() -> &'static str {
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    { "macOS arm64" }
    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    { "macOS x86_64" }
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    { "Linux x86_64" }
    #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
    { "Linux aarch64" }
}

fn resolve_install_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    if let Ok(path) = std::env::var("PATH") {
        for dir in path.split(':') {
            if dir.starts_with(&home) {
                let p = PathBuf::from(dir);
                if p.is_dir() && is_writable(&p) {
                    return p;
                }
            }
        }
    }
    let fallback = PathBuf::from(&home).join(".local/bin");
    let _ = fs::create_dir_all(&fallback);
    fallback
}

fn is_writable(path: &PathBuf) -> bool {
    let test = path.join(".rsfetch_write_test");
    if fs::write(&test, b"").is_ok() {
        let _ = fs::remove_file(&test);
        true
    } else {
        false
    }
}

fn display_path(path: &PathBuf) -> String {
    let home = std::env::var("HOME").unwrap_or_default();
    let s = path.display().to_string();
    if !home.is_empty() && s.starts_with(&home) {
        format!("~{}", &s[home.len()..])
    } else {
        s
    }
}

#[derive(Clone)]
enum StepStatus {
    Pending,
    Active,
    Done(String),
}

struct InstallerState {
    platform: String,
    install_dir: PathBuf,
    steps: Vec<(String, StepStatus)>,
    error: Option<String>,
    finished: bool,
}

impl InstallerState {
    fn new() -> Self {
        let install_dir = resolve_install_dir();
        Self {
            platform: platform_label().to_string(),
            install_dir,
            steps: vec![
                ("Platform".to_string(), StepStatus::Pending),
                ("Install to".to_string(), StepStatus::Pending),
                ("Downloading".to_string(), StepStatus::Pending),
                ("Extracting".to_string(), StepStatus::Pending),
                ("Verifying".to_string(), StepStatus::Pending),
            ],
            error: None,
            finished: false,
        }
    }

    fn set_done(&mut self, idx: usize, detail: String) {
        self.steps[idx].1 = StepStatus::Done(detail);
    }

    fn set_active(&mut self, idx: usize) {
        self.steps[idx].1 = StepStatus::Active;
    }
}

fn build_ui(state: &InstallerState, spinner_frame: usize) -> Element {
    let spinner_char = SPINNERS[spinner_frame % SPINNERS.len()];
    let mut rows: Vec<Element> = Vec::new();

    // Header
    rows.push(element! {
        Text(content: format!("  rsfetch installer v{}", VERSION), bold: true, color: Color::White)
    });
    rows.push(element! {
        Text(content: "  \u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}", dim: true)
    });
    rows.push(element! { Text(content: "") });

    // Steps
    for (name, status) in &state.steps {
        let (icon, detail, icon_color) = match status {
            StepStatus::Pending => {
                ("\u{25cb}".to_string(), String::new(), Color::DarkGray)
            }
            StepStatus::Active => {
                (spinner_char.to_string(), String::new(), Color::Cyan)
            }
            StepStatus::Done(d) => {
                ("\u{2713}".to_string(), d.clone(), Color::Green)
            }
        };

        let line = if detail.is_empty() {
            format!("  {} {}", icon, name)
        } else {
            format!("  {} {}   {}", icon, name, detail)
        };

        rows.push(element! {
            Text(content: line, color: icon_color)
        });
    }

    // Error
    if let Some(err) = &state.error {
        rows.push(element! { Text(content: "") });
        rows.push(element! {
            Text(content: format!("  Error: {}", err), color: Color::Red)
        });
    }

    // Success box
    if state.finished && state.error.is_none() {
        rows.push(element! { Text(content: "") });
        rows.push(element! {
            Text(content: "  \u{256d}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{256e}", color: Color::Green)
        });
        rows.push(element! {
            Text(content: "  \u{2502}  \u{2713} rsfetch installed!            \u{2502}", color: Color::Green)
        });
        rows.push(element! {
            Text(content: "  \u{2502}    Run: rsfetch                  \u{2502}", color: Color::Green)
        });
        rows.push(element! {
            Text(content: "  \u{2570}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{256f}", color: Color::Green)
        });

        // PATH warning if needed
        let install_str = state.install_dir.display().to_string();
        if let Ok(path) = std::env::var("PATH") {
            if !path.split(':').any(|p| p == install_str) {
                rows.push(element! { Text(content: "") });
                rows.push(element! {
                    Text(content: format!("  note: add {} to your PATH", display_path(&state.install_dir)), dim: true)
                });
            }
        }
    }

    Element::node::<Box>(
        BoxProps {
            flex_direction: FlexDirection::Column,
            ..Default::default()
        },
        rows,
    )
}

fn render_spinner(blaeck: &mut Blaeck<io::Stdout>, state: &InstallerState, start: Instant) -> io::Result<()> {
    let frame = (start.elapsed().as_millis() / 80) as usize;
    blaeck.render(build_ui(state, frame))?;
    Ok(())
}

fn download(url: &str) -> Result<Vec<u8>, String> {
    let agent = ureq::AgentBuilder::new()
        .redirects(10)
        .timeout(Duration::from_secs(60))
        .build();
    let resp = agent.get(url).call().map_err(|e| format!("download failed: {}", e))?;
    let mut bytes = Vec::new();
    resp.into_reader()
        .read_to_end(&mut bytes)
        .map_err(|e| format!("read failed: {}", e))?;
    Ok(bytes)
}

fn extract_binary(tarball: &[u8], install_dir: &PathBuf) -> Result<PathBuf, String> {
    let decoder = GzDecoder::new(tarball);
    let mut archive = Archive::new(decoder);
    let entries = archive.entries().map_err(|e| format!("tar error: {}", e))?;

    for entry in entries {
        let mut entry = entry.map_err(|e| format!("tar entry error: {}", e))?;
        let path = entry.path().map_err(|e| format!("path error: {}", e))?;
        let name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        if name == "rsfetch" {
            let dest = install_dir.join("rsfetch");
            let mut file = fs::File::create(&dest).map_err(|e| format!("write error: {}", e))?;
            io::copy(&mut entry, &mut file).map_err(|e| format!("copy error: {}", e))?;
            fs::set_permissions(&dest, fs::Permissions::from_mode(0o755))
                .map_err(|e| format!("chmod error: {}", e))?;
            return Ok(dest);
        }
    }
    Err("rsfetch binary not found in archive".to_string())
}

fn verify(binary_path: &PathBuf) -> Result<String, String> {
    let output = Command::new(binary_path)
        .arg("--version")
        .output()
        .map_err(|e| format!("verify failed: {}", e))?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Err("rsfetch --version returned non-zero".to_string())
    }
}

fn main() -> io::Result<()> {
    let mut blaeck = Blaeck::new(io::stdout())?;
    blaeck.set_max_fps(20);
    let mut state = InstallerState::new();
    let spinner_start = Instant::now();

    // Step 0: Platform
    state.set_active(0);
    render_spinner(&mut blaeck, &state, spinner_start)?;
    state.set_done(0, state.platform.clone());

    // Step 1: Install dir
    state.set_active(1);
    render_spinner(&mut blaeck, &state, spinner_start)?;
    state.set_done(1, display_path(&state.install_dir));

    // Step 2: Download
    state.set_active(2);
    let target = target_triple();
    let url = format!(
        "https://github.com/gustafeden/rsfetch/releases/download/v{}/rsfetch-{}.tar.gz",
        VERSION, target
    );

    // Spin while downloading (in a thread)
    let url_clone = url.clone();
    let download_handle = thread::spawn(move || download(&url_clone));

    while !download_handle.is_finished() {
        render_spinner(&mut blaeck, &state, spinner_start)?;
        thread::sleep(Duration::from_millis(50));
    }

    let tarball = match download_handle.join().unwrap() {
        Ok(bytes) => {
            let size_mb = bytes.len() as f64 / 1_048_576.0;
            state.set_done(
                2,
                format!("rsfetch v{} ({:.1} MB)", VERSION, size_mb),
            );
            bytes
        }
        Err(e) => {
            state.steps[2].1 = StepStatus::Done(String::new());
            state.error = Some(e);
            blaeck.render(build_ui(&state, 0))?;
            blaeck.unmount()?;
            std::process::exit(1);
        }
    };

    // Step 3: Extract
    state.set_active(3);
    render_spinner(&mut blaeck, &state, spinner_start)?;
    let install_dir = state.install_dir.clone();
    let binary_path = match extract_binary(&tarball, &install_dir) {
        Ok(p) => {
            state.set_done(3, display_path(&p));
            p
        }
        Err(e) => {
            state.error = Some(e);
            blaeck.render(build_ui(&state, 0))?;
            blaeck.unmount()?;
            std::process::exit(1);
        }
    };

    // Step 4: Verify
    state.set_active(4);
    render_spinner(&mut blaeck, &state, spinner_start)?;
    match verify(&binary_path) {
        Ok(version_str) => {
            state.set_done(4, version_str);
        }
        Err(e) => {
            state.error = Some(e);
            blaeck.render(build_ui(&state, 0))?;
            blaeck.unmount()?;
            std::process::exit(1);
        }
    }

    state.finished = true;
    blaeck.render(build_ui(&state, 0))?;
    blaeck.unmount()?;

    Ok(())
}

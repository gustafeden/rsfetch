use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

pub fn clear() {
    let path = cache_path();
    let _ = std::fs::remove_file(&path);
}

/// Simple key-value cache persisted to disk.
/// Invalidated when system boot time changes.
pub struct Cache {
    path: PathBuf,
    boot_time: u64,
    entries: HashMap<String, String>,
    dirty: bool,
}

impl Cache {
    pub fn load() -> Self {
        let path = cache_path();
        let boot_time = get_boot_time();
        let entries = read_cache(&path, boot_time);
        Self {
            path,
            boot_time,
            entries,
            dirty: false,
        }
    }

    /// Get a cached value, or compute and cache it.
    pub fn get_or_insert(&mut self, key: &str, f: impl FnOnce() -> String) -> String {
        if let Some(val) = self.entries.get(key) {
            return val.clone();
        }
        let val = f();
        self.entries.insert(key.to_string(), val.clone());
        self.dirty = true;
        val
    }

    /// Write cache to disk if anything changed.
    pub fn save(&self) {
        if !self.dirty {
            return;
        }
        if let Some(parent) = self.path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let mut content = format!("boot_time={}\n", self.boot_time);
        for (k, v) in &self.entries {
            // Simple line-based format, escape newlines
            content.push_str(&format!("{}={}\n", k, v.replace('\n', "\\n")));
        }
        let _ = fs::write(&self.path, content);
    }
}

fn cache_path() -> PathBuf {
    if let Ok(home) = std::env::var("HOME") {
        PathBuf::from(home)
            .join(".cache")
            .join("blaeckfetch")
            .join("cache")
    } else {
        PathBuf::from("/tmp/blaeckfetch-cache")
    }
}

fn get_boot_time() -> u64 {
    #[cfg(target_os = "macos")]
    {
        use std::mem::MaybeUninit;
        use std::os::raw::c_char;

        #[repr(C)]
        struct Timeval {
            tv_sec: i64,
            tv_usec: i32,
        }

        extern "C" {
            fn sysctlbyname(
                name: *const c_char,
                oldp: *mut u8,
                oldlenp: *mut usize,
                newp: *const u8,
                newlen: usize,
            ) -> i32;
        }

        let name = b"kern.boottime\0";
        let mut tv = MaybeUninit::<Timeval>::uninit();
        let mut len = std::mem::size_of::<Timeval>();

        unsafe {
            if sysctlbyname(
                name.as_ptr() as *const c_char,
                tv.as_mut_ptr() as *mut u8,
                &mut len,
                std::ptr::null(),
                0,
            ) == 0
            {
                let tv = tv.assume_init();
                return tv.tv_sec as u64;
            }
        }
        0
    }
    #[cfg(target_os = "linux")]
    {
        // On Linux, use /proc/stat btime
        if let Ok(content) = fs::read_to_string("/proc/stat") {
            for line in content.lines() {
                if let Some(rest) = line.strip_prefix("btime ") {
                    if let Ok(t) = rest.trim().parse::<u64>() {
                        return t;
                    }
                }
            }
        }
        0
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        0
    }
}

fn read_cache(path: &PathBuf, current_boot_time: u64) -> HashMap<String, String> {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return HashMap::new(),
    };

    let mut map = HashMap::new();
    let mut cached_boot_time: Option<u64> = None;

    for line in content.lines() {
        if let Some((key, val)) = line.split_once('=') {
            if key == "boot_time" {
                cached_boot_time = val.parse().ok();
            } else {
                map.insert(key.to_string(), val.replace("\\n", "\n"));
            }
        }
    }

    // Invalidate if boot time changed
    match cached_boot_time {
        Some(bt) if bt == current_boot_time => map,
        _ => HashMap::new(),
    }
}

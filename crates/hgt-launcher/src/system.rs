//! What this PC is: processor, graphics cards, memory, Windows, free disk space. Read from the
//! registry and Windows' own counters only: nothing is run on the graphics card and nothing
//! goes over the network.

use std::path::Path;
use windows_sys::Win32::System::Registry::{
    HKEY, HKEY_LOCAL_MACHINE, RRF_RT_ANY, RegGetValueW, REG_BINARY, REG_DWORD, REG_QWORD, REG_SZ,
};

#[derive(Clone, Debug, Default)]
pub struct Gpu {
    pub name: String,
    /// Dedicated video memory, if the driver says.
    pub vram: Option<u64>,
}

impl Gpu {
    pub fn vendor(&self) -> &'static str {
        let n = self.name.to_ascii_lowercase();
        if n.contains("nvidia") || n.contains("geforce") || n.contains("rtx") {
            "NVIDIA"
        } else if n.contains("amd") || n.contains("radeon") {
            "AMD"
        } else if n.contains("intel") || n.contains("arc") {
            "Intel"
        } else {
            "other"
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct System {
    pub cpu: String,
    pub threads: usize,
    pub gpus: Vec<Gpu>,
    pub ram_total: u64,
    pub windows: String,
    pub build: u32,
}

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain([0]).collect()
}

enum RegValue {
    Text(String),
    Number(u64),
}

/// A value under `HKEY_LOCAL_MACHINE`.
fn reg(key: &str, name: &str) -> Option<RegValue> {
    reg_in(HKEY_LOCAL_MACHINE, key, name)
}

fn reg_in(root: HKEY, key: &str, name: &str) -> Option<RegValue> {
    let (k, n) = (wide(key), wide(name));
    let mut kind = 0u32;
    let mut size = 0u32;
    // SAFETY: a size query (null buffer), with valid NUL-terminated names.
    let first = unsafe { RegGetValueW(root, k.as_ptr(), n.as_ptr(), RRF_RT_ANY, &mut kind, std::ptr::null_mut(), &mut size) };
    if first != 0 || size == 0 {
        return None;
    }
    let mut buf = vec![0u8; size as usize + 2];
    let mut len = buf.len() as u32;
    // SAFETY: `buf` holds `len` bytes.
    let read = unsafe { RegGetValueW(root, k.as_ptr(), n.as_ptr(), RRF_RT_ANY, &mut kind, buf.as_mut_ptr().cast(), &mut len) };
    if read != 0 {
        return None;
    }
    let bytes = &buf[..len as usize];
    match kind {
        REG_SZ => {
            let units: Vec<u16> = bytes.as_chunks::<2>().0.iter().map(|c| u16::from_le_bytes(*c)).take_while(|&u| u != 0).collect();
            Some(RegValue::Text(String::from_utf16_lossy(&units).trim().to_string()))
        }
        REG_DWORD if bytes.len() >= 4 => Some(RegValue::Number(u64::from(u32::from_le_bytes(bytes[..4].try_into().ok()?)))),
        REG_QWORD if bytes.len() >= 8 => Some(RegValue::Number(u64::from_le_bytes(bytes[..8].try_into().ok()?))),
        REG_BINARY if bytes.len() >= 8 => Some(RegValue::Number(u64::from_le_bytes(bytes[..8].try_into().ok()?))),
        REG_BINARY if bytes.len() >= 4 => Some(RegValue::Number(u64::from(u32::from_le_bytes(bytes[..4].try_into().ok()?)))),
        _ => None,
    }
}

fn text(v: Option<RegValue>) -> Option<String> {
    match v {
        Some(RegValue::Text(s)) if !s.is_empty() => Some(s),
        _ => None,
    }
}

fn number(v: Option<RegValue>) -> Option<u64> {
    match v {
        Some(RegValue::Number(n)) => Some(n),
        _ => None,
    }
}

/// Display adapters as Windows lists them (the display device class), without the
/// software renderer.
fn gpus() -> Vec<Gpu> {
    const CLASS: &str = r"SYSTEM\CurrentControlSet\Control\Class\{4d36e968-e325-11ce-bfc1-08002be10318}";
    let mut out: Vec<Gpu> = Vec::new();
    for i in 0..16 {
        let key = format!(r"{CLASS}\{i:04}");
        let Some(name) = text(reg(&key, "DriverDesc")) else { continue };
        if name.to_ascii_lowercase().contains("basic render") || name.to_ascii_lowercase().contains("basic display") {
            continue;
        }
        let vram = number(reg(&key, "HardwareInformation.qwMemorySize")).or_else(|| number(reg(&key, "HardwareInformation.MemorySize")));
        if !out.iter().any(|g| g.name == name) {
            out.push(Gpu { name, vram });
        }
    }
    out
}

pub fn ram() -> (u64, u64) {
    use windows_sys::Win32::System::SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX};
    // SAFETY: a zeroed structure with its length set, as the call asks.
    unsafe {
        let mut m: MEMORYSTATUSEX = std::mem::zeroed();
        m.dwLength = std::mem::size_of::<MEMORYSTATUSEX>() as u32;
        if GlobalMemoryStatusEx(&mut m) != 0 { (m.ullTotalPhys, m.ullAvailPhys) } else { (0, 0) }
    }
}

/// Free bytes on the drive holding `path` (or its nearest existing parent).
pub fn free_space(path: &Path) -> Option<u64> {
    use windows_sys::Win32::Storage::FileSystem::GetDiskFreeSpaceExW;
    let existing = path.ancestors().find(|p| p.exists())?;
    let w = wide(&existing.display().to_string());
    let mut free = 0u64;
    // SAFETY: a valid path; the other outputs may be null.
    let ok = unsafe { GetDiskFreeSpaceExW(w.as_ptr(), &mut free, std::ptr::null_mut(), std::ptr::null_mut()) };
    (ok != 0).then_some(free)
}

/// `%SystemRoot%\System32`.
pub fn system32() -> std::path::PathBuf {
    std::path::PathBuf::from(std::env::var("SystemRoot").unwrap_or_else(|_| r"C:\Windows".into())).join("System32")
}

pub fn read() -> System {
    const NT: &str = r"SOFTWARE\Microsoft\Windows NT\CurrentVersion";
    let build = text(reg(NT, "CurrentBuildNumber")).and_then(|b| b.parse().ok()).unwrap_or(0);
    let release = text(reg(NT, "DisplayVersion")).unwrap_or_default();
    // (ProductName still says Windows 10 on Windows 11.)
    let family = if build >= 22000 { "Windows 11" } else { "Windows 10" };
    System {
        cpu: text(reg(r"HARDWARE\DESCRIPTION\System\CentralProcessor\0", "ProcessorNameString")).unwrap_or_else(|| "unknown processor".into()),
        threads: std::thread::available_parallelism().map_or(0, |n| n.get()),
        gpus: gpus(),
        ram_total: ram().0,
        windows: format!("{family} {release} (build {build})").replace("  ", " "),
        build,
    }
}

pub fn gb(bytes: u64) -> String {
    format!("{:.1} GB", bytes as f64 / (1u64 << 30) as f64)
}

/// Bytes and files under `dir` (a cache), counted without following links.
pub fn folder_size(dir: &Path) -> (u64, u64) {
    let (mut bytes, mut files) = (0u64, 0u64);
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&d) else { continue };
        for e in entries.flatten() {
            let Ok(kind) = e.file_type() else { continue };
            if kind.is_symlink() {
                continue;
            } else if kind.is_dir() {
                stack.push(e.path());
            } else if let Ok(m) = e.metadata() {
                bytes += m.len();
                files += 1;
            }
        }
    }
    (bytes, files)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn this_pc_is_read() {
        let s = read();
        assert!(s.threads > 0 && s.ram_total > 0 && s.build > 0, "{s:?}");
        assert!(!s.cpu.is_empty());
    }

    #[test]
    fn vendors_are_named() {
        let g = |n: &str| Gpu { name: n.into(), vram: None };
        assert_eq!(g("NVIDIA GeForce RTX 4070 Ti").vendor(), "NVIDIA");
        assert_eq!(g("AMD Radeon RX 7900 XTX").vendor(), "AMD");
        assert_eq!(g("Intel(R) Arc(TM) A770").vendor(), "Intel");
    }

    #[test]
    fn a_folder_is_counted() {
        let dir = std::env::temp_dir().join(format!("hgt-size-{}", std::process::id()));
        std::fs::create_dir_all(dir.join("a")).unwrap();
        std::fs::write(dir.join("a").join("x"), [0u8; 10]).unwrap();
        std::fs::write(dir.join("y"), [0u8; 5]).unwrap();
        assert_eq!(folder_size(&dir), (15, 2));
        let _ = std::fs::remove_dir_all(&dir);
    }
}

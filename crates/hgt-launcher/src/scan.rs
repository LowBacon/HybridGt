//! Finding the game, the runtime and the folders on this PC, so nothing has to be typed.
//!
//! Every fixed drive is walked breadth first, a few levels deep, past folders that never hold
//! either (Windows, caches, package stores). A runtime found beside one of HybridGt's
//! launchers (`play-fast.cmd`) brings that launcher's shader cache, saves and FidelityFX folders
//! with it, so a PC that already ran HybridGt keeps its saves and warm cache.

use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

#[derive(Default, Debug, Clone)]
pub struct Found {
    pub game_dir: Option<PathBuf>,
    pub runtime_exe: Option<PathBuf>,
    pub classic_exe: Option<PathBuf>,
    pub source_dir: Option<PathBuf>,
    pub cache_dir: Option<String>,
    pub data_dir: Option<String>,
    pub classic_cache_dir: Option<String>,
    pub classic_data_dir: Option<String>,
    pub ffx_dir: Option<String>,
    /// NVIDIA Streamline (DLSS, frame generation), where the runtime's own
    /// `Get-NVIDIA-DLSS.cmd` put it.
    pub streamline_dir: Option<String>,
    pub folders_seen: usize,
}

const SKIP: &[&str] = &[
    "windows",
    "$recycle.bin",
    "system volume information",
    "programdata",
    "appdata",
    "windowsapps",
    "node_modules",
    ".git",
    ".cargo",
    ".rustup",
    "deps",
    "incremental",
    "build",
    "shaders",
    "captures",
    "logs",
    "cache-next3",
    "msys64",
    "steamapps",
];

const DEPTH: usize = 5;
const MOST_FOLDERS: usize = 400_000;
const MOST_TIME: Duration = Duration::from_secs(45);

fn drives() -> Vec<PathBuf> {
    (b'A'..=b'Z').map(|d| PathBuf::from(format!("{}:\\", d as char))).filter(|p| p.is_dir()).collect()
}

/// A game folder: the executable beside its `MicrosoftGame.Config`.
pub fn is_game_dir(dir: &Path) -> bool {
    dir.join("game_scarlett_master.exe").is_file() && dir.join("MicrosoftGame.Config").is_file()
}

/// Walks every drive; `progress` gets the number of folders looked at now and then.
pub fn scan(progress: impl Fn(usize)) -> Found {
    let mut found = Found::default();
    let started = Instant::now();
    let mut queue: VecDeque<(PathBuf, usize)> = drives().into_iter().map(|d| (d, 0)).collect();
    // The launcher's own folder and its parents first: a runtime next to it wins.
    if let Ok(exe) = std::env::current_exe() {
        for dir in exe.ancestors().skip(1).take(4) {
            queue.push_front((dir.to_path_buf(), DEPTH.saturating_sub(3)));
        }
    }
    let mut runtimes: Vec<PathBuf> = Vec::new();
    while let Some((dir, depth)) = queue.pop_front() {
        found.folders_seen += 1;
        if found.folders_seen % 2000 == 0 {
            progress(found.folders_seen);
        }
        if found.folders_seen > MOST_FOLDERS || started.elapsed() > MOST_TIME {
            break;
        }
        if found.game_dir.is_none() && is_game_dir(&dir) {
            found.game_dir = Some(dir.clone());
        }
        let exe = dir.join("victory.exe");
        if exe.is_file() && dir.ends_with(Path::new("target").join("release")) && !runtimes.contains(&exe) {
            runtimes.push(exe);
        }
        if depth >= DEPTH {
            continue;
        }
        let Ok(entries) = std::fs::read_dir(&dir) else { continue };
        for e in entries.flatten() {
            let Ok(kind) = e.file_type() else { continue };
            if !kind.is_dir() || kind.is_symlink() {
                continue;
            }
            let name = e.file_name().to_string_lossy().to_lowercase();
            if SKIP.contains(&name.as_str()) || (name.starts_with('.') && name != ".") {
                continue;
            }
            queue.push_back((e.path(), depth + 1));
        }
    }
    // HybridGt's runtime since 0.2.0 is the one built in a `victory-nvidia` checkout (or any
    // other than the earlier `victory-gta6` one); that earlier one is Classic's.
    let named = |part: &str| runtimes.iter().find(|r| r.to_string_lossy().to_lowercase().contains(part)).cloned();
    found.classic_exe = named("victory-gta6");
    found.runtime_exe = named("victory-nvidia")
        .or_else(|| runtimes.iter().find(|r| !r.to_string_lossy().to_lowercase().contains("victory-gta6")).cloned());
    let checkout = |rt: &Path| rt.ancestors().nth(3).map(Path::to_path_buf);
    if let Some(c) = found.runtime_exe.as_deref().and_then(checkout) {
        if c.join("Cargo.toml").is_file() {
            found.source_dir = Some(c.clone());
        }
        if let Some(work) = c.parent() {
            let (mut cache, mut data, mut ffx) = (None, None, None);
            read_launcher(&work.join("play-nvidia.cmd"), &mut cache, &mut data, &mut ffx);
            (found.cache_dir, found.data_dir) = (cache, data);
            found.ffx_dir = found.ffx_dir.take().or(ffx);
        }
    }
    if let Some(c) = found.classic_exe.as_deref().and_then(checkout)
        && let Some(work) = c.parent()
    {
        let (mut cache, mut data, mut ffx) = (None, None, None);
        read_launcher(&work.join("play-fast.cmd"), &mut cache, &mut data, &mut ffx);
        (found.classic_cache_dir, found.classic_data_dir) = (cache, data);
        found.ffx_dir = found.ffx_dir.take().or(ffx);
    }
    // NVIDIA's and AMD's files, fetched by the runtime's own scripts (`Get-NVIDIA-DLSS.cmd`,
    // `Get-AMD-FSR.cmd`), lie where the walk does not go (under AppData) or below a checkout:
    // looked for where those scripts put them. Nothing is fetched here.
    let mut places: Vec<PathBuf> = std::env::var_os("LOCALAPPDATA").map(|l| PathBuf::from(l).join("victory")).into_iter().collect();
    places.extend([found.runtime_exe.as_deref(), found.classic_exe.as_deref()].into_iter().flatten().filter_map(checkout));
    let (streamline, ffx) = vendor_dirs(&places);
    found.streamline_dir = streamline;
    found.ffx_dir = found.ffx_dir.take().or(ffx);
    progress(found.folders_seen);
    found
}

/// The first folder below `places` holding NVIDIA Streamline (`sl.interposer.dll`) and the first
/// holding AMD's FidelityFX loader (`amd_fidelityfx_loader_dx12.dll`), as the runtime's
/// download scripts lay them out.
fn vendor_dirs(places: &[PathBuf]) -> (Option<String>, Option<String>) {
    let first = |sub: &[&str], file: &str| {
        places.iter().map(|p| sub.iter().fold(p.clone(), |d, s| d.join(s))).find(|d| d.join(file).is_file()).map(|d| d.display().to_string())
    };
    (
        first(&["nvidia-sdk", "Streamline", "bin", "x64"], "sl.interposer.dll"),
        first(&["amd-sdk", "FidelityFX", "bin"], "amd_fidelityfx_loader_dx12.dll"),
    )
}

/// The folders a HybridGt launcher (`set VICTORY_...=` lines) names, where they exist.
fn read_launcher(path: &Path, cache: &mut Option<String>, data: &mut Option<String>, ffx: &mut Option<String>) {
    let Ok(text) = std::fs::read_to_string(path) else { return };
    for line in text.lines() {
        let Some(rest) = line.trim().strip_prefix("set ") else { continue };
        let Some((key, value)) = rest.split_once('=') else { continue };
        let value = value.trim().to_string();
        if value.is_empty() || !Path::new(&value).exists() {
            continue;
        }
        match key.trim() {
            "VICTORY_CACHE_DIR" => *cache = Some(value),
            "VICTORY_DATA_DIR" => *data = Some(value),
            "VICTORY_FFX_DIR" => *ffx = Some(value),
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_launcher_gives_its_folders() {
        let dir = std::env::temp_dir().join(format!("hgt-scan-{}", std::process::id()));
        std::fs::create_dir_all(dir.join("cache")).unwrap();
        let cmd = dir.join("play-fast.cmd");
        std::fs::write(&cmd, format!("@echo off\r\nset VICTORY_CACHE_DIR={}\r\nset VICTORY_DATA_DIR=Z:\\nowhere\r\n", dir.join("cache").display())).unwrap();
        let (mut cache, mut data, mut ffx) = (None, None, None);
        read_launcher(&cmd, &mut cache, &mut data, &mut ffx);
        assert_eq!(cache.as_deref(), Some(dir.join("cache").to_string_lossy().as_ref()));
        assert_eq!(data, None, "a folder that does not exist is not taken");
        assert_eq!(ffx, None);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn vendor_files_are_found_where_the_scripts_put_them() {
        let root = std::env::temp_dir().join(format!("hgt-vendor-{}", std::process::id()));
        let sl = root.join("b").join("nvidia-sdk").join("Streamline").join("bin").join("x64");
        let fx = root.join("a").join("amd-sdk").join("FidelityFX").join("bin");
        std::fs::create_dir_all(&sl).unwrap();
        std::fs::create_dir_all(&fx).unwrap();
        std::fs::write(sl.join("sl.interposer.dll"), b"").unwrap();
        std::fs::write(fx.join("amd_fidelityfx_loader_dx12.dll"), b"").unwrap();
        let (s, f) = super::vendor_dirs(&[root.join("a"), root.join("b")]);
        let _ = std::fs::remove_dir_all(&root);
        assert_eq!(s, Some(sl.display().to_string()));
        assert_eq!(f, Some(fx.display().to_string()));
    }
}

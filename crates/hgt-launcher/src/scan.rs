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
    pub nvidia_exe: Option<PathBuf>,
    pub source_dir: Option<PathBuf>,
    pub cache_dir: Option<String>,
    pub data_dir: Option<String>,
    pub ffx_dir: Option<String>,
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
    // HybridGt's runtime is the one built in a `victory-gta6` checkout; VIctory-NVIDIA's in one
    // named for it. Otherwise the first found.
    let named = |part: &str| runtimes.iter().find(|r| r.to_string_lossy().to_lowercase().contains(part)).cloned();
    found.runtime_exe = named("victory-gta6").or_else(|| runtimes.iter().find(|r| !r.to_string_lossy().to_lowercase().contains("nvidia")).cloned());
    found.nvidia_exe = named("nvidia");
    if let Some(rt) = &found.runtime_exe {
        // ...\<checkout>\target\release\victory.exe
        let checkout = rt.ancestors().nth(3).map(Path::to_path_buf);
        if let Some(c) = &checkout
            && c.join("Cargo.toml").is_file()
        {
            found.source_dir = Some(c.clone());
        }
        if let Some(work) = checkout.as_deref().and_then(Path::parent) {
            read_launcher(&work.join("play-fast.cmd"), &mut found);
        }
    }
    progress(found.folders_seen);
    found
}

/// The folders a HybridGt launcher (`set VICTORY_...=` lines) names.
fn read_launcher(path: &Path, found: &mut Found) {
    let Ok(text) = std::fs::read_to_string(path) else { return };
    for line in text.lines() {
        let Some(rest) = line.trim().strip_prefix("set ") else { continue };
        let Some((key, value)) = rest.split_once('=') else { continue };
        let value = value.trim().to_string();
        if value.is_empty() || !Path::new(&value).exists() {
            continue;
        }
        match key.trim() {
            "VICTORY_CACHE_DIR" => found.cache_dir = Some(value),
            "VICTORY_DATA_DIR" => found.data_dir = Some(value),
            "VICTORY_FFX_DIR" => found.ffx_dir = Some(value),
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
        let mut found = Found::default();
        read_launcher(&cmd, &mut found);
        assert_eq!(found.cache_dir.as_deref(), Some(dir.join("cache").to_string_lossy().as_ref()));
        assert_eq!(found.data_dir, None, "a folder that does not exist is not taken");
        let _ = std::fs::remove_dir_all(&dir);
    }
}

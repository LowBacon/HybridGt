//! What the launcher remembers, in `hybridgt-launcher.json` beside the executable (or under
//! `%LOCALAPPDATA%\HybridGt` when that folder cannot be written). No path of any one PC is
//! built in: everything is found by the scan or chosen by the player.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::presets::{self, Preset};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct Config {
    pub game_dir: String,
    /// The runtime (VIctory-NVIDIA base with HybridGt's patch).
    pub runtime_exe: String,
    /// The earlier runtime (fxceless's victory-gta6 with HybridGt's patch), for "Classic".
    pub classic_exe: String,
    pub classic_cache_dir: String,
    pub classic_data_dir: String,
    /// 2 since the runtime moved to the VIctory-NVIDIA base (0.2.0); a config without it is
    /// older (0), not the default.
    #[serde(default)]
    pub version: u32,
    /// Read from launchers before 0.2.0 only.
    #[serde(skip_serializing)]
    pub nvidia_exe: String,
    pub source_dir: String,
    pub cache_dir: String,
    pub data_dir: String,
    pub logs_dir: String,
    /// AMD FidelityFX DLLs (FSR 3.1).
    pub ffx_dir: String,
    /// NVIDIA Streamline (DLSS); the runtime loads it from here if set.
    pub streamline_dir: String,
    /// DLSS through Streamline, when its files are in `streamline_dir`.
    pub dlss: bool,
    #[serde(deserialize_with = "presets::lenient")]
    pub preset: Preset,
    pub inplace: bool,
    pub hash_wide: bool,
    pub ambient: bool,
    pub audio_delay_ms: u32,
    /// 0 Auto (the runtime fills the card's budget with textures), 1 Balanced, 2 Low.
    pub memory: u8,
    /// Frames a second the runtime starts at most (0: the game's own pacing).
    pub fps_cap: u32,
    /// The runtime's log level: "warn", "info" or "debug".
    pub log_level: String,
    /// Interface animations (the header's drift, hover lifts).
    pub animations: bool,
    /// The shader count the last preparation reported, for a true percentage next time.
    pub shader_total: Option<u64>,
}

impl Default for Config {
    fn default() -> Self {
        let home = app_dir();
        Config {
            game_dir: String::new(),
            runtime_exe: String::new(),
            classic_exe: String::new(),
            classic_cache_dir: home.join("classic-cache").display().to_string(),
            classic_data_dir: home.join("classic-data").display().to_string(),
            version: 2,
            nvidia_exe: String::new(),
            source_dir: String::new(),
            cache_dir: home.join("cache").display().to_string(),
            data_dir: home.join("data").display().to_string(),
            logs_dir: home.join("logs").display().to_string(),
            ffx_dir: String::new(),
            streamline_dir: String::new(),
            dlss: false,
            preset: Preset::Fast,
            inplace: false,
            hash_wide: false,
            ambient: true,
            audio_delay_ms: 0,
            memory: 0,
            fps_cap: 0,
            log_level: "info".into(),
            animations: true,
            shader_total: None,
        }
    }
}

/// `%LOCALAPPDATA%\HybridGt`.
pub fn app_dir() -> PathBuf {
    std::env::var_os("LOCALAPPDATA").map(PathBuf::from).unwrap_or_else(std::env::temp_dir).join("HybridGt")
}

/// Beside the executable when that folder takes files, else in the app folder.
pub fn config_path() -> PathBuf {
    let beside = std::env::current_exe().ok().and_then(|p| p.parent().map(|d| d.join("hybridgt-launcher.json")));
    match beside {
        Some(p) if p.exists() || p.parent().is_some_and(writable) => p,
        _ => app_dir().join("hybridgt-launcher.json"),
    }
}

fn writable(dir: &std::path::Path) -> bool {
    let probe = dir.join(".hybridgt-write-test");
    let ok = std::fs::write(&probe, b"").is_ok();
    let _ = std::fs::remove_file(&probe);
    ok
}

impl Config {
    pub fn load() -> Self {
        Self::from_text(&std::fs::read_to_string(config_path()).unwrap_or_default())
    }

    /// Parses a config, moving one of a launcher before 0.2.0 to the two-runtime layout:
    /// its runtime and folders become Classic's (saves and warm caches kept), and the
    /// VIctory-NVIDIA build it named, if any, the main runtime.
    pub fn from_text(text: &str) -> Self {
        let mut c: Config = serde_json::from_str(text).unwrap_or_default();
        if c.version < 2 && !text.trim().is_empty() {
            let defaults = Config::default();
            c.classic_exe = std::mem::take(&mut c.runtime_exe);
            c.classic_cache_dir = std::mem::replace(&mut c.cache_dir, defaults.cache_dir);
            c.classic_data_dir = std::mem::replace(&mut c.data_dir, defaults.data_dir);
            c.runtime_exe = std::mem::take(&mut c.nvidia_exe);
            // Presets before 0.2.0 meant other switch sets (one was also called
            // "Performance"): an older config starts on Fast.
            c.preset = Preset::Fast;
        }
        c.version = 2;
        if !matches!(c.log_level.as_str(), "warn" | "info" | "debug") {
            c.log_level = "info".into();
        }
        c
    }

    pub fn save(&self) {
        let path = config_path();
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        if let Ok(t) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(path, t);
        }
    }

    /// The shader cache, saves folder and runtime a preset uses.
    pub fn folders_for(&self, preset: Preset) -> (&str, &str, &str) {
        if preset.classic() {
            (&self.classic_exe, &self.classic_cache_dir, &self.classic_data_dir)
        } else {
            (&self.runtime_exe, &self.cache_dir, &self.data_dir)
        }
    }

    /// Problems a run would meet, said plainly (empty: none).
    pub fn problems(&self) -> Vec<String> {
        let mut out = Vec::new();
        if !crate::scan::is_game_dir(std::path::Path::new(&self.game_dir)) {
            out.push("The game folder is not set, or has no game_scarlett_master.exe and MicrosoftGame.Config.".into());
        }
        let (exe, cache, data) = self.folders_for(self.preset);
        if !std::path::Path::new(exe).is_file() {
            out.push(format!("No runtime for the {} preset (set it under Settings).", self.preset.label()));
        }
        for (what, dir) in [("shader cache", cache), ("saves", data), ("logs", self.logs_dir.as_str())] {
            if dir.trim().is_empty() {
                out.push(format!("The {what} folder is not set."));
            }
        }
        if self.preset.fsr() && self.ffx_dir.trim().is_empty() {
            out.push("FSR 3.1 needs the folder of AMD's FidelityFX DLLs (Settings).".into());
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_older_launchers_config_moves_to_classic() {
        let old = r#"{"game_dir":"G","runtime_exe":"old.exe","nvidia_exe":"new.exe","cache_dir":"C","data_dir":"D","preset":"Performance"}"#;
        let c = Config::from_text(old);
        assert_eq!(c.preset, Preset::Fast, "an unknown preset reads as Fast");
        assert_eq!((c.classic_exe.as_str(), c.classic_data_dir.as_str(), c.runtime_exe.as_str()), ("old.exe", "D", "new.exe"));
        assert_eq!(c.version, 2);
    }

    #[test]
    fn a_current_config_stays_as_it_is() {
        let c = Config { runtime_exe: "r.exe".into(), data_dir: "D".into(), log_level: "loud".into(), ..Config::default() };
        let back = Config::from_text(&serde_json::to_string(&c).unwrap());
        assert_eq!((back.runtime_exe.as_str(), back.data_dir.as_str(), back.log_level.as_str()), ("r.exe", "D", "info"));
    }

    #[test]
    fn an_empty_or_broken_file_is_the_defaults() {
        assert_eq!(Config::from_text("").runtime_exe, "");
        assert_eq!(Config::from_text("{ not json").preset, Preset::Fast);
    }

    #[test]
    fn problems_are_said_for_a_blank_config() {
        let c = Config { game_dir: String::new(), ..Config::default() };
        let p = c.problems();
        assert!(p.iter().any(|s| s.contains("game folder")) && p.iter().any(|s| s.contains("runtime")));
    }
}

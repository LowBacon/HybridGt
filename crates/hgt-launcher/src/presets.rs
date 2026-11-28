//! The launcher's presets. Since 0.2.0 HybridGt's runtime is VIctory-NVIDIA's tree
//! (pepezin-ai, on readmet3xt-lab's foxhound base) with HybridGt's additions; the switch sets
//! here are its `play-foxhound.ps1` defaults, measured on this project's test PC at 14.2 fps
//! median against 5.8 for the earlier runtime (docs/PERFORMANCE_OPTIMIZATION.md). "Classic"
//! keeps the earlier runtime (fxceless's victory-gta6 with HybridGt's patch).

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum Preset {
    Fast,
    Performance,
    AllInOne,
    SeriesSFsr,
    RayTracing,
    Classic,
}

/// The runtime's speed switches (`play-foxhound.ps1`), without the folders.
const FAST: &[(&str, &str)] = &[
    ("VICTORY_LOG", "info"),
    ("VICTORY_THIN", "1"),
    ("VICTORY_JOIN_PARTS", "8"),
    ("VICTORY_RELAY", "2"),
    ("VICTORY_RELAY_COPY", "1"),
    ("VICTORY_WIDE_BACK", "1"),
    ("VICTORY_CARD_REHASH_RUNS", "1"),
    ("VICTORY_JOIN_COPIES", "1"),
    ("VICTORY_CARD_STEPS", "2"),
    ("VICTORY_CARD_AWAKE", "1"),
    ("VICTORY_CARD_BUFFERS_MIN_KB", "64"),
    ("VICTORY_INDIRECT_WINDOW", "1"),
    ("VICTORY_INDIRECT_BATCH", "2"),
    ("VICTORY_UPLOAD_AHEAD", "1"),
    ("VICTORY_SHOW_AFTER", "1"),
    ("VICTORY_MAP_THREADS", "1"),
    ("VICTORY_WC_NO_BACK", "1"),
    ("VICTORY_WC_NO_BACK_MIN_KB", "1024"),
    ("VICTORY_SOFT_VIEWS", "copies"),
    ("VICTORY_TEXTURE_POOL", "1"),
    ("VICTORY_RIPE_BLOCKS", "0"),
    ("VICTORY_TEXTURE_NO_WAIT", "1"),
    ("VICTORY_TEXTURE_CATCHUP", "0"),
    ("VICTORY_TEXTURE_STAGE_MS", "16"),
    ("VICTORY_LEVELS_ON_DEMAND", "0"),
    ("VICTORY_PRELUDE_UPLOADS", "1"),
    ("VICTORY_SHOW_NEW_LATER", "1"),
    ("VICTORY_POOL_SMALL", "0"),
    ("VICTORY_TABLED_REREAD", "1"),
    ("VICTORY_QUEUE_HOST_TEXTURES", "0"),
    ("VICTORY_HEAP_SYNC_FIRST", "1"),
    ("VICTORY_SYNC_IN_PLACE", "1"),
    ("VICTORY_SPANS_QUICK", "1"),
    ("VICTORY_SYNC_FINE", "1"),
    ("VICTORY_HEAP_SYNC_AT_SUBMIT", "1"),
    ("VICTORY_AMBIENT", "0.8"),
    ("VICTORY_AMBIENT_AIR", "1.5"),
    ("VICTORY_AMBIENT_SPECULAR", "1"),
    ("VICTORY_EARLY_DEPTH", "1"),
    ("VICTORY_LEAVE_OUT", "02384a60fda6f2d0,731a39171b568298"),
    ("VICTORY_PIPELINE_BOOK", "1"),
];

/// The earlier runtime's `play-fast` switches (measured 5.8-6.9 fps median).
const CLASSIC: &[(&str, &str)] = &[
    ("VICTORY_LOG", "info"),
    ("VICTORY_DISPLAY_MODE", "1920x1080"),
    ("VICTORY_LEVELS_ON_DEMAND", "1"),
    ("VICTORY_WIDE_REFRESH", "1"),
    ("VICTORY_WIDE_MERGE", "1"),
    ("VICTORY_GPU_BUFFER_COPIES", "1"),
    ("VICTORY_GPU_FILLS", "1"),
    ("VICTORY_ASYNC_SUBMIT", "1"),
    ("VICTORY_QUEUE2", "1"),
    ("VICTORY_INDIRECT_WINDOW", "1"),
    ("VICTORY_TEXTURE_NO_WAIT", "1"),
    ("VICTORY_JOIN_PARTS", "1"),
    ("VICTORY_INDIRECT_GATHER", "1"),
    ("VICTORY_CARD_REHASH_RUNS", "1"),
    ("VICTORY_COPY_PLAIN", "1"),
    ("VICTORY_BACK_OWN_VIEW", "1"),
    ("VICTORY_NATIVE_PIPELINES", "1"),
    ("VICTORY_NATIVE_GRAPHICS", "0"),
    ("VICTORY_NATIVE_RECTS", "1"),
    ("VICTORY_TEXTURE_BACK", "1"),
    ("VICTORY_HEAP_WORKER", "1"),
    ("VICTORY_HEAP_LOCKS", "1"),
    ("VICTORY_MERGE_GAP", "64"),
    ("VICTORY_MERGE_RUNS", "4096"),
    ("VICTORY_STAGING_POOL", "0"),
    ("VICTORY_CARD_WINDOWS_MB", "512"),
    ("VICTORY_TEXTURE_ROOM_PERCENT", "96"),
    ("VICTORY_CARD_BUFFERS_MIN_KB", "64"),
    ("VICTORY_CARD_BUFFERS_MB", "1280"),
    ("VICTORY_CARD_HASH_LEN", "1"),
    ("VICTORY_BVH_BUILDER", "0"),
    ("VICTORY_AMBIENT", "1"),
    ("VICTORY_WAVE_WIDE", "last"),
    ("VICTORY_WAVE_PAIRS_ONLY", "5dc3b25986622949,6f225b641c5b0708"),
    ("VICTORY_WAVE_PAIRS_FP32", "non-ieee-rne-ftz"),
    ("VICTORY_BAKE_SOURCES", "full"),
    ("VICTORY_LOOKUP_CACHE", "1"),
    ("VICTORY_SETTLE_ROOM", "1"),
    ("VICTORY_PARALLEL_REFRESH", "1"),
];

impl Preset {
    pub const ALL: [Preset; 6] = [Preset::Fast, Preset::Performance, Preset::AllInOne, Preset::SeriesSFsr, Preset::RayTracing, Preset::Classic];

    pub fn label(self) -> &'static str {
        match self {
            Preset::Fast => "Fast",
            Preset::Performance => "Performance",
            Preset::AllInOne => "All in one",
            Preset::SeriesSFsr => "Series S + FSR 3.1",
            Preset::RayTracing => "Ray tracing",
            Preset::Classic => "Classic",
        }
    }

    pub fn blurb(self) -> &'static str {
        match self {
            Preset::Fast => "Series X at 1440p, every\nspeed switch. 14 fps measured.",
            Preset::Performance => "Fast, plus DLSS and frame\ngeneration, worker pools for\nthis CPU, High priority.",
            Preset::AllInOne => "FSR 3.1 + the game's ray\ntracing + reflections stand-in.\nExperimental, slow with rays.",
            Preset::SeriesSFsr => "720p inside, AMD FSR 3.1 to\n1080p. Any GPU (AMD's DLLs).",
            Preset::RayTracing => "The game's own traced light.\nExperimental: slow, can\nlook broken on this runtime.",
            Preset::Classic => "The earlier runtime (fxceless\nbase). 6 fps, its own saves.",
        }
    }

    /// Which runtime runs it: the new one, or the earlier one.
    pub fn classic(self) -> bool {
        self == Preset::Classic
    }

    /// The console profile the runtime is started with.
    pub fn console(self) -> Option<&'static str> {
        match self {
            Preset::SeriesSFsr | Preset::AllInOne => Some("series-s"),
            Preset::Classic => None,
            _ => Some("series-x"),
        }
    }

    pub fn fsr(self) -> bool {
        matches!(self, Preset::SeriesSFsr | Preset::AllInOne)
    }

    pub fn switches(self) -> Vec<(&'static str, &'static str)> {
        let mut out: Vec<(&str, &str)> = match self {
            Preset::Classic => return CLASSIC.to_vec(),
            _ => FAST.to_vec(),
        };
        if self.console() == Some("series-x") {
            out.push(("VICTORY_NATIVE_1440P", "1"));
        }
        if self == Preset::Performance {
            // VIctory-NVIDIA's play-gta6-framegen.ps1: Vice-Tay's placed order (with its worker
            // pools, measured there at +22-29 % in heavy scenes on an RTX 4070 SUPER).
            out.push(("VICTORY_PLACED_ORDER", "1"));
        }
        if matches!(self, Preset::RayTracing | Preset::AllInOne) {
            out.retain(|(k, _)| !k.starts_with("VICTORY_AMBIENT") || *k == "VICTORY_AMBIENT_SPECULAR");
            out.push(("VICTORY_BVH_BUILDER", "1"));
        }
        out
    }
}

/// Reads a preset by name; names of older launchers' presets read as `Fast`.
pub fn lenient<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Preset, D::Error> {
    let name = String::deserialize(d).unwrap_or_default();
    Ok(Preset::ALL.into_iter().find(|p| format!("{p:?}") == name).unwrap_or(Preset::Fast))
}

/// Every switch a run of `preset` gets: the preset's own, the settings, and the folders.
pub fn environment(c: &crate::config::Config, preset: Preset) -> Vec<(String, String)> {
    let mut env: Vec<(String, String)> = preset.switches().iter().map(|(k, v)| (k.to_string(), v.to_string())).collect();
    let set = |env: &mut Vec<(String, String)>, k: &str, v: String| {
        env.retain(|(key, _)| key != k);
        env.push((k.to_string(), v));
    };
    let (_, cache, data) = c.folders_for(preset);
    set(&mut env, "VICTORY_GAME_DIR", c.game_dir.clone());
    set(&mut env, "VICTORY_CACHE_DIR", cache.to_string());
    set(&mut env, "VICTORY_DATA_DIR", data.to_string());
    set(&mut env, "VICTORY_LOG", c.log_level.clone());
    if preset.classic() {
        // Device method 0x218's index of the game's own shader heads, made on this PC.
        let index = std::path::Path::new(cache).join("native-index.txt");
        if index.is_file() {
            set(&mut env, "VICTORY_NATIVE_INDEX", index.display().to_string());
        }
        if c.inplace {
            set(&mut env, "VICTORY_INPLACE", "1".into());
        }
        if c.hash_wide {
            set(&mut env, "VICTORY_CARD_HASH_WIDE", "1".into());
        }
    } else {
        // Less memory: textures copied to the card capped (more of them wait), smaller
        // loading buffers.
        let caps: &[(&str, &str)] = match c.memory {
            1 => &[("VICTORY_TEXTURE_BUDGET_MB", "6144"), ("VICTORY_DSTORAGE_STAGING_MB", "128")],
            2 => &[("VICTORY_TEXTURE_BUDGET_MB", "4096"), ("VICTORY_TEXTURE_ROOM_PERCENT", "70"), ("VICTORY_DSTORAGE_STAGING_MB", "64")],
            _ => &[],
        };
        for (k, v) in caps {
            set(&mut env, k, v.to_string());
        }
        if let Some(us) = 1_000_000u32.checked_div(c.fps_cap) {
            set(&mut env, "VICTORY_FRAME_INTERVAL_US", us.to_string());
        }
        if c.dlss && !preset.fsr() && std::path::Path::new(&c.streamline_dir).join("sl.interposer.dll").is_file() {
            set(&mut env, "VICTORY_DLSS", "1".into());
            set(&mut env, "VICTORY_DLSS_MODE", "quality".into());
            set(&mut env, "VICTORY_STREAMLINE_DIR", c.streamline_dir.clone());
        }
    }
    if preset == Preset::Performance {
        // Worker pools by logical CPUs, as play-gta6-framegen.ps1 sizes them.
        let cpus = std::thread::available_parallelism().map_or(8, |n| n.get());
        let pools: &[(&str, &str)] = if cpus >= 24 {
            &[("VICTORY_THIN_THREADS", "12"), ("VICTORY_UNDER_THREADS", "8"), ("VICTORY_LAND_THREADS", "8")]
        } else if cpus >= 16 {
            &[("VICTORY_THIN_THREADS", "12"), ("VICTORY_UNDER_THREADS", "8"), ("VICTORY_LAND_THREADS", "6")]
        } else {
            &[]
        };
        for (k, v) in pools {
            set(&mut env, k, v.to_string());
        }
        // DLSS Quality and frame generation 2x (what an RTX 40 card generates; the runtime
        // gives no more frames than the card and driver report) where NVIDIA's files are.
        if std::path::Path::new(&c.streamline_dir).join("sl.interposer.dll").is_file() {
            set(&mut env, "VICTORY_DLSS", "1".into());
            set(&mut env, "VICTORY_DLSS_MODE", "quality".into());
            set(&mut env, "VICTORY_STREAMLINE_DIR", c.streamline_dir.clone());
            set(&mut env, "VICTORY_FRAMEGEN", "1".into());
            set(&mut env, "VICTORY_FRAMEGEN_X", "2".into());
        }
        set(&mut env, crate::jobs::PRIORITY, "high".into());
    }
    if !c.ambient {
        env.retain(|(k, _)| k != "VICTORY_AMBIENT" && k != "VICTORY_AMBIENT_AIR");
    }
    if c.audio_delay_ms > 0 {
        set(&mut env, "VICTORY_AUDIO_DELAY_MS", c.audio_delay_ms.to_string());
    }
    if preset.fsr() && !c.ffx_dir.is_empty() {
        set(&mut env, "VICTORY_FFX_DIR", c.ffx_dir.clone());
    }
    env
}

/// Switches the runtime's F1 menu owns (`victory-gpu` `host::menu`): a start passes them as
/// `victory play` options, which what the menu saved overrides, never with `--set`, which
/// would override the menu at every start.
const MENU_OWNED: &[&str] = &[
    "VICTORY_DLSS",
    "VICTORY_DLSS_MODE",
    "VICTORY_STREAMLINE_DIR",
    "VICTORY_FRAMEGEN",
    "VICTORY_FRAMEGEN_X",
    "VICTORY_FRAME_INTERVAL_US",
    "VICTORY_NATIVE_1440P",
    "VICTORY_AMBIENT",
    "VICTORY_AMBIENT_AIR",
    "VICTORY_AMBIENT_SPECULAR",
    "VICTORY_EARLY_DEPTH",
];

/// Folders `victory play` keeps from the environment (its `KEPT`).
const KEPT: &[&str] = &["VICTORY_GAME_DIR", "VICTORY_CACHE_DIR", "VICTORY_DATA_DIR", "VICTORY_STREAMLINE_DIR", "VICTORY_FIDELITYFX_DIR", "VICTORY_LOG", "VICTORY_HOME"];

/// A start of the current runtime through `victory play`, as its own launcher starts it, so the
/// F1 menu saves its settings (in HybridGt's own home, `%LOCALAPPDATA%\HybridGt\home`) and
/// they apply at every start: the arguments after the program, and the environment. The
/// preset's and the settings' switches the menu does not own go as `--set`; play's own tuned
/// switches stand in for the Fast list. Classic is started with `run` ([`environment`]).
pub fn play_command(c: &crate::config::Config, preset: Preset, game_exe: &str) -> (Vec<String>, Vec<(String, String)>) {
    let all = environment(c, preset);
    let get = |k: &str| all.iter().find(|(key, _)| key == k).map(|(_, v)| v.clone());
    let mut args = vec!["play".to_string(), game_exe.to_string()];
    if preset.console() == Some("series-s") {
        args.push("--series-s".into());
    }
    let dlss_files = std::path::Path::new(&c.streamline_dir).join("sl.interposer.dll").is_file();
    args.push("--dlss".into());
    args.push(if dlss_files && !preset.fsr() { get("VICTORY_DLSS_MODE").unwrap_or_else(|| "quality".into()) } else { "off".into() });
    if let Some(x) = get("VICTORY_FRAMEGEN_X") {
        args.push("--framegen".into());
        args.push(x);
    }
    args.push("--fps".into());
    args.push(if c.fps_cap > 0 { c.fps_cap.to_string() } else { "1000".into() });
    let mut env: Vec<(String, String)> = Vec::new();
    for (k, v) in &all {
        if KEPT.contains(&k.as_str()) || k == crate::jobs::PRIORITY {
            env.push((k.clone(), v.clone()));
        } else if !MENU_OWNED.contains(&k.as_str()) && !FAST.iter().any(|(f, fv)| f == k && fv == v) {
            args.push("--set".into());
            args.push(format!("{k}={v}"));
        }
    }
    if dlss_files {
        env.push(("VICTORY_STREAMLINE_DIR".into(), c.streamline_dir.clone()));
    }
    if !c.ffx_dir.is_empty() {
        env.push(("VICTORY_FIDELITYFX_DIR".into(), c.ffx_dir.clone()));
    }
    env.push(("VICTORY_HOME".into(), crate::config::app_dir().join("home").display().to_string()));
    (args, env)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    fn get<'a>(env: &'a [(String, String)], k: &str) -> Option<&'a str> {
        env.iter().find(|(key, _)| key == k).map(|(_, v)| v.as_str())
    }

    #[test]
    fn a_play_start_keeps_folders_in_the_environment_and_the_menu_free() {
        let mut c = Config { game_dir: "G".into(), audio_delay_ms: 150, memory: 1, ..Config::default() };
        c.fps_cap = 30;
        let (args, env) = play_command(&c, Preset::Performance, "G/game_scarlett_master.exe");
        assert_eq!(&args[..2], ["play", "G/game_scarlett_master.exe"]);
        let at = |flag: &str| args.iter().position(|a| a == flag).map(|i| args[i + 1].as_str());
        assert_eq!(at("--fps"), Some("30"));
        assert_eq!(at("--dlss"), Some("off"), "no NVIDIA files in a default config");
        let sets: Vec<&str> = args.iter().enumerate().filter(|(_, a)| *a == "--set").map(|(i, _)| args[i + 1].as_str()).collect();
        assert!(sets.contains(&"VICTORY_AUDIO_DELAY_MS=150") && sets.contains(&"VICTORY_PLACED_ORDER=1"));
        assert!(sets.contains(&"VICTORY_TEXTURE_BUDGET_MB=6144"));
        assert!(sets.iter().all(|s| !s.starts_with("VICTORY_DLSS") && !s.starts_with("VICTORY_FRAME") && !s.starts_with("VICTORY_AMBIENT")), "{sets:?}");
        let e = |k: &str| env.iter().find(|(key, _)| key == k).map(|(_, v)| v.as_str());
        assert_eq!(e("VICTORY_GAME_DIR"), Some("G"));
        assert!(e("VICTORY_HOME").is_some_and(|h| h.ends_with("home")));
        assert_eq!(e(crate::jobs::PRIORITY), Some("high"));
        assert!(env.iter().all(|(k, _)| KEPT.contains(&k.as_str()) || k == crate::jobs::PRIORITY), "{env:?}");
    }

    #[test]
    fn the_performance_preset_is_tuned_and_high_priority() {
        let env = environment(&Config::default(), Preset::Performance);
        assert_eq!(get(&env, "VICTORY_PLACED_ORDER"), Some("1"));
        assert_eq!(get(&env, "VICTORY_NATIVE_1440P"), Some("1"));
        assert_eq!(get(&env, crate::jobs::PRIORITY), Some("high"));
        // No NVIDIA files in a default config: no DLSS asked for.
        assert_eq!(get(&env, "VICTORY_DLSS"), None);
        assert_eq!(get(&environment(&Config::default(), Preset::Fast), crate::jobs::PRIORITY), None);
    }

    #[test]
    fn every_preset_has_switches_and_a_label() {
        for p in Preset::ALL {
            assert!(!p.switches().is_empty() && p.label().len() > 2);
        }
    }

    #[test]
    fn the_folders_follow_the_runtime() {
        let c = Config { cache_dir: "NEW".into(), classic_cache_dir: "OLD".into(), ..Config::default() };
        assert_eq!(get(&environment(&c, Preset::Fast), "VICTORY_CACHE_DIR"), Some("NEW"));
        assert_eq!(get(&environment(&c, Preset::Classic), "VICTORY_CACHE_DIR"), Some("OLD"));
    }

    #[test]
    fn memory_and_frame_cap_settings_reach_the_runtime() {
        let c = Config { memory: 2, fps_cap: 30, ..Config::default() };
        let env = environment(&c, Preset::Fast);
        assert_eq!(get(&env, "VICTORY_TEXTURE_BUDGET_MB"), Some("4096"));
        assert_eq!(get(&env, "VICTORY_FRAME_INTERVAL_US"), Some("33333"));
        assert_eq!(get(&environment(&Config::default(), Preset::Fast), "VICTORY_TEXTURE_BUDGET_MB"), None);
        // Each switch once.
        let mut keys: Vec<_> = env.iter().map(|(k, _)| k.clone()).collect();
        let n = keys.len();
        keys.sort();
        keys.dedup();
        assert_eq!(keys.len(), n);
    }

    #[test]
    fn ray_tracing_turns_the_builder_on_and_the_stand_in_off() {
        let env = environment(&Config::default(), Preset::RayTracing);
        assert_eq!(get(&env, "VICTORY_BVH_BUILDER"), Some("1"));
        assert_eq!(get(&env, "VICTORY_AMBIENT"), None);
        assert_eq!(get(&env, "VICTORY_AMBIENT_SPECULAR"), Some("1"));
    }

    #[test]
    fn dlss_needs_its_files() {
        let c = Config { dlss: true, streamline_dir: std::env::temp_dir().display().to_string(), ..Config::default() };
        assert_eq!(get(&environment(&c, Preset::Fast), "VICTORY_DLSS"), None, "no sl.interposer.dll there");
    }
}

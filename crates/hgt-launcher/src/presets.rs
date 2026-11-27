//! The launcher's presets. Since 0.2.0 HybridGt's runtime is VIctory-NVIDIA's tree
//! (pepezin-ai, on readmet3xt-lab's foxhound base) with HybridGt's additions; the switch sets
//! here are its `play-foxhound.ps1` defaults, measured on this project's test PC at 14.2 fps
//! median against 5.8 for the earlier runtime (docs/PERFORMANCE_OPTIMIZATION.md). "Classic"
//! keeps the earlier runtime (fxceless's victory-gta6 with HybridGt's patch).

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum Preset {
    Fast,
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
    pub const ALL: [Preset; 5] = [Preset::Fast, Preset::AllInOne, Preset::SeriesSFsr, Preset::RayTracing, Preset::Classic];

    pub fn label(self) -> &'static str {
        match self {
            Preset::Fast => "Fast",
            Preset::AllInOne => "All in one",
            Preset::SeriesSFsr => "Series S + FSR 3.1",
            Preset::RayTracing => "Ray tracing",
            Preset::Classic => "Classic",
        }
    }

    pub fn blurb(self) -> &'static str {
        match self {
            Preset::Fast => "Series X at 1440p, every\nspeed switch. 14 fps measured.",
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

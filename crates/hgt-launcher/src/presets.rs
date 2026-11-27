//! The launcher's presets: the switch sets of HybridGt's measured launchers
//! (`tools/play-fast.example.cmd`) and of VIctory-NVIDIA's `play-foxhound.ps1`.

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum Preset {
    AllInOne,
    Performance,
    RayTracing,
    SeriesSFsr,
    Nvidia,
}

/// HybridGt's `play-fast` switches (measured, see docs/PERFORMANCE_OPTIMIZATION.md), without
/// the folders, which the launcher sets.
const FAST: &[(&str, &str)] = &[
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

/// The game's own ray tracing: the structure builder on (its default), no stand-in light.
const RAY_TRACING_OFF: &[&str] = &["VICTORY_BVH_BUILDER", "VICTORY_AMBIENT"];

/// VIctory-NVIDIA's `play-foxhound.ps1` defaults (Series X, native 1440p).
const NVIDIA: &[(&str, &str)] = &[
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
    ("VICTORY_NATIVE_1440P", "1"),
];

impl Preset {
    pub const ALL: [Preset; 5] = [Preset::AllInOne, Preset::Performance, Preset::RayTracing, Preset::SeriesSFsr, Preset::Nvidia];

    pub fn label(self) -> &'static str {
        match self {
            Preset::AllInOne => "All in one",
            Preset::Performance => "Performance",
            Preset::RayTracing => "Ray tracing",
            Preset::SeriesSFsr => "Series S + FSR 3.1",
            Preset::Nvidia => "VIctory-NVIDIA",
        }
    }

    pub fn blurb(self) -> &'static str {
        match self {
            Preset::AllInOne => "FSR 3.1 + ray tracing + water\nfix + in-place memory.\nSlow while the rays build.",
            Preset::Performance => "HybridGt's measured switches.\nStand-in light, fastest.",
            Preset::RayTracing => "The game's own traced light.\nBeautiful, very slow today.",
            Preset::SeriesSFsr => "720p inside, AMD FSR 3.1\nto 1080p (needs AMD's DLLs).",
            Preset::Nvidia => "pepezin-ai's runtime on the\nfoxhound base (its own build).",
        }
    }

    /// The console profile the runtime is started with, if not its default.
    pub fn console(self) -> Option<&'static str> {
        match self {
            Preset::SeriesSFsr | Preset::AllInOne => Some("series-s"),
            Preset::Nvidia => Some("series-x"),
            _ => None,
        }
    }

    /// Each runtime and profile keeps its own shader cache.
    pub fn cache_name(self) -> &'static str {
        match self {
            Preset::Nvidia => "nvidia",
            _ => "hybridgt",
        }
    }

    /// The saves folder: VIctory-NVIDIA keeps its own (it cannot read HybridGt's).
    pub fn data_name(self) -> &'static str {
        match self {
            Preset::Nvidia => "nvidia",
            _ => "hybridgt",
        }
    }

    pub fn switches(self) -> Vec<(&'static str, &'static str)> {
        match self {
            Preset::Performance | Preset::SeriesSFsr => FAST.to_vec(),
            Preset::RayTracing => FAST.iter().copied().filter(|(k, _)| !RAY_TRACING_OFF.contains(k)).collect(),
            Preset::Nvidia => NVIDIA.to_vec(),
            // Everything at once: the game's ray tracing, AMD FSR 3.1 through the Series S path,
            // the title's memory opened to the card in place (measured +1 fps median, one run).
            Preset::AllInOne => {
                let mut all: Vec<_> = FAST.iter().copied().filter(|(k, _)| !RAY_TRACING_OFF.contains(k)).collect();
                all.push(("VICTORY_INPLACE", "1"));
                all
            }
        }
    }
}

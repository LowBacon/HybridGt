//! What the runtime needs on this PC, checked without installing or downloading anything.
//!
//! Only real needs are listed (from the runtime's own build and imports): 64-bit Windows 10 or
//! 11, Direct3D 12 and DXGI, the Visual C++ runtime the runtime links against, a graphics card
//! with enough memory, the game folder, and room for caches. DLSS (NVIDIA Streamline) and FSR
//! (AMD FidelityFX) are optional and come from their makers. Python, the DirectX Shader
//! Compiler and the Windows SDK are not needed to play (the runtime signs its own shaders);
//! Rust and the Visual Studio C++ tools only to build the runtime from source.

use std::path::Path;

use crate::config::Config;
use crate::system::{self, System};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Need {
    Required,
    Recommended,
    Optional,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Ok,
    Missing,
    Low,
}

#[derive(Clone, Debug)]
pub struct Check {
    pub name: &'static str,
    pub need: Need,
    pub state: State,
    pub detail: String,
    /// What to do when it is not met (no download is made for the player).
    pub fix: &'static str,
}

impl Check {
    pub fn blocks(&self) -> bool {
        self.need == Need::Required && self.state != State::Ok
    }
}

const GB: u64 = 1 << 30;

pub fn check(config: &Config, sys: &System) -> Vec<Check> {
    let s32 = system::system32();
    let file = |n: &str| s32.join(n).is_file();
    let mut out = Vec::new();
    let mut add = |name, need, ok: bool, low: bool, detail: String, fix| {
        out.push(Check { name, need, state: if ok { State::Ok } else if low { State::Low } else { State::Missing }, detail, fix });
    };
    add(
        "Windows 10 or 11, 64-bit",
        Need::Required,
        cfg!(target_pointer_width = "64") && sys.build >= 19041,
        sys.build > 0,
        sys.windows.clone(),
        "Windows 10 2004 (build 19041) or newer is needed.",
    );
    add(
        "Direct3D 12 and DXGI",
        Need::Required,
        file("d3d12.dll") && file("dxgi.dll"),
        false,
        format!("{} in System32", if file("d3d12.dll") { "d3d12.dll found" } else { "d3d12.dll missing" }),
        "Update Windows and the graphics driver.",
    );
    add(
        "Visual C++ runtime 2015-2022",
        Need::Required,
        file("vcruntime140.dll") && file("msvcp140.dll"),
        false,
        if file("vcruntime140.dll") { "vcruntime140.dll found".into() } else { "vcruntime140.dll missing".into() },
        "Install Microsoft's Visual C++ Redistributable (x64), from an installer you have.",
    );
    let best = sys.gpus.iter().max_by_key(|g| g.vram.unwrap_or(0));
    let vram = best.and_then(|g| g.vram).unwrap_or(0);
    add(
        "Graphics card",
        Need::Required,
        best.is_some() && vram >= 8 * GB,
        best.is_some(),
        best.map_or("none found".into(), |g| format!("{} ({}, {})", g.name, g.vendor(), if vram > 0 { system::gb(vram) } else { "memory not said".into() })),
        "A Direct3D 12 card with 8 GB or more (tested: RTX 4070 Ti 12 GB).",
    );
    add(
        "Memory (RAM)",
        Need::Recommended,
        sys.ram_total >= 30 * GB,
        sys.ram_total >= 15 * GB,
        system::gb(sys.ram_total),
        "32 GB is recommended; 16 GB runs with the Low memory setting.",
    );
    add(
        "Game folder",
        Need::Required,
        crate::scan::is_game_dir(Path::new(&config.game_dir)),
        false,
        if config.game_dir.is_empty() { "not set".into() } else { config.game_dir.clone() },
        "Choose the folder of your own copy (with game_scarlett_master.exe).",
    );
    let (exe, cache, _) = config.folders_for(config.preset);
    add(
        "Runtime (victory.exe)",
        Need::Required,
        is_ours(Path::new(exe)),
        false,
        if exe.is_empty() {
            "not set".into()
        } else if Path::new(exe).is_file() && !is_ours(Path::new(exe)) {
            format!("{exe}: not HybridGt's runtime")
        } else {
            exe.to_string()
        },
        "HybridGt.exe unpacks its own, or build one under Settings: other builds of VIctory are not started.",
    );
    let free = system::free_space(Path::new(cache)).unwrap_or(0);
    add(
        "Room for caches",
        Need::Required,
        free >= 20 * GB,
        free >= 5 * GB,
        format!("{} free where the shader cache lives", system::gb(free)),
        "The shader and pipeline caches take several GB.",
    );
    let ffx = Path::new(&config.ffx_dir);
    add(
        "AMD FidelityFX (FSR 3.1)",
        Need::Optional,
        ffx.join("amd_fidelityfx_loader_dx12.dll").is_file() || ffx.join("amd_fidelityfx_dx12.dll").is_file(),
        false,
        if config.ffx_dir.is_empty() { "not set".into() } else { config.ffx_dir.clone() },
        "For the FSR presets: AMD's FidelityFX SDK DLLs, in a folder you set.",
    );
    let sl = Path::new(&config.streamline_dir);
    add(
        "NVIDIA Streamline (DLSS)",
        Need::Optional,
        sl.join("sl.interposer.dll").is_file(),
        false,
        if config.streamline_dir.is_empty() { "not set".into() } else { config.streamline_dir.clone() },
        "For DLSS: NVIDIA's Streamline files, in a folder you set.",
    );
    let cargo = which("cargo").is_some() || std::env::var_os("USERPROFILE").is_some_and(|h| Path::new(&h).join(".cargo").join("bin").join("cargo.exe").is_file());
    add(
        "Rust (to build from source)",
        Need::Optional,
        cargo,
        false,
        if cargo { "cargo found".into() } else { "not installed".into() },
        "Only to build the runtime yourself: Rust and the Visual Studio C++ tools.",
    );
    out
}

/// What HybridGt's runtimes carry and other builds of VIctory do not: its repository line (the
/// F1 menu's, since the VIctory-NVIDIA df2daafe merge) or its own switch, the sound delay
/// (every HybridGt runtime since 0.1, the Classic one too).
const MARKERS: [&[u8]; 2] = [b"GITHUB.COM/LOWBACON/HYBRIDGT", b"VICTORY_AUDIO_DELAY_MS"];

/// Whether `exe` is one of HybridGt's runtimes, read from the file (it is not run).
pub fn is_ours(exe: &Path) -> bool {
    let Ok(bytes) = std::fs::read(exe) else { return false };
    MARKERS.iter().any(|m| bytes.windows(m.len()).any(|w| w == *m))
}

pub fn which(name: &str) -> Option<std::path::PathBuf> {
    let path = std::env::var_os("PATH")?;
    std::env::split_paths(&path).map(|d| d.join(format!("{name}.exe"))).find(|p| p.is_file())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_hybridgts_runtime_counts_as_one() {
        let dir = std::env::temp_dir().join(format!("hgt-ours-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let (ours, other) = (dir.join("ours.exe"), dir.join("other.exe"));
        std::fs::write(&ours, b"MZ...VICTORY_AUDIO_DELAY_MS...").unwrap();
        std::fs::write(&other, b"MZ...VICTORY_LOG...").unwrap();
        let (a, b) = (is_ours(&ours), is_ours(&other));
        let _ = std::fs::remove_dir_all(&dir);
        assert!(a && !b);
        assert!(!is_ours(Path::new("no such file.exe")));
    }

    #[test]
    fn a_blank_config_is_blocked_by_the_game_folder_and_runtime() {
        let checks = check(&Config::default(), &system::read());
        let blocked: Vec<_> = checks.iter().filter(|c| c.blocks()).map(|c| c.name).collect();
        assert!(blocked.contains(&"Game folder") && blocked.contains(&"Runtime (victory.exe)"), "{blocked:?}");
        assert!(checks.iter().filter(|c| c.need == Need::Optional).all(|c| !c.blocks()));
    }

    #[test]
    fn this_pc_has_direct3d_12() {
        let checks = check(&Config::default(), &system::read());
        assert!(checks.iter().any(|c| c.name == "Direct3D 12 and DXGI" && c.state == State::Ok));
    }
}

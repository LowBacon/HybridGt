# HybridGt

**Running the Xbox Series X|S build of Grand Theft Auto VI on Windows PCs: patches, tools and
measurements.**

> Tested on one PC: Windows 11 · NVIDIA RTX 4070 Ti (12 GB) · Ryzen 7 5800X3D · 32 GB RAM.
> You need **your own copy** of the game's Xbox files. This repository holds no game files,
> assets or shaders.

HybridGt is **not a standalone runtime**. The game runs inside
[victory-gta6](https://github.com/fxceless/victory-gta6), a fork of
[VIctory](https://github.com/Marvisak/VIctory) (both GPL-3.0-or-later). HybridGt adds:

| Folder | What is in it |
|---|---|
| `patches/victory-gta6-next3/` | **Current patch**: everything HybridGt adds to victory-gta6 `f6a09d3b`, as one diff |
| `tools/` | Launchers (`play-fast.example.cmd`, …), log statistics, pipeline decoders |
| `crates/` | Our own D3D12 diagnostic harness (`hgt-gfxdiag`) and texture code (`hgt-texture`) |
| `docs/` | Every test run, measurement and root cause ([PERFORMANCE_OPTIMIZATION.md](docs/PERFORMANCE_OPTIMIZATION.md)) |

---

## Status at a glance (2026-11-26)

**Not yet playable at a good frame rate.** On a new game, the intro, menus and prologue render,
with whole characters, at **7.7 fps median** in the world (best 9.3) on an RTX 4070 Ti, or 6.7
with interiors lit by the ambient stand-in (the default launcher).
Menus run at 30 fps.

### Frame rate

```
World, median   ███████▋░░░░░░░░░░░░░░░░░░░░░░   7.7 / 30 fps   (26 %)
World, best     █████████▎░░░░░░░░░░░░░░░░░░░░   9.3 / 30 fps   (31 %)
Menus           ██████████████████████████████  30.0 / 30 fps  (100 %)
```

History of the world median: 3.0 → 3.7 → 4.8 → 5.8 → 6.7 → **7.7 fps**. Each step was
measured; see [PERFORMANCE_OPTIMIZATION.md](docs/PERFORMANCE_OPTIMIZATION.md). The frame is
CPU-bound (copying and preparing title memory for the GPU), not GPU-bound.

### Shaders and pipelines

```
Shader containers translated   ███████████████████▉  99.8 %   25,938 of 25,985
Refused in game (last run)     ███████████████████▉   1 shader (underwater light volume)
Ray-tracing builder programs   ████████████████████  100 %    52 of 52 translate exactly for NVIDIA
Validated by Microsoft's DXIL  ███████████████████▊  99.1 %
```

### Fixes, step by step

Every problem moves through five steps. A fix counts as done only once it has been **seen in
the game**.

`①` found · `②` cause known · `③` fix written · `④` seen in game · `⑤` done

| Problem | Progress | Where it stands |
|---|---|---|
| Characters' bodies missing | `●●●●●` ⑤ | Whole characters in the prologue. Fixed by the fxceless merge with the 0x1c8 pipelines off |
| Intro "triangles" (rectangle lists) | `●●●●●` ⑤ | `VICTORY_NATIVE_RECTS=1` |
| Late textures | `●●●●○` ④ | "No room for textures" 60 → 20 a run (`VICTORY_CARD_WINDOWS_MB=1024`, `TEXTURE_ROOM_PERCENT=96`). Still some late |
| AMD FSR 3.x upscaling | `●●●●○` ④ | Works through the game's Series S path (`--console series-s`), 720p → 1080p. No fps gain; one black face and one white frame seen |
| Series S mode | `●●●●○` ④ | Boots and plays the prologue |
| Boat sinks ("Cal fell off the boat") | `●●●○○` ③ | fxceless's `VICTORY_TEXTURE_BACK=1` makes boats float; not yet checked at that mission in our runs |
| Black interiors (traced indirect light) | `●●●●○` ④ | `VICTORY_AMBIENT=1` stand-in lights them (costs about 1 fps). On in `play-fast` |
| Ray tracing (real traced light) | `●●●○○` ③ | Builder translates whole (52/52) and runs, but our build crashes 11 s later; VIctory-7900XTX's build survives it but loses the GPU after 3.5 min. Porting their 17 builder commits next |
| 0x1c8 graphics pipelines (console path) | `●●●○○` ③ | Built, but they hide bodies (stencil not decoded). Off |
| Lag spikes | `●●○○○` ② | Worst frames 1.3-2.8 s, when textures and pipelines arrive |
| Underwater light volume shader | `●●○○○` ② | One shader: a lane mask read as data |
| Mouth animation | `●○○○○` ① | Mouths don't move when characters talk; cause not found |

The full list with causes: [docs/KNOWN_FAILURES.md](docs/KNOWN_FAILURES.md) and
[docs/MASTER_STATUS.md](docs/MASTER_STATUS.md).

---

## Build and run

### 1. What you need

| | |
|---|---|
| OS | Windows 11 x64 |
| GPU | D3D12 with mesh shaders (tested: RTX 4070 Ti; 12 GB recommended) |
| Tools | [Rust](https://rustup.rs) (stable, MSVC toolchain), Git, Windows SDK (for `dxil.dll`) |
| Game | **Your own** copy of the Xbox Series files, in a folder outside every repository |
| FSR (optional) | AMD FidelityFX SDK DLLs: `amd_fidelityfx_loader_dx12.dll`, `amd_fidelityfx_upscaler_dx12.dll` |

### 2. Get the runtime and apply the patch

```powershell
git clone https://github.com/LowBacon/HybridGt
git clone https://github.com/fxceless/victory-gta6
cd victory-gta6
git checkout f6a09d3b
git switch -c hybridgt
git apply ..\HybridGt\patches\victory-gta6-next3\hybridgt-on-fxceless-f6a09d3b.patch
```

### 3. Build

```powershell
cargo build --release -p victory
```

The program is `target\release\victory.exe`.

### 4. Translate the shaders once (about 20 minutes)

This reads the game files and writes only to the cache folder. On NVIDIA, `32` matches what the
game run uses:

```powershell
$env:VICTORY_CACHE_DIR = 'D:\gta-cache'
$env:VICTORY_HOST_MAX_WAVE = '32'
.\target\release\victory.exe translate --warm all 'D:\Games\GTA6-Xbox'
```

### 5. Make your launcher

1. Copy `HybridGt\tools\play-fast.example.cmd` to a folder of your own.
2. Edit the first lines to your folders:

   ```bat
   set VICTORY_GAME_DIR=D:\Games\GTA6-Xbox
   set VICTORY_CACHE_DIR=D:\gta-cache
   set VICTORY_DATA_DIR=D:\gta-saves
   set VICTORY_FFX_DIR=D:\ffx-sdk\bin
   ```
3. Edit the last line to the paths of your `victory.exe` and the game's exe.

### 6. Play

Double-click the launcher. The log goes to `logs\<launcher name>.log`.

| Key | Pad |
|---|---|
| Enter | A |
| Esc | Menu |
| W A S D | Left stick |

The first run after a build is slower and flickers until the GPU pipelines it saves are made.

### Launchers

| Launcher | Use |
|---|---|
| `play-fast.example.cmd` | **Default**: fastest measured settings |
| `play-seriess-fsr.example.cmd` | Series S mode with AMD FSR 3.x upscaling |
| `play-merged.example.cmd` | Older test set, kept for comparisons |

---

## HybridGt's own tools and tests

Needs Rust (stable MSVC), plus DXC on `PATH` or in `HYBRIDGT_DXC`. Without DXC the GPU checks
print `SKIPPED` and pass; a skip does not count as tested.

```powershell
cargo test --workspace                 # GPU checks on WARP (software): safe on any machine
$env:HYBRIDGT_ADAPTER = "hw"           # the same checks on the first hardware GPU
cargo run -p hgt-gfxdiag               # the harness, one line per check
$env:HYBRIDGT_D3D12_DEBUG = "1"        # D3D12 debug layer: errors fail a check
```

Hardware runs skip the checks that once hung a GPU, unless `HYBRIDGT_ALLOW_HANG_REPRO=1` is set.

`python -I tools/victory_logstats.py <log>` prints a run's median frame cost in menus and in the world.

---

## Where the work comes from

The runtime merges work from several victory-gta6 forks:
[fxceless](https://github.com/fxceless/victory-gta6) (base),
[ZTRX](https://github.com/Zettrox/ztrx), DexNom, VIctory-7900XTX, VInAMD and cool_fox. Each
piece was measured on this PC before it was kept, and some were rejected: cool_fox's relay
switches (2.1 fps) and the cool_fox base (2.7 fps here). The commits are listed in
`patches/victory-gta6-next3/COMMITS.txt`.

## Licence

GPL-3.0-or-later, like VIctory and victory-gta6. XCL (GPL-2.0-only) was studied as a reference
only; no XCL code is used. Third-party use: [THIRD_PARTY.md](THIRD_PARTY.md).

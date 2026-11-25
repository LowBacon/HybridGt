# HybridGt

Engineering work toward running the Xbox Series X|S build of Grand Theft Auto VI, from files the
user owns, natively on Windows x64. Tested on one PC: Windows 11, NVIDIA RTX 4070 Ti.

HybridGt is **not a standalone runtime**. The game runs inside
[victory-gta6](https://github.com/fxceless/victory-gta6) (a fork of
[VIctory](https://github.com/Marvisak/VIctory), both GPL-3.0-or-later). HybridGt holds:

- **patches** on top of that fork (`patches/victory-gta6/`), plus an older series on upstream
  VIctory (`patches/victory/`, superseded);
- **independent tools and tests** (`crates/`): a D3D12 diagnostic harness and texture code;
- **documentation** of measurements, failures and root causes (`docs/`).

No game files, extracted assets or shaders are in this repository.

## Status (2026-11-25)

**Not playable.** The first gameplay scene (the prison prologue) renders and the player can move,
at **3.5–3.8 fps** (about 270 ms per frame) on an RTX 4070 Ti. Shadowed areas are black.

### Works (seen on screen or measured)

| What | Evidence |
|---|---|
| Boot, intro, menus at 30 fps | run logs `fork3`–`fork8` |
| Prologue renders: characters (Lucia, hair), textures, sky, sunlit exteriors, HUD prompts | user screenshots, `docs/MASTER_STATUS.md` |
| Player movement and camera through the game's own input | user report |
| 5,000–6,500 draws per frame in the world | run log `fork8` |
| Shader translation: 14,160 of 14,259 vertex/pixel, 2,718 of 2,734 compute, 8,762 of 8,765 mesh programs | offline `translate` over the title's shaders |
| No GPU hang or device removal in the last runs | run logs `fork6`, `fork8` |

### Runtime base and patches (`patches/victory-gta6/`, on fork commit `a26859f`)

Since 2026-11-25 the patches apply to fxceless/victory-gta6 `a26859f`. That commit already
contains all of [ZTRX](https://github.com/Zettrox/ztrx) `main`. It also adds:
- depth bounds;
- textures picked by index re-checked after upload;
- byte-copied descriptor mirroring;
- faster defaults.

See [docs/ZTRX_IMPLEMENTATION_AUDIT.md](docs/ZTRX_IMPLEMENTATION_AUDIT.md).

| Patch | What it does | Source | Test |
|---|---|---|---|
| 0001 | Wave-wide branch form as a last resort for wave64 programs a 32-lane GPU would refuse | HybridGt | shader tests |
| 0002–0003 | FidelityFX (FSR): the title's FSR DLL loads; its calls run AMD's PC upscaler on the host list (`VICTORY_FFX_DIR`) | HybridGt | builds, HLE tests; game test pending |
| 0004 | Samplers built as constants at a join get reserved slots 2032–2047 of the host sampler heap | HybridGt | shader + GPU tests |
| 0005–0006 | No blending on integer render targets (a refused world mesh pipeline: missing walls); LDS-flag regions run as two host waves (`CS_Tiling_Generate_Opaque`) | ZTRX PR #36 | GPU tests, RTX 4070 Ti |
| 0007–0008 | Guarded textures re-checked every frame after a change (intro video jumping back and forth) | ZTRX PR #32 | GPU tests; game test pending |
| 0009–0010 | `ds_bpermute_b32` / `ds_permute_b32` across two host waves (shadow-mask trace programs) | ZTRX PR #43 | card tests, RTX 4070 Ti |
| 0011 | Test fix-up | HybridGt | — |

`cargo test --release -p victory-shader -p victory-gpu -p victory-hle` on the RTX 4070 Ti: **467 passed, 0 failed, 6 ignored** (2026-11-25).
None of these changes has been run in the game yet.

Also in `crates/hgt-gfxdiag`: GPU checks proving the RTX 4070 Ti runs 32-lane waves and that
two-half wave64 emulation gives the console's answer in compute and mesh shaders, including
ballots inside branches.

### In progress

- **Frame rate.** Measured per world frame: about 105 ms CPU preparation (about 130 MB of title
  memory copied to the GPU), about 94 ms GPU work, about 38 ms copying results back, all on one
  CPU thread. Next: a GPU timing run naming the slowest shaders, and the fork's copy-free
  `VICTORY_INPLACE` mode.
- **FSR (stage 2).** The title's upscaler uses the same five-function API as AMD's PC FidelityFX
  SDK (MIT). The plan is to run the PC upscaler on the host's command list when the title calls
  `ffxDispatch`. Not started beyond stage 1.
- **Series S mode** (`--console series-s`, lower resolution): it stopped at a black screen
  because the FSR DLL was refused; retest with patch 0007 pending.

### Not working

| Problem | Cause (as far as known) |
|---|---|
| Shadowed areas and interiors black | The title's indirect light and lamp/sun shadows are ray traced in its own compute shaders; the fork's ray-tracing path is off by default and not yet run end to end |
| Some sun shadows and mirror reflections missing (40 programs) | A sampler chosen between a loaded one and a constant one; needs reserved slots in the host sampler heap |
| Menu / startup animation flickers back and forth | Unknown; not the relay or texture switches (tested). Next: a RenderDoc capture |
| 2 compute programs still need 64-lane waves; 1 mesh pipeline refused at run time | — |
| 2 device methods (`0x1c8`, `0x218`) not implemented: 29 pipelines skipped | Unknown layout; likely ray-tracing state |
| 195 voxelization geometry shaders do not translate | No geometry-shader stage; not requested in the scenes tested |

Details: [docs/MASTER_STATUS.md](docs/MASTER_STATUS.md),
[docs/KNOWN_FAILURES.md](docs/KNOWN_FAILURES.md),
[docs/ROOT_CAUSE_ANALYSIS.md](docs/ROOT_CAUSE_ANALYSIS.md),
[docs/NVIDIA_WAVE64_COMPATIBILITY.md](docs/NVIDIA_WAVE64_COMPATIBILITY.md).

## Build and run the game

Requirements: Windows 11 x64, a D3D12 GPU with mesh shaders (tested: RTX 4070 Ti), Rust (stable,
MSVC toolchain), Git, the Windows SDK (for `dxil.dll`), and **your own** copy of the game's
Xbox Series files in a folder outside every repository.

```powershell
git clone https://github.com/fxceless/victory-gta6
cd victory-gta6
git checkout a26859f
git switch -c hybridgt
git am <path to HybridGt>\patches\victory-gta6\*.patch
cargo build --release -p victory
```

Set the paths (any folders outside the repository):

```powershell
$env:VICTORY_GAME_DIR  = '<your game folder>'
$env:VICTORY_CACHE_DIR = '<a cache folder>'
$env:VICTORY_DATA_DIR  = '<a save-data folder>'
```

Translate every shader into the cache once per build (about 20 minutes; it reads the game files
and writes only to the cache). On an NVIDIA GPU this setting matches what the game run picks:

```powershell
$env:VICTORY_HOST_MAX_WAVE = '32'
.\target\release\victory.exe translate --warm all $env:VICTORY_GAME_DIR
Remove-Item Env:VICTORY_HOST_MAX_WAVE
```

Run. On `a26859f` the fork's own defaults already include the speed settings HybridGt used
to set by hand: relay stage 2, wide copy-back, and card copies from 1 to 1,024 MB. Upstream
measured about 230 ms per frame with them, where it was about 400 ms, on an RTX 4070. HybridGt
has not measured them on its RTX 4070 Ti yet.

```powershell
$env:VICTORY_DISPLAY_MODE = '1920x1080'
$env:VICTORY_FFX_DIR = '<folder with amd_fidelityfx_loader_dx12.dll and amd_fidelityfx_upscaler_dx12.dll>'
.\target\release\victory.exe run "$env:VICTORY_GAME_DIR\<the game's exe>" --execute
```

Keyboard without a pad: Enter = A, Esc = Menu, WASD = left stick. The first run after a build
still flickers until the GPU pipelines it saves are made.

## Build and test HybridGt's own tools

Requirements: Rust (stable MSVC) and DXC on `PATH` or in `HYBRIDGT_DXC`. Without DXC the GPU
checks print `SKIPPED` and pass; skips are not coverage.

```powershell
cargo test --workspace                 # GPU checks on WARP (software); safe on any machine
$env:HYBRIDGT_ADAPTER = "hw"           # the same checks on the first hardware GPU
cargo run -p hgt-gfxdiag               # the harness, one line per check
$env:HYBRIDGT_D3D12_DEBUG = "1"        # D3D12 debug layer: errors fail a check
```

Hardware runs refuse the checks that hung a GPU before, unless `HYBRIDGT_ALLOW_HANG_REPRO=1` is set.

## Licence

GPL-3.0-or-later, as VIctory and victory-gta6. XCL (GPL-2.0-only) was studied as a reference
only; no XCL code is used. Third-party use is listed in [THIRD_PARTY.md](THIRD_PARTY.md).

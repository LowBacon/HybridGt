# Build and run

[← Back to the README](../../README.md)

## 1. What you need

| | |
|---|---|
| OS | Windows 11 x64 |
| GPU | Direct3D 12 with mesh shaders (tested: RTX 4070 Ti; 12 GB recommended) |
| Tools | [Rust](https://rustup.rs) (stable, MSVC toolchain), Git, Windows SDK (for `dxil.dll`) |
| Game | **Your own** copy of the Xbox Series files, in a folder outside every repository |
| AMD FSR (optional) | FidelityFX SDK DLLs: `amd_fidelityfx_loader_dx12.dll`, `amd_fidelityfx_upscaler_dx12.dll` |

## 2. Get the runtime and apply the patch

```powershell
git clone https://github.com/LowBacon/HybridGt
git clone https://github.com/fxceless/victory-gta6
cd victory-gta6
git checkout f6a09d3b
git switch -c hybridgt
git apply ..\HybridGt\patches\victory-gta6-next3\hybridgt-on-fxceless-f6a09d3b.patch
```

## 3. Build

```powershell
cargo build --release -p victory
```

The program is `target\release\victory.exe`.

## 4. Translate the shaders once (about 20 minutes)

This reads the game files and writes only to the cache folder. On NVIDIA, `32` matches what the
game run uses:

```powershell
$env:VICTORY_CACHE_DIR = 'D:\gta-cache'
$env:VICTORY_HOST_MAX_WAVE = '32'
.\target\release\victory.exe translate --warm all 'D:\Games\GTA6-Xbox'
```

## 5. Make your launcher

1. Copy `HybridGt\tools\play-fast.example.cmd` to a folder of your own.
2. Edit the first lines to your folders:

   ```bat
   set VICTORY_GAME_DIR=D:\Games\GTA6-Xbox
   set VICTORY_CACHE_DIR=D:\gta-cache
   set VICTORY_DATA_DIR=D:\gta-saves
   set VICTORY_FFX_DIR=D:\ffx-sdk\bin
   ```
3. Edit the last line to the paths of your `victory.exe` and the game's exe.

## 6. Play

Double-click the launcher. The log goes to `logs\<launcher name>.log`. The first run after a build
is slower and flickers until the GPU pipelines it saves are made.

| Launcher | Use |
|---|---|
| `play-fast.example.cmd` | **Default**: fastest measured settings, interiors lit |
| `play-seriess-fsr.example.cmd` | Series S mode with AMD FSR 3.1 upscaling (720p → 1080p) |
| `play-merged.example.cmd` | Older test set, kept for comparisons |

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

| Tool | What it does |
|---|---|
| `python -I tools/victory_logstats.py <log>` | A run's median frame cost in menus and in the world |
| `python -I tools/make_status_images.py` | Redraws the README's badges and progress chart |
| `python -I tools/decode_1c8.py <dump> <index>` | Decodes saved 0x1c8 pipeline descriptions |

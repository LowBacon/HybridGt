# HybridGt launcher (0.3.0)

One executable, `HybridGt.exe`, built from [`crates/hgt-launcher`](../crates/hgt-launcher). It finds
your own copy of the game and runs it on the HybridGt runtime (VIctory-NVIDIA with HybridGt's patch).
It carries no game data and changes nothing in the game's folder.

## Pages

| Page | What it shows and does |
|---|---|
| Home | Your game's banner art and name (read from your folder), version, status chips, PLAY/STOP, presets, last played, time played, **INITIALIZE** with every step's status |
| Library | The game card: logo, location, version, size, time played; installation, shader cache, pipeline cache, assets and compatibility status |
| Performance | Processor, graphics cards and their memory, RAM (total and free), Windows; the preset's resolution, upscaler, ray tracing, memory mode, frame cap; cache sizes |
| Settings | Game folder and scan, memory (Auto/Balanced/Low), frame cap, DLSS, estimated light, Classic switches, sound delay, every folder, build from source, log level, animations, the offline policy, reset, clear shader cache (asks twice), run setup again |
| Diagnostics | Requirements with how to fix each, problems in the settings, the last setup, the running session's frame rate, frame time and draws, the last log summarized, the log |

Frame rates appear only on Diagnostics. The game's name, the window title and the library never
carry a frame rate.

## Initialize

One button runs these steps in order. Each ends **done**, **not applicable** (with why),
**blocked** (with what is missing) or **failed** (with the error). The bar counts settled steps and
the real counts of the step that is running (shaders done of the total last counted); it reaches
100 % only when every step has settled, and the panel then says whether all were done.

| Step | What it does |
|---|---|
| Dependencies | Windows 10 2004+ (build 19041), Direct3D 12 and DXGI, the Visual C++ runtime, a graphics card with 8 GB or more, RAM, 20 GB free for caches. A missing requirement stops the steps after it, each marked blocked |
| Game files | The folder has `game_scarlett_master.exe` and `MicrosoftGame.Config`; reads its name, version and archives |
| Configuration | Creates the cache, saves and logs folders; checks the runtime for the preset |
| Shader cache | `victory shaders <game> --jobs N`, followed line by line; records the total for next time |
| Pipeline cache | `victory warm`, when a game session has recorded `known-pipelines.bin`; otherwise blocked, saying one session is needed; not applicable for Classic |
| Asset cache | Not applicable: the runtime streams assets from the game's own archives (DirectStorage), so there is nothing to unpack |
| Graphics compatibility | Names the card and whether it has been tested (NVIDIA: tested; AMD: expected, not tested) |
| Integrity check | Counts the cache, writes `logs/setup-report-<time>.txt` |

The mission brief listed 14 stages. They map onto these 8: required software, Visual C++ and
DirectX are *Dependencies*; game detection and validation are *Game files*; configuration and
folder creation are *Configuration*; the asset and texture cache stages are *Asset cache*;
renderer and upscaler compatibility are *Graphics compatibility*; the final validation and report
are *Integrity check*. Nothing is downloaded: a missing piece is reported with what to supply.

## Offline

- No network crate is a dependency, and no source file opens a socket, a URL or a browser. Two unit
  tests (`offline::tests`) fail the build otherwise.
- No update checks, telemetry, sign-in or cloud saves.
- The game reaches only loopback: the runtime's `victory-hle` `modules::net` refuses every other
  connection and name lookup, always.

## Identity

- Executable: `HybridGt.exe`, with an icon drawn from HybridGt's own mark (`src/mark.rs`, also the
  window icon) and Windows version details (product HybridGt 0.3.0, GPL-3.0-or-later), compiled by
  the Windows SDK's `rc.exe` in `build.rs`. Without the SDK the build still succeeds, without them.
- The game is shown as its own `MicrosoftGame.Config` names it ("Grand Theft Auto VI"), with the
  splash and logo images from **your** folder, read at run time. HybridGt ships no logo or art of
  the game and makes none up.
- The game's executable is not renamed or copied as `GTA6.exe`: the game folder is never modified,
  and the runtime finds the title by `game_scarlett_master.exe`.

## Building

```powershell
cargo build --release --offline -p hgt-launcher
```

```powershell
cargo test --release --offline -p hgt-launcher
```

`HGT_RUNTIME_EXE=<victory.exe>` at build time embeds the runtime; `HGT_RC=<rc.exe>` points to an
SDK outside the Windows Kits folder.

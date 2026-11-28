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
| Settings | Game folder and scan, memory (Auto/Balanced/Low), frame cap, DLSS, estimated light, Classic switches, sound delay, every folder, **Get NVIDIA DLSS**, **Get AMD FSR 3** and **Get MFG Unlocker (mod)** (DLSS and FSR 3 show whether their files are present), build from source, log level, animations, the offline policy, reset, clear shader cache (asks twice), run setup again |
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
are *Integrity check*. Initialize downloads nothing: a missing piece is reported with what to supply.

## Which runtime it starts

Only HybridGt's own runtimes. Before a start, a download or taking a scan result, the launcher
reads the `victory.exe` file (it does not run it) for one of HybridGt's markers: the F1 menu's line
`GITHUB.COM/LOWBACON/HYBRIDGT` (runtimes since the VIctory-NVIDIA df2daafe merge) or the
`VICTORY_AUDIO_DELAY_MS` switch (HybridGt's sound delay, in the current runtime and the Classic one). Other builds of
VIctory are passed by in the scan, and PLAY refuses them with a message (`deps::is_ours`).

## How a game is started

| Preset | Started with | Settings go |
|---|---|---|
| Every preset but Classic | `victory play <game>` | The F1 menu saves to `victory-settings.txt` in HybridGt's home, `%LOCALAPPDATA%\HybridGt\home` (`VICTORY_HOME`), and they apply at the next start |
| Classic | `victory run <game> --execute` | The launcher's own settings, as before |

- The settings the F1 menu owns (DLSS and its mode, frame generation, frame cap, the Series S
  profile) are passed as play options (`--dlss`, `--framegen`, `--fps`, `--series-s`), which what
  the menu saved overrides. They are never passed with `--set`, which would override the menu at
  every start. The Settings DLSS switch does not turn DLSS off for these presets when NVIDIA's
  files are found: use F1.
- Other switches of the preset and the settings go as `--set NAME=value`, except Fast-list
  switches that `victory play` already sets the same way. Switches the F1 menu owns that have no
  play option (1440p, estimated light, early depth) are not passed: the F1 menu decides them, so
  the launcher's estimated-light setting does not apply to these presets (`presets::play_command`).
- The folders (game, cache, saves, NVIDIA Streamline, AMD FidelityFX frame generation, home) and
  the log level are passed in the environment, which `victory play` keeps; HybridGt's FidelityFX
  stand-in folder (`VICTORY_FFX_DIR`) goes as `--set`.
- `victory play` keeps `VICTORY_HOME` across its own first-start restart, so the F1 menu's file
  stays in HybridGt's home. Before that fix the restart fell back to `%LOCALAPPDATA%\victory`.

## Presets

| Preset | What it is |
|---|---|
| Fast | Series X profile at 1440p with the runtime's speed switches |
| Performance | Fast, plus the items below |
| All in one | Series S profile, FSR 3.1, the game's ray tracing and the reflections stand-in; experimental |
| Series S + FSR 3.1 | 720p inside, AMD FSR 3.1 to 1080p (AMD's DLLs) |
| Ray tracing | The game's own traced light; experimental |
| Classic | The earlier runtime (fxceless base), with its own saves |

Performance adds:

- DLSS Quality and frame generation 2x, only where NVIDIA's files (`sl.interposer.dll`) are found.
  An RTX 40 card generates 2x (the runtime reports more than 2x needs an RTX 50 card). With the
  MFG Unlocker on, the tester saw 6x on an RTX 4070 Ti, and cars looked and handled oddly while
  driving.
- Worker pools sized by logical CPUs, as VIctory-NVIDIA's `play-gta6-framegen.ps1` sizes them:

  | Logical CPUs | `VICTORY_THIN_THREADS` | `VICTORY_UNDER_THREADS` | `VICTORY_LAND_THREADS` |
  |---|---|---|---|
  | 24 or more | 12 | 8 | 8 |
  | 16 to 23 | 12 | 8 | 6 |
  | fewer than 16 | runtime default | runtime default | runtime default |

- `VICTORY_PLACED_ORDER=1`.
- The game process at High priority.

Upstream measured the pool and placed-order tuning at +22-29 % in heavy scenes on an RTX 4070
SUPER. HybridGt has not yet measured the Performance preset.

Frame generation multiplies the frames shown, not the game's own rate; generated frames guess
motion (see [PERFORMANCE.md](PERFORMANCE.md)).

## NVIDIA, AMD and mod files

HybridGt does not carry them. Settings has three buttons. Each runs the runtime's own command with
`VICTORY_HOME` set to HybridGt's home; the runtime fetches the files from their makers' own GitHub
and checks them by SHA-256. Nothing is fetched until a button is pressed. The log is under
Diagnostics.

| Button | Runs | Fetches | Into (under `%LOCALAPPDATA%\HybridGt\home`) |
|---|---|---|---|
| Get NVIDIA DLSS | `victory get-dlss` | NVIDIA Streamline: DLSS and DLSS Frame Generation (about 70 MB) | `nvidia-sdk` |
| Get AMD FSR 3 | `victory get-fsr` | AMD FidelityFX SDK: FSR 3 frame generation (about 30 MB) | `amd-fidelityfx` |
| Get MFG Unlocker (mod) | `victory get-mfg` | [MFG Unlocker](https://github.com/dashdogy/MFG-Unlocker), someone else's MIT mod for DLSS frame generation on RTX 20 and 30 cards | `mfg-unlocker` |

Get AMD FSR 3 brings FSR 3 frame generation only. The FSR 3.1 presets' upscaler also needs
`amd_fidelityfx_upscaler_dx12.dll` in the FidelityFX folder; without it they run without
HybridGt's FidelityFX stand-in.

MFG Unlocker is off until it is turned on in the game's F1 menu. An RTX 40 card does 2x without it
(the runtime reports more than 2x needs an RTX 50 card). With the MFG Unlocker on, the tester saw
6x on an RTX 4070 Ti (80-100+ fps shown), and cars looked and handled oddly while driving.

After a download, and on every scan, the launcher looks for NVIDIA Streamline
(`nvidia-sdk\Streamline\bin\x64\sl.interposer.dll`) and AMD FidelityFX
(`amd-fidelityfx\` or `amd-sdk\FidelityFX\bin\`, with `amd_fidelityfx_loader_dx12.dll`) in, in
order: HybridGt's home, `%LOCALAPPDATA%\victory`, and the checkout of a runtime it found. A folder
found is set where none is set yet.

VIctory-NVIDIA's own download scripts are also in [`tools/vendor`](../tools/vendor), unchanged and
credited, for running by hand.

## Offline

- The launcher has no network code. No network crate is a dependency, and no source file opens a
  socket, a URL or a browser. Two unit tests (`offline::tests`) fail otherwise.
- No update checks, telemetry, sign-in or cloud saves.
- Downloads happen only when a Get button is pressed, and the runtime makes them, not the launcher:
  NVIDIA's, AMD's or the mod author's files from their own GitHub, checked by SHA-256.
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

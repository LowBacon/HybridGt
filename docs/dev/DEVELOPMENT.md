# Development

[← README](../../README.md) · [Architecture](ARCHITECTURE.md) · [Shader translation](SHADER_TRANSLATION.md) · [Contributing](../../CONTRIBUTING.md)

HybridGt lives in two places:

| Repository | What you build there |
|---|---|
| **HybridGt** (this repository) | The launcher, `HybridGt.exe`; the diagnostic harness `hgt-gfxdiag`; the texture crate `hgt-texture`; the runtime patches |
| **VIctory-NVIDIA** at `df2daafe` plus `patches/victory-nvidia` | The runtime, `victory.exe` |

## Toolchain

| Tool | Version | Used for |
|---|---|---|
| Rust | 1.88 or later, MSVC toolchain, edition 2024 | Everything |
| Windows SDK 10.0.x | Any recent release | `rc.exe` (the launcher's icon and version details), `dxil.dll` (offline shader validation), `dxc.exe` (the harness's GPU checks and disassembly) |
| Git | Any | Applying the runtime patch |

All builds work offline once the dependencies are in Cargo's cache. Add `--offline` to any command
below to make sure nothing is fetched.

## Building the launcher

```powershell
cargo build --release -p hgt-launcher
```

The result is `target\release\HybridGt.exe`. `build.rs` draws the icon from `src/mark.rs`. It writes a
VERSIONINFO resource (product HybridGt, the crate version, GPL-3.0-or-later) and compiles both with the
Windows SDK's newest x64 `rc.exe`. Set `HGT_RC` to use another `rc.exe`. Without the SDK the build
still succeeds, with a warning and no icon.

To put the runtime inside the launcher, as releases do, set `HGT_RUNTIME_EXE` to a built
`victory.exe` before building. The launcher then unpacks the runtime on first start.

## Building the runtime

```powershell
git clone https://github.com/pepezin-ai/VIctory-NVIDIA
cd VIctory-NVIDIA
git checkout df2daafe
git switch -c hybridgt
git apply ..\HybridGt\patches\victory-nvidia\hybridgt-on-victory-nvidia-df2daafe.patch
cargo build --release -p victory
```

`df2daafe` is 50 upstream commits newer than `9d6300fa`, the base of HybridGt's earlier patch. The
patch holds 9 commits over 32 files; they are listed in
[`COMMITS-df2daafe.txt`](../../patches/victory-nvidia/COMMITS-df2daafe.txt). The patch on `9d6300fa`
([`hybridgt-on-victory-nvidia-9d6300fa.patch`](../../patches/victory-nvidia/hybridgt-on-victory-nvidia-9d6300fa.patch),
[`COMMITS.txt`](../../patches/victory-nvidia/COMMITS.txt)) stays for reference.

The result is `target\release\victory.exe`. Its commands, as the launcher uses them:

| Command | What it does |
|---|---|
| `victory shaders <game folder> --jobs N` | Translates every shader into the cache ahead of time |
| `victory warm` | Makes the pipelines a previous session recorded (`cache\pipelines\known-pipelines.bin`) |
| `victory run <game folder>\<executable> [--console series-x\|series-s] --execute` | Runs the game without the F1 menu or F3 panel. The launcher uses `run` only for the *Classic* preset, which uses the earlier runtime (see below) |
| `victory play [<executable>] [--dlss off\|dlaa\|quality\|balanced\|performance] [--framegen 2-6] [--fps N] [--series-s] [--set NAME=VALUE]` | Runs the game with the F1 settings menu and F3 stats panel; the F1 settings are saved in `victory-settings.txt` in the home folder: `VICTORY_HOME` if set (the launcher sets `%LOCALAPPDATA%\HybridGt\home`), else `%LOCALAPPDATA%\victory`. Non-Classic presets start this way |
| `victory get-dlss` | Fetches NVIDIA's DLSS and frame generation files from NVIDIA's GitHub (about 70 MB) |
| `victory get-fsr` | Fetches AMD's FSR 3 frame generation files from AMD's GitHub (about 30 MB) |
| `victory get-mfg` | Fetches MFG Unlocker, someone else's MIT mod ([github.com/dashdogy/MFG-Unlocker](https://github.com/dashdogy/MFG-Unlocker), about 5 MB), for DLSS frame generation on RTX 20 and 30 cards. F1's MFG UNLOCKER option, which loads it, is off by default |

`get-fsr`, `get-mfg` and `get-d3d12` check each download against a SHA-256. `get-dlss` checks only
each file's size against NVIDIA's archive. The launcher runs them only when its Settings buttons are
pressed; the launcher itself has no network code. `play` can still fetch by itself. It runs
`victory get-d3d12` (Microsoft's newer Direct3D 12 runtime from nuget.org, for Windows 10) where
Windows' own is too old, without asking when it has no console, as when the launcher starts it. It also
fetches MFG Unlocker when F1's MFG UNLOCKER is on.

The runtime reads its folders and switches from environment variables (`VICTORY_GAME_DIR`,
`VICTORY_CACHE_DIR`, `VICTORY_DATA_DIR`, `VICTORY_HOME`, and the preset's switches in
[`presets.rs`](../../crates/hgt-launcher/src/presets.rs)). F1-owned settings go to `play` as its options,
not as `--set`.

The *Classic* runtime is built from fxceless's victory-gta6, as described in
[docs/user/BUILD.md](../user/BUILD.md).

## Tests

**In this repository:**

```powershell
cargo test --release -p hgt-launcher
```

The launcher's 34 unit tests cover settings and their migration, presets and their switches, setup
progress, the output parsers, reading this PC, PNG decoding, playtime, and the offline policy. Two
tests fail the build if a network crate or network code appears.

```powershell
cargo test --workspace
```

This adds `hgt-gfxdiag` and `hgt-texture`. The GPU checks run on WARP, the software adapter, by
default, so a broken check can't reset a real GPU driver. `HYBRIDGT_ADAPTER=hw` runs them on the first
hardware GPU, and `HYBRIDGT_D3D12_DEBUG=1` turns on the D3D12 debug layer. Without DXC, GPU checks print
`SKIPPED`. A skip doesn't count as tested.

**In the runtime checkout:**

```powershell
cargo test --release --workspace --no-fail-fast
```

On the `df2daafe` build: 1,078 passed, 0 failed, 52 ignored (2026-11-28). The flaky upstream test passed in that run too.

On the earlier `9d6300fa` build this ran 1,032 unit and integration tests. Another 52 were ignored by
design: they need a title, or a long run. Some GPU tests use the first hardware GPU as well as WARP, so
close games first.

One upstream test, `victory-gpu`'s
`watch::tests::what_is_put_in_beside_the_guards_is_told_at_every_view`, can fail when tests run in
parallel, because they share the write-watch state. It passes run alone, and all 408 `victory-gpu`
tests pass with `--test-threads 1`. On 2026-11-27, on the `9d6300fa` build, the other 1,031 tests
passed. For the offline shader check against `dxil.dll`, see
[SHADER_TRANSLATION.md](SHADER_TRANSLATION.md#checking-it-yourself).

## Rules the code keeps

- **No game data in the repository, ever.** Not shaders, dumps, textures, logs with game paths, or
  screenshots of the game's art. The runtime's `victory-core` policy refuses to write game-derived
  output inside the source tree, and the offline shader test writes nothing.
- **The game folder is read only.** Nothing writes, renames or links into it.
- **Offline.** The launcher has no network dependency, which `offline.rs` enforces. The runtime keeps
  the game to loopback. Only the runtime's `get-*` commands download, and `play`'s own fetch of the
  Direct3D 12 runtime and MFG Unlocker (see above).
- **Measure before claiming.** A change that is meant to be faster comes with a before-and-after run on
  the same route. A rendering fix counts as done only once it has been seen in the game (see
  [STATUS.md](../STATUS.md)).
- **Credit.** Code from another fork keeps its author (cherry-pick with `-x`) and is listed in
  [THIRD_PARTY.md](../../THIRD_PARTY.md). GPL-2.0-only code (XCL) is never copied.
- **Style.** `cargo fmt` with the repository's settings. `cargo clippy` clean on new code. Comments say
  why, in plain words. Switches that change behaviour are named `VICTORY_*` (runtime) and documented
  where they are read.

## Updating the runtime patch

After committing on the `hybridgt` branch of the runtime checkout:

```powershell
git diff df2daafe HEAD > ..\HybridGt\patches\victory-nvidia\hybridgt-on-victory-nvidia-df2daafe.patch
git log --oneline df2daafe..HEAD > ..\HybridGt\patches\victory-nvidia\COMMITS-df2daafe.txt
```

Leave the `9d6300fa` patch and `COMMITS.txt` as they are; they are kept for reference.

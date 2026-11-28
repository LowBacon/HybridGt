# Development

[← README](../../README.md) · [Architecture](ARCHITECTURE.md) · [Shader translation](SHADER_TRANSLATION.md) · [Contributing](../../CONTRIBUTING.md)

HybridGt lives in two places:

| Repository | What you build there |
|---|---|
| **HybridGt** (this repository) | The launcher, `HybridGt.exe`; the diagnostic harness `hgt-gfxdiag`; the texture crate `hgt-texture`; the runtime patches |
| **VIctory-NVIDIA** at `9d6300fa` plus `patches/victory-nvidia` | The runtime, `victory.exe` |

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
git checkout 9d6300fa
git switch -c hybridgt
git apply ..\HybridGt\patches\victory-nvidia\hybridgt-on-victory-nvidia-9d6300fa.patch
cargo build --release -p victory
```

The result is `target\release\victory.exe`. Its commands, as the launcher uses them:

| Command | What it does |
|---|---|
| `victory shaders <game folder> --jobs N` | Translates every shader into the cache ahead of time |
| `victory warm` | Makes the pipelines a previous session recorded (`cache\pipelines\known-pipelines.bin`) |
| `victory run <game folder>\<executable> [--console series-x\|series-s] --execute` | Runs the game |

The runtime reads its folders and switches from environment variables (`VICTORY_GAME_DIR`,
`VICTORY_CACHE_DIR`, `VICTORY_DATA_DIR`, and the preset's switches in
[`presets.rs`](../../crates/hgt-launcher/src/presets.rs)).

The *Classic* runtime is built from fxceless's victory-gta6, as described in
[docs/user/BUILD.md](../user/BUILD.md).

## Tests

**In this repository:**

```powershell
cargo test --release -p hgt-launcher
```

The launcher's 32 unit tests cover settings and their migration, presets and their switches, setup
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

This runs 1,032 unit and integration tests. Another 52 are ignored by design: they need a title, or a
long run. Some GPU tests use the first hardware GPU as well as WARP, so close games first.

One upstream test, `victory-gpu`'s
`watch::tests::what_is_put_in_beside_the_guards_is_told_at_every_view`, can fail when tests run in
parallel, because they share the write-watch state. It passes run alone, and all 408 `victory-gpu`
tests pass with `--test-threads 1`. On 2026-11-27 the other 1,031 tests passed. For the offline shader check against `dxil.dll`, see
[SHADER_TRANSLATION.md](SHADER_TRANSLATION.md#checking-it-yourself).

## Rules the code keeps

- **No game data in the repository, ever.** Not shaders, dumps, textures, logs with game paths, or
  screenshots of the game's art. The runtime's `victory-core` policy refuses to write game-derived
  output inside the source tree, and the offline shader test writes nothing.
- **The game folder is read only.** Nothing writes, renames or links into it.
- **Offline.** The launcher has no network dependency, which `offline.rs` enforces. The runtime keeps
  the game to loopback.
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
git diff 9d6300fa HEAD > ..\HybridGt\patches\victory-nvidia\hybridgt-on-victory-nvidia-9d6300fa.patch
git log --oneline 9d6300fa..HEAD > ..\HybridGt\patches\victory-nvidia\COMMITS.txt
```

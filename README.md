# HybridGt

Research and engineering toward running an Xbox Series X|S title's files, owned by the user, on Windows x64.
It builds on the public work of VIctory and XCL and adds what they lack.

**Status: no game frame rendered. No gameplay.** HybridGt is not a runtime yet. What exists:

| Part | What it does | Evidence |
|---|---|---|
| `crates/hgt-texture` | BCPack unpacking with strict failure semantics (VIctory's decoder behind an adapter), a requested/filled ledger, and BC1 reference decoders (exact and NVIDIA) | 14 unit tests |
| `crates/hgt-gfxdiag` | D3D12 diagnostic harness: clear/readback, BC1 upload and fetch, indirect draws with GPU-produced counts, and a count clamp guard | 10 integration tests (WARP); CLI run on WARP and on an RTX 4070 Ti |
| `docs/COMPATIBILITY_MATRIX.md` | XCL vs VIctory, from source and public issues, each cell labelled by evidence | — |
| `docs/findings/` | Reproducible findings | — |

No game data is in this repository, and none was used to produce these results.

## Build and test

Requirements: Windows 10/11 x64, Rust (stable MSVC), and DXC on `PATH` or in `HYBRIDGT_DXC` (Windows SDK or
Vulkan SDK). Without DXC the GPU checks print `SKIPPED` and pass; skips are not coverage.

```powershell
cargo test --workspace                 # GPU checks run on WARP (software); safe on any machine
cargo run -p hgt-gfxdiag               # the harness, one line per check
$env:HYBRIDGT_D3D12_DEBUG = "1"        # D3D12 debug layer: errors fail a check
$env:HYBRIDGT_ADAPTER = "hw"           # run on the first hardware adapter
```

Hardware runs refuse the checks that hung a GPU before, unless `HYBRIDGT_ALLOW_HANG_REPRO=1` is set (this may
reset the display driver).

## Licence

GPL-3.0-or-later. Third-party use is listed in [THIRD_PARTY.md](THIRD_PARTY.md).

# HybridGt master status

Updated 2026-11-24. Hardware: RTX 4070 Ti, Windows 11, D3D12. Everything below was
measured; nothing is claimed beyond what a run or test showed.

## Workspace rule (from 2026-11-24 brief)

From this brief on, `E:\GTA\GTA6.Xbox.v1.0.2.1` is not accessed by the agent in any way,
and new work lives only in `E:\GTA\HybridGt`. Game runs listed below (runs 1–19) were
made earlier under the user's read-only permission; they are not repeated by the agent.
The user can still launch the game themselves (see README).

## What HybridGt is

| Part | What it does | Status | Evidence |
|---|---|---|---|
| `crates/hgt-texture` | Strict BCPack unpacking over VIctory's decoder; BC1 reference decoders (exact + NVIDIA) | Works on synthetic fixtures | 14 unit tests pass |
| `crates/hgt-gfxdiag` | D3D12 harness: clear, BC1 upload/sample, indirect draw counts with clamp guard, view-heap limit probe, buffer read benchmark | Works on RTX 4070 Ti | 10 tests pass; probe: 1,000,000-slot heap cap |
| `patches/victory/0001–0011` | Patch series on VIctory `ffe310e` (wave64 emulation, queue waits, heap caps, watchdog switch, diagnostics) | Experimental, opt-in env switches | Runs 16–19 below |
| `tools/victory_logstats.py` | Summarises VIctory logs (drops, refused pipelines) | Works | used on runs 6–19 |

HybridGt has no runtime of its own: the game runs inside VIctory with these patches.

## Best measured result (run 17, patches 0001–0010)

- Boots, menus, intro video and audio, loads the world, reaches the Prologue.
- 8+ minutes without a GPU reset (user report); run 18 reset once at frame 2,113.
- Screen black except the "PROLOGUE" title card. About 1 fps.

## Not working

See `KNOWN_FAILURES.md` and `ROOT_CAUSE_ANALYSIS.md`. In short: world geometry,
lighting, characters, gameplay, performance.

## Tests

`cargo test --workspace`: 24 passed, 0 failed (2026-11-24).

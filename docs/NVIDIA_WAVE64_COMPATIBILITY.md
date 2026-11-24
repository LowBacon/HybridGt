# NVIDIA wave64 compatibility

## Measured (2026-11-24, `hgt-gfxdiag` check `wave64`)

| Adapter | Host lanes | Native ballot = console wave64 ballot | Two-half emulated ballot = console |
|---|---|---|---|
| RTX 4070 Ti | 32 | **no** | yes |
| WARP | (reported by test) | per lane count | yes |

Test: one 64-thread group, predicate `t % 3 == 0`. The console (RDNA2, wave64)
answers one 64-bit mask for all 64 threads. On the 4070 Ti each thread sees only
its own 32-lane half, so any shader that reads a ballot, lane index, mbcnt,
readlane or a DPP/permute across lanes 32–63 gets wrong data. Emulating the wave
as two halves joined through group-shared memory gives the console answer.
Reproduce: `HYBRIDGT_ADAPTER=hw cargo test -p hgt-gfxdiag wave64 -- --nocapture`.

## Why this matters for the black world

All 9,282 mesh shaders of the title are wave64. VIctory's wave64 emulation
(patch 0001) covers compute shaders only. Two outcomes seen:

- Run 19: an unfinished mesh-emulation experiment refused 469 mesh shaders
  ("emulated wave operation outside uniform control flow"); their draws were
  dropped, so the G-buffer stayed empty. The experiment is now set aside.
- Run 20 (without it): mesh-shader failures fell to 11, but the GPU was removed
  ~frame 1,651 once the world began drawing. Leading cause (unproven): mesh
  shaders running with 32-lane semantics on wave64 code.

## Still needed

1. A mesh-stage version of this check (emulation in a mesh shader with its
   28 KiB group-shared limit and SetMeshOutputCounts uniformity rule).
2. Mesh-stage emulation in the translator that also handles cross-lane ops
   inside divergent control flow (the 469 refusals).
3. Vertex and pixel stages: cannot use group-shared memory; need a separate
   strategy (e.g. per-wave-half analysis or WaveSize-aware rewrite).

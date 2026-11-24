# Engineering status

Updated 2026-11-24. Every result below is labelled:
- **measured**: produced by a run here;
- **source**: read in code or public issues;
- **suspected**: not yet confirmed;
- **UNVERIFIED**: implemented but not tested.

## Current state

| Milestone | State | Evidence |
|---|---|---|
| A. Startup: launch, runtime, audio, subtitles | **Reached with VIctory** | measured: boot, intro and menus at 30.0 fps; audio, subtitles and the first mission's title card on an RTX 4070 Ti |
| B. Rendering: models, textures, lighting | **Not reached** | measured: world frame 1750's depth and G-buffer hold only wires and thin objects; another pass shows a car, people and buildings; the final image is black |
| C. World stability: 10,000 frames, no reset | **Not reached** | measured: device removed 0–300 frames into the world in runs 3, 4, 5 and 6 |
| D. Gameplay | Not reached | — |
| E. Performance: 30–60 fps | **Not reached** | measured: about 1 fps in the world (0.6–2.4 s/frame) |

Setup:
- Runtime under test: VIctory `ffe310e` plus the HybridGt patches in `patches/victory/`, built from an isolated clone
  (`E:\GTA\HybridGt-work\victory`, branch `hybridgt/count-guard`).
- Game: the user's own dump, read only. File list, sizes and times are verified unchanged after the runs.
- XCL: not run yet.

## Fixes

| # | Fix | Where | Test | Result |
|---|---|---|---|---|
| 1 | Write-watch answers reused within one submission | `patches/victory/0001` | unit test `epoch_tests`; runs 3 → 4 | measured: watch time 294 → 142 ms/frame, frame 1266 → 1140 ms (more draws in run4) |
| 2 | Capped descriptor heap (NVIDIA: 1,000,000 slots) gets 2^18 dynamic slots instead of 2^16 | work clone, uncommitted until run7 | descriptor unit tests 7/7; run7 | **UNVERIFIED** (run7 in progress) |
| — | Loop cap 4096 (`VICTORY_SHADER_LOOP_LIMIT`) | env only | run6 | measured: **did not prevent the hang**; rejected as a fix |

## Known issues, in work order

1. **GPU hang in the world** (measured, runs 3–6). DRED places it in direct `DispatchMesh` of translated mesh pipelines, a
   different pipeline each time, with no page fault. Suspected cause: the descriptor shortage below (VIctory's own
   comment links 2^16 dynamic slots to device removals on AMD, #59).
2. **Descriptor heap on NVIDIA** (measured).
   - The title's view heap is 1,245,184 slots. The RTX 4070 Ti's limit is exactly 1,000,000 (hgt-gfxdiag probe;
     WARP allows 2,097,152).
   - VIctory capped the heap at 934,464 title slots plus 65,536 dynamic slots.
   - Draws were dropped with "no dynamic slots for its tables", 16,384+ by the end of run6.
   - No title slot past 934,464 was written in runs 4 or 6. → Fix 2.
3. **Main geometry missing from depth and G-buffer** (measured, frame 1750). Candidates:
   - dropped draws: dynamic slots (#2), untranslated shaders (#4), refused pipelines (#5);
   - dropped indirect draws: 560–2,000 per second.
4. **437 shaders do not translate** (measured): compute 367, pixel 37, mesh 28, vertex 5. Reasons are listed in
   `docs/findings/2026-11-24-victory-world-on-rtx.md`.
5. **235 pipelines refused by the host** (measured, 0x80070057).
6. **Frame cost** (measured): copies of title memory 450–1,370 ms, write watch, 30–36 serialized submissions per frame,
   copy-back 45–335 ms, page faults 65k–1M per frame, GPU 170–600 ms.
7. Smaller API gaps (measured):
   - texture formats 86 and 115 not created;
   - device slots 0x218 and 0x1c8 not implemented;
   - UAV format 53;
   - 2 title textures released while still in use.
8. Indirect counts are not clamped in VIctory (source). An unclamped count hung the RTX once in hgt-gfxdiag;
   `CountGuard::Clamp` is verified there, not yet in VIctory.
9. A zlib stream that ends short of its destination passes in VIctory's DirectStorage (source).
10. XCL is not investigated beyond the source read and #878.

## Tests

- `cargo test --workspace` (HybridGt): 24 passed, 0 failed (WARP).
- `hgt-gfxdiag` on the RTX 4070 Ti: 10 passed, 2 skipped (unguarded garbage counts, which hang this GPU).
- VIctory work clone: `cargo test -p victory-gpu epoch` 1 passed; `cargo test -p victory-gpu descriptor` 7 passed.
  The full VIctory suite has not been run on the work clone yet.

## Next priority

Read run7.
- If the world survives and the "no dynamic slots" drops are gone: compare its render targets with run6's frame 1750,
  then take the remaining dropped draws in order of count.
- If it still hangs: rerun with the D3D12 debug layer and GPU-based validation, and narrow it to the descriptor reads
  of the hanging mesh pipelines.

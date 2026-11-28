# Roadmap

[← README](../README.md) · [Known issues](KNOWN_ISSUES.md) · [Changelog](../CHANGELOG.md)

Ordered by what most improves the game for the effort. Each item names how it will be checked. An item
is done only when that check passes.

## Done

| Milestone | Checked by |
|---|---|
| The game boots, menus and videos at 30 fps | Game sessions |
| Whole characters, lit interiors (estimated light), working boats | Game sessions (tester) |
| AMD FSR 3.1 through the game's own calls (earlier runtime) | Game session: FSR 3.1.6, 720p → 1080p |
| Mouth animation (fxceless #927) | Game session (earlier runtime) |
| Runtime moved to VIctory-NVIDIA: 14.2 fps median, 5.6 GB | Same-route measurement, back to back |
| One-file launcher: scan, presets, setup with real progress, offline policy | 32 unit tests; built offline |
| Every vertex and pixel program passes Microsoft's validator; 98.9 % of all programs | Offline translate-and-validate test |

## Next

| # | Work | Why | Check |
|---|---|---|---|
| 1 | Remap descriptor views past Direct3D 12's 1,000,000-view cap | Eyes and surfaces that read an empty slot | No *"host view heap capped"* in the log; cutscene eyes compared with the tester's reference |
| 2 | Track texture rewrites | Textures in the wrong colour or blurry while loading | Fewer *"changed after its upload"* lines; tester |
| 3 | Fit the 71 oversized mesh programs: two 16-bit values per output, interpolated in the pixel shader, or two host groups per console group (pruning was ruled out: their pixel shaders read every output) | 71 programs over 32 KB | Offline validator: mesh 8,765 of 8,765 |
| 4 | 64-lane waves on 32-lane GPUs through group-shared memory | 23 compute programs | WARP unit tests against console results; offline validator |
| 5 | Geometry stage for the voxelizer | 195 programs, voxelized lighting | Offline validator; then a game session |
| 6 | Fewer CPU-side copies | The frame is bound by keeping two memories in step | Same-route fps measurement |
| 7 | Measure the memory modes, frame caps, FSR 3.1 and DLSS on the current runtime | They are built but unmeasured | Same-route runs with each setting |
| 8 | Test on AMD Radeon | Only NVIDIA is verified | A tester's run with the Diagnostics summary |

## Goal

**30 fps in the open world**, the game's frame rate on console, with a correct image, on a 12 GB card.
That is a target, not a promise. Today's measured figure is 14.2 fps median.

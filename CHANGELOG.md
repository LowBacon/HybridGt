# Changelog

## 2026-11-24
- Milestone: 3D models and lighting visible on RTX 4070 Ti using the fxceless/victory-gta6 fork (user report). Not playable: 0.4 fps, flicker on first start.
- VIctory patch 0014: mesh programs the wave64 emulation cannot follow fall back to host waves (counted). Offline translate: 8,214 of 8,713 mesh programs emit (1,486 emulated, 6,728 fallback), was 1,629 with 0013 alone.
- VIctory patches 0012–0013: guarded wave ops by predication; wave64 emulation in mesh shaders.
- gfxdiag: divergent wave64 ballot by predication proven correct (compute + mesh, RTX 4070 Ti).
- gfxdiag: wave64 check also runs as a mesh shader (passes on RTX 4070 Ti and WARP).
- gfxdiag: `wave64` check — measures host wave size and proves two-half
  wave64 ballot emulation against the console answer (RTX 4070 Ti: 32 lanes).
- Patch series 0001–0011 exported (adds wave64 emulation, queue waits, title).
- Found: run 19's empty G-buffer coincides with 469 mesh shaders refused by an
  unfinished experiment; removing it exposes a GPU removal at world start.
- Docs: MASTER_STATUS, KNOWN_FAILURES, ROOT_CAUSE_ANALYSIS, TEST_RESULTS,
  NVIDIA_WAVE64_COMPATIBILITY.

# Changelog

## 2026-11-24
- gfxdiag: `wave64` check — measures host wave size and proves two-half
  wave64 ballot emulation against the console answer (RTX 4070 Ti: 32 lanes).
- Patch series 0001–0011 exported (adds wave64 emulation, queue waits, title).
- Found: run 19's empty G-buffer coincides with 469 mesh shaders refused by an
  unfinished experiment; removing it exposes a GPU removal at world start.
- Docs: MASTER_STATUS, KNOWN_FAILURES, ROOT_CAUSE_ANALYSIS, TEST_RESULTS,
  NVIDIA_WAVE64_COMPATIBILITY.

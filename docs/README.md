# HybridGt documentation

[← Back to the project README](../README.md)

## For players

| Document | Contents |
|---|---|
| [user/TROUBLESHOOTING.md](user/TROUBLESHOOTING.md) | Symptoms, causes and fixes; how to report a problem |
| [LAUNCHER.md](LAUNCHER.md) | The launcher's pages, the INITIALIZE steps, offline policy and identity |
| [WINDOWS_COMPATIBILITY.md](WINDOWS_COMPATIBILITY.md) | What the runtime needs from Windows and the graphics card; upscalers; network; folders |
| [GPU_MEMORY.md](GPU_MEMORY.md) | Why graphics memory fills up, and the memory modes |
| [KNOWN_ISSUES.md](KNOWN_ISSUES.md) | Current issues and what is known about each |
| [user/BUILD.md](user/BUILD.md) | Building and running the *Classic* runtime by hand |

## For developers

| Document | Contents |
|---|---|
| [dev/ARCHITECTURE.md](dev/ARCHITECTURE.md) | Where the code comes from, the runtime's crates, what the HybridGt patch changes, the launcher's modules |
| [dev/SHADER_TRANSLATION.md](dev/SHADER_TRANSLATION.md) | How console shaders become DXIL, current validator results, what is left, the offline check |
| [dev/DEVELOPMENT.md](dev/DEVELOPMENT.md) | Toolchain, builds, tests, the rules the code keeps, updating the patch |
| [../CONTRIBUTING.md](../CONTRIBUTING.md) | How to contribute |
| [NVIDIA_WAVE64_COMPATIBILITY.md](NVIDIA_WAVE64_COMPATIBILITY.md) | 64-lane console programs on 32-lane GPUs |
| [GEOMETRY_SHADERS.md](GEOMETRY_SHADERS.md) | Notes on the console's geometry stage |
| [SERIES_S_AND_FSR.md](SERIES_S_AND_FSR.md) | Series S mode and the FidelityFX path |

## Status and measurements

| Document | Contents |
|---|---|
| [PERFORMANCE.md](PERFORMANCE.md) | Latest measurements, method, where a frame's time goes |
| [STATUS.md](STATUS.md) | Every problem on a five-step scale, from found to seen fixed in game |
| [ROADMAP.md](ROADMAP.md) | Done, next, and how each item will be checked |
| [PERFORMANCE_OPTIMIZATION.md](PERFORMANCE_OPTIMIZATION.md) | The full log of runs and measurements |
| [UPSTREAM_SWEEP_2026-11-28.md](UPSTREAM_SWEEP_2026-11-28.md) | New upstream commits and ports, ranked as merge candidates (research only) |
| [MISSION_REPORT.md](MISSION_REPORT.md) | The launcher 0.3.0 work and the offline log analysis |

## History

Earlier runtimes and investigations, kept for reference:
- [KNOWN_FAILURES.md](KNOWN_FAILURES.md), [ROOT_CAUSE_ANALYSIS.md](ROOT_CAUSE_ANALYSIS.md) and [MASTER_STATUS.md](MASTER_STATUS.md);
- [PERFORMANCE_BASELINE.md](PERFORMANCE_BASELINE.md), [PERFORMANCE_ARCHITECTURE.md](PERFORMANCE_ARCHITECTURE.md) and [TEST_RESULTS.md](TEST_RESULTS.md);
- [COMPATIBILITY_MATRIX.md](COMPATIBILITY_MATRIX.md);
- [ZTRX_FEATURE_MATRIX.md](ZTRX_FEATURE_MATRIX.md), [ZTRX_IMPLEMENTATION_AUDIT.md](ZTRX_IMPLEMENTATION_AUDIT.md) and [ZTRX_TO_HYBRIDGT_PORTING_PLAN.md](ZTRX_TO_HYBRIDGT_PORTING_PLAN.md);
- [findings/](findings).

# Performance

[← README](../README.md) · [GPU memory](GPU_MEMORY.md) · [Full run log](PERFORMANCE_OPTIMIZATION.md)

This page gives the latest measurements, how they were taken, and what limits the frame rate. Every
run, including the ones that didn't help, is in
[PERFORMANCE_OPTIMIZATION.md](PERFORMANCE_OPTIMIZATION.md).

## Test system

| | |
|---|---|
| GPU | NVIDIA GeForce RTX 4070 Ti, 12 GB |
| CPU | AMD Ryzen 7 5800X3D (8 cores, 16 threads) |
| Memory | 32 GB |
| OS | Windows 11 |
| Game | Xbox Series build v1.0.2.1, the tester's own copy |

## Latest measurement (2026-11-27)

The method:
- the same scripted route through the open world, with both runtimes back to back;
- shader caches warmed and pipelines made beforehand, so neither run was compiling;
- the game draws at 2560×1440 (Series X profile) and its own temporal upscaler outputs 3840×2160;
- frame rates come from the runtime's per-second frame line.

| Runtime | Median fps | Best second | 10th percentile | Median frame time | Graphics memory at the end |
|---|---:|---:|---:|---:|---:|
| Earlier runtime: HybridGt on victory-gta6 (now *Classic*) | 5.8 | 7.4 | 1.9 | 174 ms | 9.2 GB |
| **VIctory-NVIDIA `9d6300fa` (now the default)** | **14.2** | **18.0** | **3.5** | **70 ms** | **5.6 GB** |

What the numbers mean:

- **Median fps** is the middle value of the per-second readings: half the seconds were faster, half
  slower. Unlike an average, it isn't skewed by loading stalls.
- **10th percentile** is the speed the slowest tenth of the seconds stayed under. It is the closest
  measure we have to the "1 % low" figures many benchmarks report. True 1 % lows were **not
  measured**, because they need per-frame timings, and the runtime logs once a second.
- **Median frame time** is how long a typical frame took. 70 ms is about 14 frames a second; 33 ms
  would be 30.
- **Graphics memory** is what the runtime held at the end of the route. Most of it is a texture
  cache, not a leak (see [GPU_MEMORY.md](GPU_MEMORY.md)).

Menus, videos and the boot sequence run at the game's own cap of 30 fps on both runtimes. Average
fps wasn't recorded separately. The tester described the new runtime as "a stable 15 fps" in play.

## Measured, estimated and targeted

| Kind | Figure |
|---|---|
| **Measured** | 14.2 fps median in the world, 18.0 best, 70 ms median frame, 5.6 GB graphics memory (table above) |
| **Measured** | 30 fps in menus and videos |
| **Not yet measured** | The launcher's Balanced and Low memory modes, the 30/60 frame caps, FSR 3.1 on the new runtime, DLSS |
| **Target** | 30 fps in the world, the game's own frame rate on console |

## Where a frame's time goes

A detailed breakdown was taken on the **earlier** runtime, with frames of about 150 ms. The new runtime
copies memory faster, but the shape is the same:

| Thread | Time per frame | Doing what |
|---|---|---|
| Game's render thread | 110–140 ms | Mostly **waiting** inside its present call for the runtime |
| Runtime's submitting thread | 140–160 ms | Finding changed title memory (25–42 ms), copying it to the card (30–80 ms), refreshing views (17–24 ms), hashing copies (15–18 ms) |
| GPU | ~91 ms busy | 59 ms the game's own work, ~31 ms the runtime's hashing and comparing; idle ~45 ms waiting |

**The frame rate is limited by the CPU side of the runtime**, keeping two memories in step where a
console has one, and not by the GPU. About 100 MB a frame that shaders write is also copied back into
title memory. This is why the fastest gains so far came from faster copies (foxhound's) and fewer
copies, rather than from GPU settings.

## Other limits seen in the logs

| Limit | Effect | Status |
|---|---|---|
| Textures rewritten in place | Uploaded again up to 16 frames late: blurry or wrong-coloured surfaces | Investigating |
| Descriptor heap cap (1,000,000 views) | Views past it read an empty slot | Investigating; needs view remapping |
| First use of a pipeline | A stall the first time an effect is seen | INITIALIZE makes the recorded ones ahead of time |
| Temporal upscaler at low fps | Soft hair and faces | Needs a higher frame rate |
| Mouth timing | At 70 ms frames, a frame reaches the screen several frames after its mouth shape was chosen | `VICTORY_AUDIO_DELAY_MS` delays the sound to match |

## Reporting a benchmark

Please include:
- your GPU, driver version, CPU and RAM;
- the preset;
- whether INITIALIZE had finished, including the pipeline step;
- the route;
- the launcher's **Diagnostics** summary of the run's log.

The per-second frame lines are in the log (`fps`, `ms/frame`), and `tools/victory_logstats.py`
summarizes them.

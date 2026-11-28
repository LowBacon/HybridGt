# Performance

[← README](../README.md) · [GPU memory](GPU_MEMORY.md) · [Full run log](PERFORMANCE_OPTIMIZATION.md)

This page gives the latest same-route benchmark, newer indicative readings, how they were taken, and
what limits the frame rate. Every run, including the ones that didn't help, is in
[PERFORMANCE_OPTIMIZATION.md](PERFORMANCE_OPTIMIZATION.md).

## Test system

| | |
|---|---|
| GPU | NVIDIA GeForce RTX 4070 Ti, 12 GB |
| CPU | AMD Ryzen 7 5800X3D (8 cores, 16 threads) |
| Memory | 32 GB |
| OS | Windows 11 |
| Game | Xbox Series build v1.0.2.1, the tester's own copy |

## Latest same-route benchmark (2026-11-27)

This is still the latest same-route, back-to-back benchmark. It was taken on the VIctory-NVIDIA
`9d6300fa` base. The runtime has since moved to VIctory-NVIDIA `df2daafe`; a same-route benchmark of
that build is not yet measured (see the indicative readings below).

The method:
- the same scripted route through the open world, with both runtimes back to back;
- shader caches warmed and pipelines made beforehand, so neither run was compiling;
- the game draws at 2560×1440 (Series X profile) and its own temporal upscaler outputs 3840×2160;
- frame rates come from the runtime's per-second frame line.

| Runtime | Median fps | Best second | 10th percentile | Median frame time | Graphics memory at the end |
|---|---:|---:|---:|---:|---:|
| Earlier runtime: HybridGt on victory-gta6 (now *Classic*) | 5.8 | 7.4 | 1.9 | 174 ms | 9.2 GB |
| **VIctory-NVIDIA `9d6300fa` (the default at the time)** | **14.2** | **18.0** | **3.5** | **70 ms** | **5.6 GB** |

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

## Indicative readings on the merged build (2026-11-28)

**These are not a same-route benchmark.** They come from ordinary play sessions on the test PC, on
different routes and settings, not back to back. Use them as indicative only. The 2026-11-27 table
above remains the latest same-route benchmark.

"Merged build" means HybridGt's patch on VIctory-NVIDIA `df2daafe`.

| Build | Where | Settings | Reading |
|---|---|---|---|
| Upstream VIctory-NVIDIA `df2daafe`, downloaded, without HybridGt | The open world, the tester's own session, 441 world seconds | DLSS on, frame generation on | Median 13.2 fps; 10th percentile 10.2; 90th percentile 16.6 |
| Merged build | An interior, while pipelines were still being made (about 290 draws a frame skipped) | — | 17.3–17.9 fps per-second readings |
| Merged build | The same interior, read from the F3 panel | Frame generation off | 12.8 fps, 77.9 ms frame, longest frame 103 ms |

Notes:

- The tester remembered about 20 fps on the upstream build. That was the best moments; the median was
  13.2.
- In the interior reading, draws were skipped while their pipelines were made. A skipped draw costs
  nothing, so those readings are likely higher than the same place with every draw.
- The 30 fps target in the world is **not reached**.

### F3 panel: where the frame went

Merged build, the interior above, frame generation off:

| Item | Per frame |
|---|---:|
| Frame time | 77.9 ms (12.8 fps); longest 103 ms |
| Runtime's submitting thread, busy | 65.7 ms |
| GPU, busy | 37.5 ms |
| GPU, idle waiting for the runtime | 35.6 ms |
| Copying back what the GPU wrote | 16.5 ms |
| Graphics memory | 7.7 of 11.0 GB, plus 6.0 GB shared (system memory) |
| System memory (RAM) | 23.1 GB |

What it means:

- The GPU was idle for 35.6 ms of each frame, almost as long as it was busy (37.5 ms). It was waiting
  for the runtime.
- The runtime's submitting thread was busy 65.7 ms of the 77.9 ms frame. Copying back what the GPU
  wrote took 16.5 ms a frame.
- So the frame rate is **limited on the CPU side**: preparing title memory for the GPU and copying it
  back. It is not limited by the GPU. This is the same shape as the earlier breakdown below.
- Settings that only lower GPU work (DLSS, a lower resolution) are unlikely to remove this limit, because
  the GPU is already waiting; not yet measured.
- 6.0 GB more was held in shared (system) memory, beyond the 7.7 GB on the card. See
  [GPU_MEMORY.md](GPU_MEMORY.md).

### Frame generation

- On an RTX 4070 Ti, DLSS Frame Generation runs at 2x. The runtime reports
  "MORE THAN 2X NEEDS AN RTX 50 CARD".
- With 6x through the MFG Unlocker (someone else's MIT mod, off by default), the tester saw 80–100+
  fps shown. Cars looked and handled oddly while driving.
- Frame generation makes extra frames between the game's real frames. It **shows** more frames; it
  does not raise the game's own rate. The game still updates at its own rate (about 13 fps in these
  sessions), so it does not respond faster.
- Generated frames guess the motion between two real frames. With few real frames and fast motion,
  such as driving, the guesses are often wrong. This is the likely reason cars looked odd at 6x.
- Frame generation also takes 1.5–2 GB of graphics memory. The runtime then caps textures at 80 %,
  which can leave textures black or late.

## Measured, estimated and targeted

| Kind | Figure |
|---|---|
| **Measured** | 14.2 fps median in the world, 18.0 best, 70 ms median frame, 5.6 GB graphics memory (`9d6300fa` build, 2026-11-27 table above) |
| **Measured** | 30 fps in menus and videos (`9d6300fa` build and the earlier runtime; not yet re-checked on `df2daafe`) |
| **Indicative** | 2026-11-28 readings on the upstream `df2daafe` build and the merged build (section above); not a same-route benchmark |
| **Not yet measured** | A same-route benchmark of the merged `df2daafe` build, with and without DLSS and frame generation; the launcher's Performance preset (upstream measured its worker-pool and placed-order tuning at +22–29 % in heavy scenes on an RTX 4070 SUPER; HybridGt has not measured it); the launcher's Balanced and Low memory modes; the 30/60 frame caps; FSR 3.1 on the new runtime |
| **Target** | 30 fps in the world, the game's own frame rate on console; not reached |

## Where a frame's time goes

A detailed breakdown was taken on the **earlier** runtime, with frames of about 150 ms. The new runtime
copies memory faster, but the shape is the same (the 2026-11-28 F3 panel reading above shows it on the
merged build):

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
| First use of a pipeline | On the merged build, draws are skipped (not drawn) until their pipelines are made; 88,782 pipelines recorded | INITIALIZE's pipeline step (`victory warm`) makes the recorded ones ahead of time, about 35 minutes with the game closed |
| Temporal upscaler at low fps | Soft hair and faces | Needs a higher frame rate |
| Graphics memory full | Black or late textures; about 79,500 "no room" messages in one session with frame generation on | Frame generation takes 1.5–2 GB; the runtime then caps textures at 80 % |
| Mouth timing | At 70 ms frames, a frame reaches the screen several frames after its mouth shape was chosen | `VICTORY_AUDIO_DELAY_MS` delays the sound to match |

## Reporting a benchmark

Please include:
- your GPU, driver version, CPU and RAM;
- the preset;
- whether DLSS and frame generation were on, and at what multiple;
- whether INITIALIZE had finished, including the pipeline step;
- the route;
- the launcher's **Diagnostics** summary of the run's log.

The per-second frame lines are in the log (`fps`, `ms/frame`), and `tools/victory_logstats.py`
summarizes them.

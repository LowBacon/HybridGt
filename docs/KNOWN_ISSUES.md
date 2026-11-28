# Known issues

[← README](../README.md) · [Status of every fix](STATUS.md) · [Troubleshooting](user/TROUBLESHOOTING.md)

These are the issues on HybridGt's runtimes, with what is known about each. Older issues from earlier
runtimes are kept in [KNOWN_FAILURES.md](KNOWN_FAILURES.md) and
[ROOT_CAUSE_ANALYSIS.md](ROOT_CAUSE_ANALYSIS.md).

**Runtimes:**
- **df2daafe** (current, the merged build for v0.3.0): VIctory-NVIDIA `df2daafe` plus the HybridGt patch.
- **9d6300fa** (previous): VIctory-NVIDIA `9d6300fa` plus the HybridGt patch.

The first two sections (Seen on the merged build, Fixed) are from the merged build on 2026-11-28. The
sections after them were found on `9d6300fa` unless a row says otherwise, and have not all been
re-checked on `df2daafe`.

**Evidence:**
- **Seen**: reported by the tester in a game session.
- **Logged**: read in the runtime's logs.
- **Measured**: timed on the same route, back to back.
- **Indicative**: readings from a play session, not a same-route benchmark.
- **Offline**: found by translating or testing without the game.

## Seen on the merged build (df2daafe)

| Issue | Evidence | What is known | Status |
|---|---|---|---|
| Reflections look wrong: blue, too bright in rooms; cyan sparkle on glass; grainy floors | Seen, Logged | The runtime does not build the title's ray-tracing structures, so reflection rays miss and get the title's sky light (blue, far brighter than rooms). The runtime's reflection estimate (`VICTORY_AMBIENT_SPECULAR`) replaces it on opaque surfaces only (log: *"surfaces: opaque"*, about 110 dispatches per 10 s). Glass and other transparent surfaces keep the sky light. `VICTORY_AMBIENT_SPECULAR_ALPHA=1` covers them too, but upstream left it off because on water it made dark patches (it reads the depth under the water). Floors keep grainy half-resolution reflection noise. | A test with `VICTORY_AMBIENT_SPECULAR_ALPHA=1` is prepared; not yet reported |
| White screen while driving that fades slowly | Seen (sometimes) | Suspected: the temporal upscaler's (DLSS) history after a bad frame. Not confirmed. | Test: turn DLSS off in the F1 menu |
| Black or late textures, worse with frame generation on | Seen, Logged | Graphics memory is full: *"the card has no room for new ones"*, about 79,500 times in one session with frame generation on. Frame generation takes 1.5–2 GB, and the runtime then caps textures at 80 %. See [GPU_MEMORY.md](GPU_MEMORY.md). | Open; turning frame generation off should free 1.5–2 GB (not yet tested whether textures then load in time) |
| Draws skipped until their pipelines are made | Seen, Logged | 88,782 pipelines are recorded. Until a pipeline is made, its draws are skipped (about 290 draws a frame in one interior). `victory warm` makes them ahead of time, with the game closed (about 35 minutes). | Open; run `victory warm` once |
| High system RAM use | Seen, Logged | About 20–23 GB for the game process on a 32 GB PC (23.1 GB on the F3 panel in one interior). | Open |
| Cars look and handle oddly with 6x frame generation | Seen | Through the MFG Unlocker (someone else's mod, off by default) on an RTX 4070 Ti, 80–100+ fps were shown. Frame generation shows extra frames; it does not raise the game's own rate, and generated frames guess motion. Without the mod, DLSS Frame Generation runs at 2x on that card. | Likely from generated frames (they guess motion); not yet tested whether 2x or off looks and handles normally |

## Fixed

| Issue | Runtime | What happened | Fix |
|---|---|---|---|
| Crash about 50 s after loading a save (integer divide by zero, `0xc0000094`, in the game's own code) | df2daafe, before the fix | Seen twice at the same place. HybridGt's FidelityFX module claimed `amd_fidelityfx_upscaler_xbox.dll` with no AMD PC files, so the title's upscaler context failed. Upstream's DLSS interposer saw no context (0 upscales by DLSS; log: *"an upscale dispatch on a context we did not see made"*), and the title divided by zero. | Fixed 2026-11-28. The FidelityFX stand-in now runs only when `VICTORY_FFX_DIR` names a folder with `amd_fidelityfx_loader_dx12.dll` and `amd_fidelityfx_upscaler_dx12.dll`. After the fix DLSS took the upscales (324 and counting), the warning did not appear, and a session ran about 12 minutes past the old crash point and closed normally. |
| F1 menu settings saved to the wrong folder after the first start | df2daafe, before the fix | Started through `victory play`, settings save to the home folder (`VICTORY_HOME`). The planned first-start restart dropped `VICTORY_HOME` and fell back to `%LOCALAPPDATA%\victory`. | Fixed: `victory play` keeps `VICTORY_HOME` across its restarts. |

## Rendering

Found on `9d6300fa` unless the row says otherwise. The offline counts (geometry, compute, mesh) were
measured on the 9d6300fa build; re-checked on df2daafe on 2026-11-28: the same counts.

| Issue | Evidence | What is known | Status |
|---|---|---|---|
| Textures blurry or in the wrong colour until they load | Seen, Logged | The game rewrites textures in memory without telling the runtime (*"a texture picked by index changed after its upload"*, 150–200 times a session). The runtime uploads them again, up to 16 frames later. | Investigating: track writes to those pages |
| Eyes and close-up faces look wrong in cutscenes | Seen | A likely cause: the game addresses about 1.25 million descriptor views and Direct3D 12 caps a heap at 1,000,000, so views past the cap read an empty slot (*"host view heap capped"*). Upstream `df2daafe` changes how placed views are read (PR #58, *"confetti on faces"*); whether that changes this is not yet seen in game. | Investigating: view remapping |
| Water shows dark pixels | Seen (earlier runtime) | On the *Classic* runtime the two water programs use exact paired sums (`VICTORY_WAVE_PAIRS_ONLY`, `VICTORY_WAVE_PAIRS_FP32`), and the dark pixels remained. Upstream `df2daafe` brings an open-water fix (`35c15608`, open water without dark blocks); not yet checked in game on the merged build. | Open |
| Hair and heads look soft | Seen | The game reconstructs 4K from 1440p over several frames, a temporal upscaler that expects 30 fps. At about 14 fps the frames are too far apart. | Needs a higher frame rate |
| Some draws dropped in busy scenes | Logged | *"no free dynamic descriptor slots: a table is not made"*, then that draw is dropped | Open |
| 29 pipelines skipped | Logged | Device method `0x218` (a pipeline creation path) isn't implemented | Open |
| Voxelized lighting missing | Offline | 195 geometry programs don't translate (see [SHADER_TRANSLATION.md](dev/SHADER_TRANSLATION.md)) | Needs a geometry stage |
| Some particles, decal trails and hair shadows missing on NVIDIA | Offline | 22 compute programs need 64-lane waves | Needs 64-lane emulation |
| Fire, decal and spatter on characters | Offline | 71 mesh programs exceed Direct3D 12's 32 KB output limit. NVIDIA's driver accepted such programs in earlier runs. Their pixel shaders read every output, so pruning can't help. | Needs packed 16-bit outputs or split groups |

## Performance

| Issue | Evidence | What is known |
|---|---|---|
| About 14 fps in the world, not 30 | Measured (9d6300fa); indicative (df2daafe) | 14.2 fps median on `9d6300fa` (2026-11-27, same route back to back). On the merged build the readings so far are indicative only, not a same-route benchmark: 17.3–17.9 fps in an interior while pipelines were still being made; 12.8 fps on the F3 panel in the same interior with frame generation off (runtime's submitting thread busy 65.7 ms a frame, GPU idle 35.6 ms waiting for it). The limit is CPU-side: preparing title memory for the GPU and copying back. See [PERFORMANCE.md](PERFORMANCE.md). |
| Stalls when a new effect appears | Seen, Logged | Pipelines are made on first use. INITIALIZE's pipeline step makes the recorded ones ahead of time. On the merged build, see *Draws skipped until their pipelines are made* above. |
| Graphics memory near the card's size | Seen | A texture cache by design. See [GPU_MEMORY.md](GPU_MEMORY.md); use Balanced or Low. On the merged build: 7.7 of 11.0 GB plus 6.0 GB shared on the F3 panel in one interior. |
| System RAM pressure on 32 GB | Logged | *"host memory pressure"* with 5.7 GB physically free; the runtime then limits prefetching. On the merged build, see *High system RAM use* above. |

## Audio and animation

| Issue | Evidence | What is known |
|---|---|---|
| Mouths trail the voice | Seen | A frame reaches the screen several frames after its mouth shape is chosen. **Settings → Sound delay** (`VICTORY_AUDIO_DELAY_MS`) holds the sound back. |

## Features not yet verified on the current runtime

| Feature | State |
|---|---|
| AMD FSR 3.1 (HybridGt's FidelityFX stand-in) | Ported; verified only on the earlier runtime. On `df2daafe` it runs only when `VICTORY_FFX_DIR` names a folder with AMD's PC DLLs. |
| The game's ray tracing | Experimental presets. On the earlier runtime, ray-traced shadows ran at 0.2–3 fps while the structure builder worked. The merged runtime does not build the title's ray-tracing structures (see reflections above). |
| NVIDIA DLSS | Takes the title's upscales on the merged build (seen in the log after the crash fix); its speed is not yet measured by HybridGt |
| Performance preset (worker pools, placed order) | Upstream measured +22–29 % in heavy scenes on an RTX 4070 SUPER; not yet measured by HybridGt |
| Launcher 0.3.0 | Built; 34 unit tests pass; its interface hasn't been seen running |
| Memory modes and frame caps | Built and unit-tested; not measured |
| Ray-traced shadow programs read from the heap (4 programs) | Translate and validate offline; not seen in game |

## Saves

Each runtime keeps its own saves folder. A save from the *Classic* runtime didn't load on the current
one: it was copied to the new folder instead. Keep each preset's saves folder as the scan found it.

The game's save body is encrypted (entropy 7.999 bits per byte). HybridGt does not edit saves.

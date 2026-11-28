# Known issues

[← README](../README.md) · [Status of every fix](STATUS.md) · [Troubleshooting](user/TROUBLESHOOTING.md)

These are the issues on the current runtime (VIctory-NVIDIA `9d6300fa` plus the HybridGt patch), with
what is known about each. Older issues from earlier runtimes are kept in
[KNOWN_FAILURES.md](KNOWN_FAILURES.md) and [ROOT_CAUSE_ANALYSIS.md](ROOT_CAUSE_ANALYSIS.md).

**Evidence:**
- **Seen**: reported by the tester in a game session.
- **Logged**: read in the runtime's logs.
- **Offline**: found by translating or testing without the game.

## Rendering

| Issue | Evidence | What is known | Status |
|---|---|---|---|
| Textures blurry or in the wrong colour until they load | Seen, Logged | The game rewrites textures in memory without telling the runtime (*"a texture picked by index changed after its upload"*, 150–200 times a session). The runtime uploads them again, up to 16 frames later. | Investigating: track writes to those pages |
| Eyes and close-up faces look wrong in cutscenes | Seen | A likely cause: the game addresses about 1.25 million descriptor views and Direct3D 12 caps a heap at 1,000,000, so views past the cap read an empty slot (*"host view heap capped"*). | Investigating: view remapping |
| Water shows dark pixels | Seen (earlier runtime) | On the *Classic* runtime the two water programs use exact paired sums (`VICTORY_WAVE_PAIRS_ONLY`, `VICTORY_WAVE_PAIRS_FP32`), and the dark pixels remained. Not yet checked on the current runtime. | Open |
| Hair and heads look soft | Seen | The game reconstructs 4K from 1440p over several frames, a temporal upscaler that expects 30 fps. At about 14 fps the frames are too far apart. | Needs a higher frame rate |
| Some draws dropped in busy scenes | Logged | *"no free dynamic descriptor slots: a table is not made"*, then that draw is dropped | Open |
| 29 pipelines skipped | Logged | Device method `0x218` (a pipeline creation path) isn't implemented | Open |
| Voxelized lighting missing | Offline | 195 geometry programs don't translate (see [SHADER_TRANSLATION.md](dev/SHADER_TRANSLATION.md)) | Needs a geometry stage |
| Some particles, decal trails and hair shadows missing on NVIDIA | Offline | 22 compute programs need 64-lane waves | Needs 64-lane emulation |
| Fire, decal and spatter on characters | Offline | 71 mesh programs exceed Direct3D 12's 32 KB output limit. NVIDIA's driver accepted such programs in earlier runs. Their pixel shaders read every output, so pruning can't help. | Needs packed 16-bit outputs or split groups |

## Performance

| Issue | Evidence | What is known |
|---|---|---|
| About 14 fps in the world, not 30 | Measured | The runtime's CPU side keeps the console's single memory and the PC's two in step. See [PERFORMANCE.md](PERFORMANCE.md). |
| Stalls when a new effect appears | Seen, Logged | Pipelines are made on first use. INITIALIZE's pipeline step makes the recorded ones ahead of time. |
| Graphics memory near the card's size | Seen | A texture cache by design. See [GPU_MEMORY.md](GPU_MEMORY.md); use Balanced or Low. |
| System RAM pressure on 32 GB | Logged | *"host memory pressure"* with 5.7 GB physically free; the runtime then limits prefetching |

## Audio and animation

| Issue | Evidence | What is known |
|---|---|---|
| Mouths trail the voice | Seen | A frame reaches the screen several frames after its mouth shape is chosen. **Settings → Sound delay** (`VICTORY_AUDIO_DELAY_MS`) holds the sound back. |

## Features not yet verified on the current runtime

| Feature | State |
|---|---|
| AMD FSR 3.1 | Ported; verified only on the earlier runtime |
| The game's ray tracing | Experimental presets. On the earlier runtime, ray-traced shadows ran at 0.2–3 fps while the structure builder worked. |
| NVIDIA DLSS | Supported by the base runtime; not measured by HybridGt |
| Launcher 0.3.0 | Built and unit-tested; its interface hasn't been seen running |
| Memory modes and frame caps | Built and unit-tested; not measured |
| Ray-traced shadow programs read from the heap (4 programs) | Translate and validate offline; not seen in game |

## Saves

Each runtime keeps its own saves folder. A save from the *Classic* runtime didn't load on the current
one: it was copied to the new folder instead. Keep each preset's saves folder as the scan found it.

# HybridGt on VIctory-NVIDIA

Since HybridGt 0.2.0 the runtime is **[VIctory-NVIDIA](https://github.com/pepezin-ai/VIctory-NVIDIA)**
(pepezin-ai / trustno1-coder, GPL-3.0-or-later) at `9d6300fa`, itself on
**[foxhound](https://github.com/readmet3xt-lab/foxhound)** by readmet3xt-lab, with HybridGt's additions on top.
Measured on HybridGt's test PC (RTX 4070 Ti, Ryzen 7 5800X3D, 32 GB), same route, back to back:
**14.2 fps median** (best 18.0) against 5.8 for the earlier runtime, with 5.6 GB of graphics memory
in use at the end against 9.2.

`hybridgt-on-victory-nvidia-9d6300fa.patch`: everything HybridGt adds (`git diff 9d6300fa HEAD`).
`COMMITS.txt`: the commits it contains. Apply on a checkout of VIctory-NVIDIA at `9d6300fa` with
`git apply`.

Carried over so far: AMD FidelityFX / FSR 3.1 for the title's upscale calls (`VICTORY_FFX_DIR`), and
the sound delay (`VICTORY_AUDIO_DELAY_MS`). Not needed: HybridGt's POPS change (this base has its own
rasterizer-ordered views for ordered sections). Next: fxceless's mouth fix (#927, 16-bit values
through typed views), which this base does not have.

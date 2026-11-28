# HybridGt on VIctory-NVIDIA

Since HybridGt 0.2.0 the runtime is **[VIctory-NVIDIA](https://github.com/pepezin-ai/VIctory-NVIDIA)**
(pepezin-ai / trustno1-coder, GPL-3.0-or-later), itself on
**[foxhound](https://github.com/readmet3xt-lab/foxhound)** by readmet3xt-lab, on Marvisak's VIctory,
with HybridGt's additions on top. HybridGt 0.3.0 (the coming release) moves the base from `9d6300fa` to `df2daafe`
(50 upstream commits newer).

This folder holds two patches:

| Patch | Base | Commit list | Status |
|---|---|---|---|
| `hybridgt-on-victory-nvidia-df2daafe.patch` | VIctory-NVIDIA `df2daafe` | `COMMITS-df2daafe.txt` (9 commits, 32 files) | current (HybridGt 0.3.0) |
| `hybridgt-on-victory-nvidia-9d6300fa.patch` | VIctory-NVIDIA `9d6300fa` | `COMMITS.txt` (7 commits, 30 files) | older, kept for reference |

Each patch is everything HybridGt adds to its base (`git diff <base> <last commit in its commit list>`).

## How to apply

Current patch, on `df2daafe`:

```
git clone https://github.com/pepezin-ai/VIctory-NVIDIA
cd VIctory-NVIDIA
git checkout df2daafe
git apply path/to/hybridgt-on-victory-nvidia-df2daafe.patch
```

Older patch, on `9d6300fa`: the same steps with `git checkout 9d6300fa` and
`hybridgt-on-victory-nvidia-9d6300fa.patch`. Each patch applies only on its own base.
`git apply --check` first tells you whether it applies cleanly.

## What the current patch (on df2daafe) adds

The 9 commits in `COMMITS-df2daafe.txt`:

- **Sound delay** (`VICTORY_AUDIO_DELAY_MS`): holds the title's sound back so speech meets the mouths.
  Off by default.
- **AMD FidelityFX stand-in** (stages 1 and 2, plus logging): the title's FSR upscale calls run AMD's
  PC upscaler DLLs (`VICTORY_FFX_DIR`). `VICTORY_FFX_DEBUG=1` turns on AMD's checks.
- **fxceless's mouth fix** (#927 by tippythetaidum): the face network's 16-bit values moved
  bit-exactly through unsigned twin views of float-format views, beside this base's own integer views;
  the table snapshots show the twin, `draw::State::halved_view`.
- **Validator signature fixes**: signature parts as the validator builds them; one interpolation mode a row.
- **Shader fixes and tests**: the level of detail takes no array slice; heap buffer descriptors read for
  their address come from title memory (4 shadow tracers); an offline translate-and-validate test and a
  disassembly test.
- **F1 menu**: shows "HYBRIDGT SETTINGS" and "GITHUB.COM/LOWBACON/HYBRIDGT - BUILT ON VICTORY-NVIDIA"
  (the credit to the fork stays).
- **FidelityFX stand-in only when AMD's files are there**: it is used only when `VICTORY_FFX_DIR` names a
  folder with `amd_fidelityfx_loader_dx12.dll` and `amd_fidelityfx_upscaler_dx12.dll`. Before this fix it
  claimed the title's upscaler without AMD's PC files; the title's upscaler context then failed, upstream's
  DLSS took no upscales, and the game crashed with an integer divide by zero about 50 s after loading a
  save (seen twice on 2026-11-28). After the fix DLSS took the upscales and a session ran about 12 minutes
  past the old crash point and closed normally.
- **`victory play` keeps `VICTORY_HOME`** across its restarts, so F1 menu settings save to the home folder
  (before, the first-start restart fell back to `%LOCALAPPDATA%\victory`).

Not needed: HybridGt's POPS change (this base has its own rasterizer-ordered views for ordered sections).

What upstream brought between `9d6300fa` and `df2daafe` (DLSS through NVIDIA Streamline, AMD FSR 3 frame
generation, the F1 settings menu and F3 stats panel, `victory play`, `victory get-dlss` / `get-fsr` /
`get-mfg`, and rendering fixes) is upstream's work, not part of this patch. See
`git log --oneline 9d6300fa..df2daafe` in a VIctory-NVIDIA checkout.

## The older patch (on 9d6300fa)

The 7 commits in `COMMITS.txt`: the sound delay, the FidelityFX stand-in stages 1 and 2 and logging,
fxceless's mouth fix, the validator signature fixes, and the shader fixes and tests. It does not have the
F1 menu name, the FidelityFX file check or the `VICTORY_HOME` fix. It is kept for reference.
(HybridGt 0.2.0 and 0.2.1 shipped earlier versions of it, with 4 and 6 of these commits.)

Measured on HybridGt's test PC (RTX 4070 Ti, Ryzen 7 5800X3D, 32 GB), same route, back to back
(2026-11-27): **14.2 fps median** on `9d6300fa` with HybridGt 0.2.0's version of this patch (best 18.0) against 5.8 for the earlier
Classic runtime, with 5.6 GB of graphics memory in use at the end against 9.2. The same comparison has not
yet been measured on `df2daafe`.

## Tests

| Build | Result |
|---|---|
| `9d6300fa` with the older patch | 1,031 of 1,032 runtime tests pass. The one failure is an upstream test (victory-gpu `watch::tests::what_is_put_in_beside_the_guards_is_told_at_every_view`) that fails only when tests run in parallel and passes alone; all 408 victory-gpu tests pass with `--test-threads 1`. |
| `df2daafe` with the current patch | 1,078 passed, 0 failed, 52 ignored (2026-11-28) |

Offline shader validation (Microsoft dxil.dll, 32-lane): 25,600 of 25,889 programs validate (98.9 %).
Measured on the 9d6300fa build; re-checked on df2daafe on 2026-11-28: the same counts.

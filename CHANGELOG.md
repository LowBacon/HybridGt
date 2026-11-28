# Changelog

## 2026-11-28 - HybridGt 0.3.0 on the merged runtime

The coming release, v0.3.0, will be HybridGt.exe with the merged runtime
inside (not yet released). Earlier releases: v0.1.0, v0.2.0, v0.2.1.

### Runtime base
- The runtime base moved from VIctory-NVIDIA 9d6300fa to VIctory-NVIDIA
  df2daafe (pepezin-ai / trustno1-coder, GPL-3.0-or-later; on
  readmet3xt-lab's foxhound, on Marvisak's VIctory). df2daafe is 50
  upstream commits newer.
- Upstream brought: open water without dark blocks (35c15608); placed views
  ignore padding words (PR #58, "confetti on faces"); ambient probe purple
  blotch fix; BC7 rotation fix (BCPack); raw buffer views clamped; an
  in-game F1 settings menu and F3 stats panel; DLSS Super Resolution and
  DLSS Frame Generation through NVIDIA Streamline; AMD FSR 3 frame
  generation for cards without DLSS Frame Generation; an MFG Unlocker
  option (someone else's MIT mod, github.com/dashdogy/MFG-Unlocker, for
  RTX 20/30, off by default); PS5 DualSense input; fullscreen, mouse and IME
  window options; optional Bink skip; `victory play` (one-command start);
  and `victory get-dlss`, `get-fsr`, `get-mfg`, `get-d3d12` download
  commands (they fetch from NVIDIA's, AMD's or the mod author's own GitHub
  with SHA-256 checks).
- HybridGt's patch on df2daafe:
  [hybridgt-on-victory-nvidia-df2daafe.patch](patches/victory-nvidia/hybridgt-on-victory-nvidia-df2daafe.patch),
  commit list [COMMITS-df2daafe.txt](patches/victory-nvidia/COMMITS-df2daafe.txt)
  (9 commits, 32 files). The older patch on 9d6300fa
  ([hybridgt-on-victory-nvidia-9d6300fa.patch](patches/victory-nvidia/hybridgt-on-victory-nvidia-9d6300fa.patch),
  [COMMITS.txt](patches/victory-nvidia/COMMITS.txt)) stays for reference.
- The 9 commits carry: audio delay (`VICTORY_AUDIO_DELAY_MS`); FidelityFX
  pseudo-module stages 1-2 and logging (HybridGt's stand-in for the title's
  FSR upscaler with AMD's PC DLLs, `VICTORY_FFX_DIR`); the fxceless #927
  mouth fix (tippythetaidum); validator signature fixes; a level-of-detail
  query fix and heap descriptors read from title memory for 4 shadow
  tracers, plus offline translate-and-validate and disassembly tests; the
  F1 menu shows "HYBRIDGT SETTINGS" and "GITHUB.COM/LOWBACON/HYBRIDGT -
  BUILT ON VICTORY-NVIDIA" (credit kept); the FidelityFX stand-in only when
  `VICTORY_FFX_DIR` names a folder with `amd_fidelityfx_loader_dx12.dll` and
  `amd_fidelityfx_upscaler_dx12.dll`; and `victory play` keeps
  `VICTORY_HOME` across its restarts.

### Crash fixed
- Before the fix, the merged build crashed twice at the same place, about
  50 s after loading a save: integer divide by zero (0xc0000094) in the
  game's own code.
- Cause: HybridGt's FidelityFX module claimed
  `amd_fidelityfx_upscaler_xbox.dll` with no AMD PC files. The title's
  upscaler context failed. Upstream's DLSS interposer saw no context (0
  upscales by DLSS; log: "an upscale dispatch on a context we did not see
  made"). The title then divided by zero.
- Fix: the FidelityFX stand-in is used only when `VICTORY_FFX_DIR` names a
  folder with both AMD PC DLLs.
- After the fix: DLSS took the upscales (324 and counting), the warning did
  not appear, and a play session ran about 12 minutes past the old crash
  point and closed normally.

### F1 menu settings
- Started through `victory play`, F1 settings save to a settings file in the
  home folder (`VICTORY_HOME`). Before the fix, the planned first-start
  restart dropped `VICTORY_HOME` and fell back to `%LOCALAPPDATA%\victory`.
  Fixed: `VICTORY_HOME` is now kept.

### Launcher (crates/hgt-launcher 0.3.0)
- Only HybridGt's own runtimes are started or picked by the scan.
  `deps::is_ours` reads the file for HybridGt markers
  ("GITHUB.COM/LOWBACON/HYBRIDGT" or "VICTORY_AUDIO_DELAY_MS"); the file is
  not run.
- Non-Classic presets start through `victory play` (`presets::play_command`)
  so the F1 menu saves and applies, with `VICTORY_HOME` =
  `%LOCALAPPDATA%\HybridGt\home`. F1-owned settings are passed as play
  options (`--dlss`, `--framegen`, `--fps`, `--series-s`), not `--set`.
- Settings has "Get NVIDIA DLSS", "Get AMD FSR 3" and "Get MFG Unlocker
  (mod)" buttons. They run the runtime's own `get-dlss`, `get-fsr` and
  `get-mfg`. Downloads happen only when pressed. The launcher itself still
  has no network code (enforced by `offline.rs` tests).
- New Performance preset: DLSS Quality and frame generation 2x where
  NVIDIA's files are; worker pools by CPU count (`VICTORY_THIN_THREADS` 12,
  `VICTORY_UNDER_THREADS` 8, `VICTORY_LAND_THREADS` 6 for 16-23 logical
  CPUs; `VICTORY_LAND_THREADS` 8 at 24+; none set below 16);
  `VICTORY_PLACED_ORDER=1`; High process priority. Upstream measured that
  pool and placed-order tuning at +22-29 % in heavy scenes on an RTX 4070
  SUPER. Not yet measured by HybridGt.
- The scan finds NVIDIA Streamline and AMD FidelityFX files in HybridGt's
  home, `%LOCALAPPDATA%\victory`, or a runtime checkout.
- [tools/vendor](tools/vendor) has VIctory-NVIDIA's download scripts
  (`Get-NVIDIA-DLSS.cmd`, `Get-AMD-FSR.cmd`) unchanged, with credit.
- 34 unit tests pass; clippy clean.

### Measurements (test PC: RTX 4070 Ti 12 GB, Ryzen 7 5800X3D, 32 GB, Windows 11)
Figures from 2026-11-28 are indicative. They are not a same-route
back-to-back benchmark.

| What | Result |
|---|---|
| Upstream df2daafe build, tester's own session, DLSS on, frame generation on | world median 13.2 fps; 10th percentile 10.2; 90th percentile 16.6; 441 world seconds |
| Merged HybridGt build, an interior, pipelines still being made (about 290 draws a frame skipped) | 17.3-17.9 fps per-second readings |
| Merged build, same interior, frame generation off (F3 panel) | 12.8 fps; 77.9 ms frame; longest 103 ms |
| Same F3 reading, where the time goes | runtime submitting thread busy 65.7 ms a frame; GPU busy 37.5 ms, idle 35.6 ms waiting for the runtime; copy-back of what the GPU wrote 16.5 ms |
| Same F3 reading, memory | VRAM 7.7 of 11.0 GB plus 6.0 GB shared (system memory); RAM 23.1 GB |
| Earlier, 2026-11-27, same route back to back | 14.2 fps median on 9d6300fa vs 5.8 on the earlier Classic runtime; 5.6 vs 9.2 GB |

- The tester remembered "20 fps" on the upstream build; that was the best
  moments.
- On an RTX 4070 Ti, DLSS Frame Generation runs at 2x. The runtime reports
  "MORE THAN 2X NEEDS AN RTX 50 CARD". With 6x through the MFG Unlocker the
  tester saw 80-100+ fps shown, with cars looking and handling oddly while
  driving. Frame generation shows frames; it does not raise the game's own
  rate, and generated frames guess motion.
- Target remains 30 fps in the world; not reached. The measured limit is
  CPU-side: preparing title memory for the GPU and copying back.

### Shader validation and tests
- Offline validation (Microsoft dxil.dll, 32-lane): 25,600 of 25,889
  programs validate (98.9 %); every vertex and pixel program (14,247); mesh
  8,694/8,765; compute 2,659/2,682; geometry 0/195. Measured on the 9d6300fa
  build; re-checked on df2daafe on 2026-11-28: the same counts.
- Runtime suite on the 9d6300fa build: 1,031 of 1,032 pass. One upstream
  test (`victory-gpu watch::tests::what_is_put_in_beside_the_guards_is_told_at_every_view`)
  fails only when tests run in parallel and passes alone; all 408
  victory-gpu tests pass with `--test-threads 1`. Merged df2daafe suite:
  1,078 passed, 0 failed, 52 ignored (2026-11-28).

### Known issues seen on the merged build (2026-11-28)
- Reflections look wrong. The runtime does not build the title's
  ray-tracing structures, so reflection rays miss and get the title's sky
  light (blue, far brighter than rooms). The runtime's reflection estimate
  (`VICTORY_AMBIENT_SPECULAR`) replaces it on opaque surfaces only (log:
  "surfaces: opaque", about 110 dispatches per 10 s). Glass and other
  transparent surfaces keep the sky light (cyan sparkle).
  `VICTORY_AMBIENT_SPECULAR_ALPHA=1` covers them too, but upstream left it
  off because on water it made dark patches (it reads the depth under the
  water). A test with it on is prepared, not yet reported. Floors keep
  grainy half-resolution reflection noise.
- A white screen sometimes while driving that fades slowly. Suspected: the
  temporal upscaler (DLSS) history after a bad frame. Not confirmed. Test:
  F1, DLSS off.
- Black or late textures: graphics memory full ("the card has no room for
  new ones", about 79,500 times in one session with frame generation on).
  Frame generation takes 1.5-2 GB and the runtime then caps textures at
  80 %.
- Draws are skipped until their pipelines are made: 88,782 pipelines
  recorded. `victory warm` makes them ahead (about 35 minutes with the game
  closed).
- RAM: about 20-23 GB for the game process on a 32 GB PC.
- The game's save body is encrypted (entropy 7.999 bits/byte). HybridGt
  does not edit saves.

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

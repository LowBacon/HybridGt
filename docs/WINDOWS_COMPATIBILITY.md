# Windows compatibility

[← README](../README.md) · [Known issues](KNOWN_ISSUES.md) · [GPU memory](GPU_MEMORY.md)

## What the runtime needs from Windows

| Requirement | Why | Checked by |
|---|---|---|
| Windows 10 version 2004 (build 19041) or later, x64 | The Direct3D 12 features below; DirectStorage-era APIs | Launcher: Diagnostics → Dependencies |
| Direct3D 12 with **mesh shaders** and **Shader Model 6.6** | Console geometry is mesh shaders; translated shaders use SM 6.6 features such as resource heap indexing | The runtime, when it makes its device (`victory-gpu` `host.rs`) |
| `d3d12.dll`, `dxgi.dll` | Present on every supported Windows | Launcher |
| Microsoft Visual C++ 2015–2022 Redistributable, x64 (`vcruntime140.dll`, `msvcp140.dll`) | Runtime libraries the build links against | Launcher |
| XAudio2 | Sound | Present on every supported Windows |
| `dxil.dll` (Windows SDK) | **Only** to validate translated shaders offline. At run time the runtime signs each shader with the container hash Direct3D 12 checks. | Optional |

The launcher checks each of these before anything runs. A missing *required* one blocks the setup,
with the reason and what to install.

## Graphics cards

| Vendor | Status | Notes |
|---|---|---|
| NVIDIA, RTX 40 series | **Verified** on an RTX 4070 Ti 12 GB | All of HybridGt's own measurements |
| NVIDIA, other RTX with mesh shaders | Expected | Same 32-lane wave size as the tested card |
| AMD Radeon RX 6000/7000/9000 | **Expected, not verified** with HybridGt's builds | The base runtime's own lineage was developed on an RX 7900 XTX. AMD cards run 64-lane waves natively, so the 22 compute programs that need them (measured on the 9d6300fa build; re-checked on df2daafe on 2026-11-28: the same counts) could translate there. |
| Intel Arc | Not tested | Needs mesh shaders and SM 6.6 |
| Cards without mesh shaders (for example GTX 10 series) | Not supported | The console's geometry stage has no other path |

8 GB of graphics memory is the minimum the launcher accepts. 12 GB is the tested size.

### 32-lane and 64-lane waves

The console's GPU runs 64 threads per wave. NVIDIA and Intel GPUs run 32, and AMD GPUs can run 64. When
a shader needs all 64 lanes to agree (lane counts, ballots, reading another lane), the translator
rewrites it for 32-lane hardware. Translations for a 32-lane host are checked with
`VICTORY_TEST_WAVE=32`. See [NVIDIA_WAVE64_COMPATIBILITY.md](NVIDIA_WAVE64_COMPATIBILITY.md).

## Upscalers

The runtime base is VIctory-NVIDIA `df2daafe`. DLSS, FSR 3 frame generation and the MFG Unlocker
option come from that base, not from HybridGt.

| Upscaler or frame generation | How it is used | Status |
|---|---|---|
| The game's own temporal upscaler | 1440p → 4K on the Series X profile. When DLSS Super Resolution is on, DLSS takes the game's upscale calls instead. | Part of the game; used when DLSS is off |
| NVIDIA DLSS Super Resolution | Through NVIDIA Streamline. On or off in the in-game **F1** menu. Non-Classic presets (except the FSR 3.1 ones) start through `victory play` with DLSS Quality where NVIDIA's files are found; what is saved in the F1 menu then applies. The *Performance* preset also turns on frame generation 2x. | In use on the merged build: DLSS took the game's upscales (324 and counting in one session) on an RTX 4070 Ti. Image quality and speed against DLSS off **not yet measured** |
| NVIDIA DLSS Frame Generation | Through NVIDIA Streamline. 2x on RTX 40 cards; more than 2x needs an RTX 50 card (the runtime reports `MORE THAN 2X NEEDS AN RTX 50 CARD` on an RTX 4070 Ti). | Seen in game at 2x on an RTX 4070 Ti. It shows extra frames; it does not raise the game's own rate. It takes 1.5–2 GB of graphics memory (see [GPU memory](GPU_MEMORY.md)) |
| MFG Unlocker (someone else's mod) | Optional, **off by default**. The [MFG Unlocker](https://github.com/dashdogy/MFG-Unlocker) mod (MIT) unlocks DLSS Frame Generation on RTX 20/30 cards. It isn't part of this repository. | Seen at 6x on an RTX 4070 Ti (the tester's report): 80–100+ fps shown, with cars looking and handling oddly while driving. Generated frames guess motion. **Not measured** |
| AMD FSR 3 frame generation | For cards without DLSS Frame Generation, or chosen in F1 (FRAME GEN TYPE). 2x at most; the base runtime needs DLSS upscaling on for it. | Supported by the base runtime; **not yet seen in game** by HybridGt |
| AMD FSR 3.1 upscaler (HybridGt's FidelityFX stand-in) | The game's own FidelityFX calls are answered by AMD's signed PC DLLs, loaded from `VICTORY_FFX_DIR`. The stand-in turns on only when that folder holds both `amd_fidelityfx_loader_dx12.dll` and `amd_fidelityfx_upscaler_dx12.dll`; otherwise the game's call goes to the runtime (and to DLSS). The DLLs aren't part of this repository. | **Verified on the earlier runtime** (FSR 3.1.6, 720p → 1080p, RTX 4070 Ti). Ported to the current runtime; **not yet run there** |

Before that check, the stand-in claimed the game's upscaler with no AMD files present. The game's
upscaler context then failed, DLSS saw no context (0 upscales by DLSS), and the game crashed with a
divide by zero (0xc0000094, in the game's own code) about 50 s
after loading a save. With the check, a play session ran about 12 minutes past that point and closed
normally. See [Known issues](KNOWN_ISSUES.md).

### Getting NVIDIA's, AMD's and the mod's files

None of these files ship with HybridGt. The launcher's **Settings** page has three buttons:

| Button | Runs | Fetches from |
|---|---|---|
| **Get NVIDIA DLSS** | the runtime's `victory get-dlss` | NVIDIA's own GitHub |
| **Get AMD FSR 3** | the runtime's `victory get-fsr` | AMD's own GitHub |
| **Get MFG Unlocker (mod)** | the runtime's `victory get-mfg` | the mod author's own GitHub |

The runtime checks each download with SHA-256. A download happens only when you press the button. The
launcher itself has no network code. The base runtime's scripts `Get-NVIDIA-DLSS.cmd` and
`Get-AMD-FSR.cmd` are also in [tools/vendor](../tools/vendor/), unchanged, with credit.

The launcher's scan finds NVIDIA Streamline and AMD FidelityFX files in HybridGt's home folder
(`%LOCALAPPDATA%\HybridGt\home`), in `%LOCALAPPDATA%\victory`, or in a runtime checkout.

## Network

The runtime's network module (`victory-hle`, `modules::net`) lets the game reach **only loopback**.
Connections and sends to anything but 127.0.0.0/8 and ::1 fail, and so do name lookups and WinHTTP
connections to other hosts. Proxies and WPAD are off. No switch turns this off.

The launcher itself has no network code. See [LAUNCHER.md → Offline](LAUNCHER.md#offline).

## Files and folders

| What | Where | Written by |
|---|---|---|
| Game folder | Wherever your copy is | Nobody: read only |
| Shader and pipeline cache | Settings → cache folder (default `%LOCALAPPDATA%\HybridGt\cache`) | Runtime |
| Saves | Settings → saves folder (default `%LOCALAPPDATA%\HybridGt\data`) | Runtime |
| Logs, setup reports | Settings → logs folder (default `%LOCALAPPDATA%\HybridGt\logs`) | Launcher and runtime |
| Launcher settings | `hybridgt-launcher.json` beside `HybridGt.exe`, or in `%LOCALAPPDATA%\HybridGt` | Launcher |
| In-game F1 menu settings (non-Classic presets, started through `victory play`) | Runtime home folder, `VICTORY_HOME` = `%LOCALAPPDATA%\HybridGt\home` | Runtime |

The *Classic* preset keeps its own cache and saves. Saves from one runtime may not load in the other.

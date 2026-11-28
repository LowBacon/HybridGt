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
| NVIDIA, RTX 40 series | **Verified** on an RTX 4070 Ti 12 GB | All measurements in this repository |
| NVIDIA, other RTX with mesh shaders | Expected | Same 32-lane wave size as the tested card |
| AMD Radeon RX 6000/7000/9000 | **Expected, not verified** with HybridGt's builds | The base runtime's own lineage was developed on an RX 7900 XTX. AMD cards run 64-lane waves natively, so the 22 compute programs that need them could translate there. |
| Intel Arc | Not tested | Needs mesh shaders and SM 6.6 |
| Cards without mesh shaders (for example GTX 10 series) | Not supported | The console's geometry stage has no other path |

8 GB of graphics memory is the minimum the launcher accepts. 12 GB is the tested size.

### 32-lane and 64-lane waves

The console's GPU runs 64 threads per wave. NVIDIA and Intel GPUs run 32, and AMD GPUs can run 64. When
a shader needs all 64 lanes to agree (lane counts, ballots, reading another lane), the translator
rewrites it for 32-lane hardware. Translations for a 32-lane host are checked with
`VICTORY_TEST_WAVE=32`. See [NVIDIA_WAVE64_COMPATIBILITY.md](NVIDIA_WAVE64_COMPATIBILITY.md).

## Upscalers

| Upscaler | How it is used | Status |
|---|---|---|
| The game's own temporal upscaler | 1440p → 4K on the Series X profile | Always on, part of the game |
| AMD FSR 3.1 | The game's own FidelityFX calls are answered by AMD's signed PC DLLs, loaded from `VICTORY_FFX_DIR`. They aren't part of this repository. | **Verified on the earlier runtime** (FSR 3.1.6, 720p → 1080p, RTX 4070 Ti). Ported to the current runtime; **not yet run there** |
| NVIDIA DLSS | Through NVIDIA Streamline, from the folder set in **Settings**. The base runtime ships a helper script (`Get-NVIDIA-DLSS.cmd`) that downloads NVIDIA's DLSS and frame generation files, about 70 MB, from NVIDIA's own GitHub. You run it yourself; HybridGt's launcher downloads nothing. | Supported by the base runtime; **not measured** by HybridGt |

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

The *Classic* preset keeps its own cache and saves. Saves from one runtime may not load in the other.

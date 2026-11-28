# Third-party code

HybridGt is GPL-3.0-or-later. This file lists every outside project HybridGt uses or borrows from.

| Project | Licence | How it is used | Changes |
|---|---|---|---|
| [VIctory](https://github.com/Marvisak/VIctory) `victory-formats` at `ffe310eb2098e52169f4c44db5ffdabf43a72622` | GPL-3.0-or-later | A Cargo git dependency, pinned to that commit (BCPack decoder and its shipped format tables). No VIctory source is copied into this repository. | None |
| [victory-gta6](https://github.com/fxceless/victory-gta6) (fork of VIctory) at `9376fd6` | GPL-3.0-or-later | The runtime the game runs in. Not vendored: `patches/victory-gta6/` holds the commits HybridGt adds to it as `git am` patches. | Patches 0001–0022 |
| [ZTRX](https://github.com/Zettrox/ztrx) pull requests #10, #32, #36, #43, #51, #56 | GPL-3.0-or-later | Nine commits by ZTRX contributors (pgm1207 and FrancescoGelosoEterea: #36; S1M40: #32; "Z User": #43; and the authors of #10, #51, #56, kept in each patch), cherry-picked with `-x`; each patch keeps its author and source commit. ZTRX `main` itself is contained in victory-gta6 `a26859f`. | Merged onto the newer base (#36: depth-bounds mesh stream) |
| [VIctory](https://github.com/Marvisak/VIctory) at `ffe310e` | GPL-3.0-or-later | `patches/victory/` (superseded): HybridGt's earlier changes to upstream VIctory, as patches. | Patches 0001–0014 |
| AMD FidelityFX API (`ffx_api.h`) | MIT | The FidelityFX patches implement the API's entry points from the public headers. At run time they load AMD's signed PC DLLs (`amd_fidelityfx_loader_dx12.dll`, `amd_fidelityfx_upscaler_dx12.dll`) from `VICTORY_FFX_DIR`; the DLLs are not in this repository. | — |

## Projects inspected but not used as code

| Project | Licence | Why no code is taken |
|---|---|---|
| [XCL](https://github.com/ivall/xcl) | GPL-2.0-only | GPL-2.0-only cannot be combined with GPL-3.0 code, and HybridGt depends on VIctory (GPL-3.0-or-later). XCL is a reference only: its behaviour and its public issues and PRs are studied, but none of its code is copied. |
| [OpenScarlett](https://github.com/californiaAlmonds/OpenScarlett) | MIT | Compatible. Not used yet. |
| [Leonida](https://github.com/ProjectLeonida/Leonida) | GPL-3.0 (see its LICENSE) | Compatible. Not used yet. |
| WinDurango | LGPL-3.0 | Not inspected. |

Licence note on upstreams (an observation, not legal advice): XCL's `crates/xcl/src/hle/bcpack/victory.rs` says it was "Ported from VIctory's victory-formats/src/bcpack.rs". That is GPL-3.0-or-later code inside a GPL-2.0-only project. HybridGt avoids the question by depending on VIctory directly.

## The runtime since 0.2.0

- **[VIctory-NVIDIA](https://github.com/pepezin-ai/VIctory-NVIDIA)** by pepezin-ai (trustno1-coder),
  GPL-3.0-or-later, at `df2daafe`: the runtime HybridGt.exe carries from 0.3.0, with HybridGt's patch
  (`patches/victory-nvidia/hybridgt-on-victory-nvidia-df2daafe.patch`, commit list
  `COMMITS-df2daafe.txt`: 9 commits). 0.2.x carried it at `9d6300fa`; that patch
  (`hybridgt-on-victory-nvidia-9d6300fa.patch`, `COMMITS.txt`) stays for reference. Its README credits
  the many projects it was built from. HybridGt's F1 menu keeps the credit:
  "GITHUB.COM/LOWBACON/HYBRIDGT - BUILT ON VICTORY-NVIDIA".
- **[foxhound](https://github.com/readmet3xt-lab/foxhound)** by readmet3xt-lab, GPL-3.0-or-later: the
  base VIctory-NVIDIA stands on (Series X at 1440p, the copy machinery that runs about twice as fast,
  DLSS and frame generation through NVIDIA Streamline, loaded at run time from a folder you supply).
- **[VIctory](https://github.com/Marvisak/VIctory)** by Marvisak and contributors, under all of them.
- NVIDIA Streamline, DLSS and Reflex, and AMD FidelityFX, are their makers'; nothing of them is in this
  repository or in HybridGt.exe: they are loaded from folders you supply. The runtime's `victory
  get-dlss` and `get-fsr` commands (also the launcher's "Get NVIDIA DLSS" and "Get AMD FSR 3" buttons)
  fetch them from NVIDIA's and AMD's own GitHub only when you ask (AMD's files are checked against
  SHA-256; NVIDIA's are checked for the sizes listed in NVIDIA's archive).
- **[MFG Unlocker](https://github.com/dashdogy/MFG-Unlocker)** by dashdogy, MIT: someone else's mod
  for RTX 20/30 cards, offered by VIctory-NVIDIA as an option that is off by default. It is not in this
  repository or in HybridGt.exe. The runtime fetches it from the author's own GitHub release, checked
  against the SHA-256 the author published, only when you ask: with `victory get-mfg` (the launcher's
  "Get MFG Unlocker (mod)" button), or at the next start after you turn on F1 > MFG UNLOCKER. The
  launcher itself has no network code.
- **`tools/vendor/`**: VIctory-NVIDIA's download scripts at `df2daafe`, GPL-3.0-or-later, with credit
  (see [tools/vendor/README.md](tools/vendor/README.md)). `get-nvidia-sdks.ps1` and `get-amd-fsr.ps1`
  are copied unchanged. The `Get-NVIDIA-DLSS.cmd` and `Get-AMD-FSR.cmd` wrappers are changed: new
  comment lines, the script beside them instead of in `scripts\`, and a `-To` folder under
  `%LOCALAPPDATA%\victory\`. They fetch NVIDIA's and AMD's files only when you run them.

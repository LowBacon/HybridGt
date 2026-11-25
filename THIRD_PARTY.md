# Third-party code

HybridGt is GPL-3.0-or-later. This file lists every outside project HybridGt uses or borrows from.

| Project | Licence | How it is used | Changes |
|---|---|---|---|
| [VIctory](https://github.com/Marvisak/VIctory) `victory-formats` at `ffe310eb2098e52169f4c44db5ffdabf43a72622` | GPL-3.0-or-later | A Cargo git dependency, pinned to that commit (BCPack decoder and its shipped format tables). No VIctory source is copied into this repository. | None |
| [victory-gta6](https://github.com/fxceless/victory-gta6) (fork of VIctory) at `a26859f` | GPL-3.0-or-later | The runtime the game runs in. Not vendored: `patches/victory-gta6/` holds the commits HybridGt adds to it as `git am` patches. | Patches 0001–0011 |
| [ZTRX](https://github.com/Zettrox/ztrx) pull requests #32, #36, #43 | GPL-3.0-or-later | Six commits by ZTRX contributors (pgm1207 and FrancescoGelosoEterea: #36; S1M40: #32; "Z User": #43), cherry-picked with `-x`; each patch keeps its author and source commit. ZTRX `main` itself is contained in victory-gta6 `a26859f`. | Merged onto the newer base (#36: depth-bounds mesh stream) |
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

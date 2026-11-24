# Third-party code

HybridGt is GPL-3.0-or-later. This file lists every outside project HybridGt uses or borrows from.

| Project | Licence | How it is used | Changes |
|---|---|---|---|
| [VIctory](https://github.com/Marvisak/VIctory) `victory-formats` at `ffe310eb2098e52169f4c44db5ffdabf43a72622` | GPL-3.0-or-later | A Cargo git dependency, pinned to that commit (BCPack decoder and its shipped format tables). No VIctory source is copied into this repository. | None |

## Projects inspected but not used as code

| Project | Licence | Why no code is taken |
|---|---|---|
| [XCL](https://github.com/ivall/xcl) | GPL-2.0-only | GPL-2.0-only cannot be combined with GPL-3.0 code, and HybridGt depends on VIctory (GPL-3.0-or-later). XCL is a reference only: its behaviour and its public issues and PRs are studied, but none of its code is copied. |
| [OpenScarlett](https://github.com/californiaAlmonds/OpenScarlett) | MIT | Compatible. Not used yet. |
| [Leonida](https://github.com/ProjectLeonida/Leonida) | GPL-3.0 (see its LICENSE) | Compatible. Not used yet. |
| WinDurango | LGPL-3.0 | Not inspected. |

Licence note on upstreams (an observation, not legal advice): XCL's `crates/xcl/src/hle/bcpack/victory.rs` says it was "Ported from VIctory's victory-formats/src/bcpack.rs". That is GPL-3.0-or-later code inside a GPL-2.0-only project. HybridGt avoids the question by depending on VIctory directly.

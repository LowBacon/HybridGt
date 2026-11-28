# Contributing to HybridGt

Thank you for helping. HybridGt is a compatibility research project: correctness and honest
measurements come first.

## Ways to help

| Contribution | Where to start |
|---|---|
| **Bug reports** | [Troubleshooting → Reporting a problem](docs/user/TROUBLESHOOTING.md#reporting-a-problem) |
| **Benchmarks**, especially on AMD and other NVIDIA cards | [PERFORMANCE.md → Reporting a benchmark](docs/PERFORMANCE.md#reporting-a-benchmark) |
| **Shader translation** | [SHADER_TRANSLATION.md](docs/dev/SHADER_TRANSLATION.md): the remaining programs and what each needs |
| **Runtime fixes** | [KNOWN_ISSUES.md](docs/KNOWN_ISSUES.md), [ROADMAP.md](docs/ROADMAP.md) |
| **Launcher** | [`crates/hgt-launcher`](crates/hgt-launcher), [LAUNCHER.md](docs/LAUNCHER.md) |
| **Documentation** | Anything that was unclear to you is worth a fix |

## Development environment

See [DEVELOPMENT.md](docs/dev/DEVELOPMENT.md): the Rust MSVC toolchain, the Windows SDK, building the
launcher and the runtime, and running the tests.

## Code organization

| Path | Contents |
|---|---|
| `crates/hgt-launcher` | The launcher (one module per concern; see [ARCHITECTURE.md](docs/dev/ARCHITECTURE.md#the-launcher)) |
| `crates/hgt-gfxdiag` | D3D12 diagnostic harness, WARP by default |
| `crates/hgt-texture` | Texture data paths |
| `patches/victory-nvidia` | The runtime patch on VIctory-NVIDIA `9d6300fa` |
| `tools` | Log statistics, decoders, status images |
| `docs` | User docs (`docs/user`), developer docs (`docs/dev`), status and measurements |

Runtime changes are made in a VIctory-NVIDIA checkout and exported to `patches/victory-nvidia` (see
[Updating the runtime patch](docs/dev/DEVELOPMENT.md#updating-the-runtime-patch)).

## Pull requests

- **One change per pull request**, with a description of what it fixes and how you checked it.
- **Tests.** `cargo test --release -p hgt-launcher` passes for launcher changes. Runtime changes pass the
  runtime's workspace tests, and for shader changes the offline translate-and-validate test shows no
  program lost.
- **Measurements.** A performance change includes before and after on the same route, same settings.
  A rendering change says whether it was seen in the game, or only offline.
- **Status.** Update [STATUS.md](docs/STATUS.md) or [KNOWN_ISSUES.md](docs/KNOWN_ISSUES.md) when a
  change moves an item.
- **Credit.** Work taken from another project keeps its author, is cherry-picked with `-x` where
  possible, and is added to [THIRD_PARTY.md](THIRD_PARTY.md) with its licence. The licence must be
  compatible with GPL-3.0-or-later. GPL-2.0-only code can't be used.

## Coding standards

- Rust 2024, `cargo fmt`, `cargo clippy` without new warnings.
- `unsafe` blocks carry a `// SAFETY:` comment saying why they hold.
- Comments explain why, in plain language, and name the run or test that showed it.
- Runtime switches are `VICTORY_*` environment variables, documented where they are read, with their
  default stated.
- No new network dependencies in the launcher (the build fails if one is added).

## What must never be committed

Game files, or anything derived from them: shaders, shader dumps, disassembly, textures, audio,
screenshots of the game's art, pipeline caches, saves, or logs that contain them. The runtime refuses
to write game-derived output inside its own source tree. Please keep the same rule by hand in this one.

## Licence

By contributing you agree that your contribution is licensed under the GNU General Public License,
version 3 or later, the licence of this repository.

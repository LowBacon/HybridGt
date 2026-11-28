# Mission report: launcher redesign, offline infrastructure, graphics investigation

Date: 2026-11-27. Done offline the whole time. No game, runtime or launcher executable was
started; nothing was downloaded; nothing was pushed. Builds ran at low priority with
`cargo --offline -j 4`.

## 1. Summary

The launcher (`HybridGt.exe`, 0.3.0) was rebuilt:

- **Pages:** five pages (Home, Library, Performance, Settings, Diagnostics).
- **Setup:** a one-click environment setup whose progress counts real work.
- **System and requirements:** a reader for this PC and a requirements check.
- **Game library:** reads the game's own name, version and art from the player's folder, and counts playtime.
- **Background processes:** a process runner with progress parsing and a log summarizer.
- **Offline policy:** enforced by tests.
- **Identity:** a Windows icon and version resource, made offline by `build.rs`.

It compiles without warnings, clippy is clean, and 30 of 30 unit tests pass.

The graphics investigation was done from the logs of the last measured sessions. It found concrete leads, ranked in section 6. None of them is fixed in the runtime yet: each needs an in-game test session to verify.

## 2. Files

**New** in `crates/hgt-launcher`:

| File | Role |
|---|---|
| `src/config.rs` | Settings file, migration from launchers before 0.2, problems check (no machine paths built in) |
| `src/system.rs` | Processor, graphics cards and memory, RAM, Windows build, free space, folder sizes (registry and Win32, read-only) |
| `src/deps.rs` | Requirements: Windows build, Direct3D 12/DXGI, VC++ runtime, card memory, RAM, game, runtime, free space, FFX, Streamline, cargo |
| `src/library.rs` | The game from `MicrosoftGame.Config`, its splash and logo decoded from the player's folder, playtime |
| `src/jobs.rs` | Runs the runtime's `shaders`/`warm`/`run` and cargo hidden, logs, parses progress and the frame line |
| `src/setup.rs` | The one-click setup: 8 steps, real progress, report |
| `src/diagnostics.rs` | Newest log, errors and warnings grouped and counted, how the run ended |
| `src/offline.rs` | The offline policy and the tests that enforce it |
| `src/pages.rs` | The five pages |
| `src/mark.rs` | HybridGt's mark as pixels (window icon and exe icon) |
| `docs/LAUNCHER.md` | Launcher documentation |
| `docs/MISSION_REPORT.md` | This report |

**Changed**:

| File | Change |
|---|---|
| `crates/hgt-launcher/src/main.rs` | App shell: navigation, processes, scanning, setup, playtime |
| `crates/hgt-launcher/src/style.rs` | Brand, hero banner, chips, progress bars, status rows |
| `crates/hgt-launcher/src/presets.rs` | `environment()`: every switch a run gets, from preset and settings |
| `crates/hgt-launcher/build.rs` | `.ico` and VERSIONINFO via the Windows SDK's `rc.exe`, with a graceful fallback |
| `crates/hgt-launcher/Cargo.toml` | 0.3.0; `serde_json`, `image` (png only), `windows-sys` features |
| `Cargo.lock` | Resolved offline from the local registry |
| `README.md` | Launcher section |

`scan.rs` is unchanged in this mission. Backups of the 0.2.1 `main.rs` and `Cargo.toml` are in `HybridGt-work/review/`.

## 3. Features

- **Home:** the game's banner art (from the player's folder) with name, version and publisher. Status chips (installed, requirements, preset), PLAY/STOP, presets, last played, time played, version, runtime state, and INITIALIZE with each step's status.
- **Library:** logo, location, version, size, playtime and sessions. Installation, shader cache, pipeline cache, assets and compatibility status. Play, prepare, settings.
- **Performance:** this PC (processor and threads, each card with its memory, RAM total and free with a usage bar, Windows). The graphics configuration (preset, internal resolution, upscaler, ray tracing, memory mode, frame cap). Direct3D 12, shader and pipeline cache state and sizes.
- **Settings:**
  - game folder and scan;
  - memory: Auto, Balanced, Low;
  - frame cap: game's own, 30, 60;
  - DLSS, estimated light, Classic switches, sound delay;
  - all ten folders;
  - build from source (`cargo --offline`);
  - log level, animations, offline policy;
  - run setup again, reset preferences (keeps folders), clear shader cache (two clicks).
- **Diagnostics:** requirements with the fix for each, settings problems, the last setup, the running session's fps, frame time and draws, the last log summarized (errors, warnings, top messages, how it ended), and the log itself.
- **Fps placement:** fps appears only on Diagnostics, never in the exe name, window title or library.
- **Setup progress:** progress is the settled steps plus the running step's real count, held under 100 % while anything runs. A missing requirement blocks the later steps with a reason; nothing is ever marked done that did not happen.

## 4. Build results

| Check | Result |
|---|---|
| `cargo build --release --offline -p hgt-launcher` | ok, 0 warnings |
| `cargo clippy --release --offline -p hgt-launcher --all-targets` | ok, 0 warnings |
| Exe metadata, read from the file (not run) | ProductName HybridGt, FileDescription "HybridGt launcher", FileVersion 0.3.0, LegalCopyright GPL-3.0-or-later; icon present |
| Size | 5.4 MB, without an embedded runtime. A release build with `HGT_RUNTIME_EXE` was not made this mission |

## 5. Offline test results

`cargo test --release --offline -p hgt-launcher`: **30 passed, 0 failed**.

The tests cover:

- **Config:** migration, round trip, broken file, problems.
- **Presets:** switches, memory and frame cap, DLSS needing its files, folders, ray tracing.
- **Setup:**
  - progress reaches 1.0 only when all steps settle;
  - a blocked step blocks those after it;
  - a blank PC stops at the dependencies with no process started.
- **Parsers:** shader, pipeline and frame-line parsing; log grouping; newest log.
- **Library and this PC:** game config attributes, PNG decode and scale, playtime; reading this PC; Direct3D 12 present; vendor names; folder counts; scan of launcher scripts.
- **Mark and offline:** the mark's shape, and the two offline tests.
  - The offline test caught one real hit during this mission: the theme set a hyperlink colour. It was removed rather than the test weakened.

The runtime (`victory-nvidia`) was not rebuilt or retested in this mission.

## 6. Rendering and performance: offline findings

These come from three logs of the last sessions:

- the launcher's `game-1795787376.log` (510 frame samples: median 10.1 fps, p10 5.9);
- the launcher's `game-1795790114.log` (54 samples: median 15.2 fps);
- `x6-nvidia.log` (76 samples: median 14.6 fps, p10 3.4).

The warnings are logged once or a few times each, so the counts below are lower bounds. Ranked by how likely each is to explain what was reported (odd eyes, rough or wrong-coloured textures while loading, dark water pixels, low fps):

1. **View heap cap.**
   - Log: "host view heap capped at 1000000; views past it read a null slot count=1245184" (`victory-gpu/src/descriptor/host.rs:1912`, marked "shortcut" in the source).
   - About 245,000 of the title's texture and buffer views read an empty slot. 1,000,000 is Direct3D 12's limit for one shader-visible heap.
   - Draws that index past it sample nothing. That fits black or odd eyes and missing detail.
   - The fix is remapping (compacting the title's view indices into the host heap), not raising the cap. It is a runtime change that must be checked in game.
2. **Textures changed after upload.**
   - 149–200 times per session: "a texture picked by index changed after its upload with nothing to tell of it: uploaded again" (`draw.rs:3307`).
   - The title rewrites texture memory the runtime cannot see. The runtime then uploads it again, after up to 16 frames of stale content.
   - That fits textures in the wrong colour or resolution until they "pop".
   - Tracking writes to those pages (or re-hashing on a cheaper schedule) would shorten the window.
3. **Descriptor slots running out.**
   - "no free dynamic descriptor slots: a table is not made n=258", followed by "a draw whose descriptor table found no free slots is dropped".
   - Dropped draws mean missing geometry or effects in busy scenes.
4. **Memory pressure.**
   - "the card has no room for more; they wait", and host memory pressure with 5.7 GB physical free and 4.1 GB commit available.
   - Textures queue on a 12 GB card; the launcher's Balanced and Low memory modes cap this.
5. **Unimplemented pipeline creation.**
   - 29 creation calls by an unimplemented method (slot 0x218): "the title goes without this pipeline".
   - Whatever those pipelines draw is missing.
6. **Smaller items.**
   - Two compute shaders do not translate, and their draws are dropped.
   - The indirect light has no stand-in for the Miss role (ray tracing).
   - One copied-back texture cannot be retiled.

From earlier offline validation (unchanged): 99.6 % of 25,694 programs validate. Remaining:

| Programs | Problem |
|---|---|
| 195 | Geometry shaders refused |
| 71 | Mesh programs over the output limit |
| 22 | Wave64 compute programs untranslated |
| 5 | Shadow tracers |
| 3 | Out-of-bound coordinates |

Water's dark pixels are still unexplained offline. The paired-wave form is applied there (seen earlier). Leads 1 and 2 are candidates.

## 7. What needs in-game verification

None of this was seen running in this mission.

- **The launcher's interface as a whole:** layout, art loading, the hero banner, hover, every button.
- **INITIALIZE end to end:** shader preparation through the runtime, pipeline warm, the report.
- **PLAY and STOP, plus play tracking:** playtime counting and the live frame line on Diagnostics.
- **The exe icon in Explorer and the taskbar.**
- **Each runtime finding above before and after any fix:** eyes and close-ups in cutscenes, textures while loading, water, busy streets, ray-traced presets.
- **Any frame rate claim.** The last measured figure stands: 14.2 fps median on the Fast preset (0.2.0). No fps improvement is claimed here.

## 8. Status by category

| Item | Implemented | Compiled | Unit-tested | Offline-validated | Needs in-game test | Blocked |
|---|---|---|---|---|---|---|
| Five pages | ✔ | ✔ | — | — | ✔ | |
| One-click setup and real progress | ✔ | ✔ | ✔ | ✔ (blank-PC path) | ✔ (full run) | |
| Requirements check | ✔ | ✔ | ✔ | ✔ (this PC) | | |
| Game library, art, playtime | ✔ | ✔ | ✔ | ✔ (decode) | ✔ | |
| Offline policy | ✔ | ✔ | ✔ | ✔ | | |
| Exe icon and version info | ✔ | ✔ | — | ✔ (read from file) | ✔ (look) | |
| fps only in Diagnostics | ✔ | ✔ | — | ✔ (code review) | | |
| Rendering findings 1–6 | analysis only | | | ✔ (logs) | ✔ | runtime fixes need a test session |
| `GTA6.exe` rename | not done, by design | | | | | the game folder must not be modified |
| Official logos in the launcher | not shipped, by design | | | | | only the player's own files, at run time |
| GitHub push and release 0.3.0 | | | | | | awaiting your approval (network) |

## 9. Safe testing, when you choose to

1. Close the game you are playing first. The launcher and the game must not compete for the card.
2. Start `target\release\HybridGt.exe` from the repo, or copy it anywhere: it keeps its settings beside itself, or in `%LOCALAPPDATA%\HybridGt`.
3. Open **Diagnostics** first. Every requirement should read ok or optional.
4. On **Home**, press **INITIALIZE**:
   - Shader cache takes minutes the first time; it is already prepared if you used 0.2.
   - Pipeline cache needs one session with the pipeline book; otherwise it says so and stays blocked, which is expected.
5. Then **PLAY** with the Fast preset. Check eyes in a cutscene, textures while driving, and water.
6. Afterwards, open the **Diagnostics** log summary. If "host view heap capped" is there, finding 1 is confirmed for your session.

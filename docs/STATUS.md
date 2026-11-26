# Status of every fix

[← Back to the README](../README.md) · Every test run: [PERFORMANCE_OPTIMIZATION.md](PERFORMANCE_OPTIMIZATION.md)

Every problem moves through five steps. A fix counts as done only once it has been **seen in the
game**.

`①` found · `②` cause known · `③` fix written · `④` seen in game · `⑤` done

| Problem | Progress | Where it stands |
|---|---|---|
| Characters' bodies missing | `●●●●●` ⑤ | Whole characters in the prologue. Fixed by the fxceless merge with the 0x1c8 pipelines off |
| Intro "triangles" (rectangle lists) | `●●●●●` ⑤ | `VICTORY_NATIVE_RECTS=1` |
| Late textures | `●●●●○` ④ | "No room for textures" 60 → 20 a run (`VICTORY_CARD_WINDOWS_MB=1024`, `TEXTURE_ROOM_PERCENT=96`). Still some late |
| AMD FSR 3.1 upscaling | `●●●●○` ④ | **FSR 3.1.6** runs on the RTX 4070 Ti (provider checked on the card), through the game's Series S path (`--console series-s`), 720p → 1080p. No fps gain; one black face and one white frame seen. The game's creation flags are now logged by name, AMD's messages go to our log, and `VICTORY_FFX_DEBUG=1` turns on its input checks for the next Series S run |
| Series S mode | `●●●●○` ④ | Boots and plays the prologue |
| Black interiors (traced indirect light) | `●●●●○` ④ | `VICTORY_AMBIENT=1` stand-in lights them (costs about 1 fps). On in `play-fast` |
| Boat sinks ("Cal fell off the boat") | `●●●○○` ③ | fxceless's `VICTORY_TEXTURE_BACK=1` makes boats float; not yet checked at that mission in our runs |
| Water light volume shaders (above and under water) | `●●●○○` ③ | A mask parked in two halves across a loop was not followed: fixed in the translator (with a test), both now translate and validate. The card still refuses their pipelines: they need 64-lane waves |
| 27 compute programs that need 64-lane waves | `●●●●○` ④ | 21 run as two 32-lane halves with branches taken by the whole wave (`VICTORY_WAVE_WIDE=last`, on in `play-fast`): in game, refused pipelines 5 -> 2, no GPU loss. The other 6 (heap defrag, triangle collection, 2 decal-trail builders: thread groups of several console waves) now get pipelines on the card with branches decided by the whole group (`VICTORY_GROUP_WIDE=1`, off until a game run); 2 of them exceed Direct3D's 32 KB of group memory by 132 bytes (NVIDIA accepts it) |
| Ray tracing (real traced light) | `●●●○○` ③ | Builder translates whole (52/52) and runs, but our build crashes 11 s later; VIctory-7900XTX's build survives it but loses the GPU after 3.5 min |
| 0x1c8 graphics pipelines (console path) | `●●●○○` ③ | Built, but they hide bodies (stencil not decoded). Off |
| Walls and surfaces missing or low quality | `●●●○○` ③ | The card is at its memory line (10.5 of 10.75 GB): 18,000-25,000 textures wait for room, so some surfaces are drawn untextured or at low detail. Textures are never let go while the game's texture table points at them. `VICTORY_CARD_WINDOWS_MB=512` (on): textures waiting 12,000 -> 10,200, 800 MB more free on the card, fps the same. Next: free idle textures safely. Textures on screen first (`VICTORY_TEXTURE_MARKS=1`, built and tested on the card, off): shaders mark the texture slots they sample, a copy queue reads the map back every half second in 2.4 ms, and marked textures are uploaded first. Its first game try read the map through the bus and stalled loading; fixed, not yet run in game again |
| Lucia's clothes don't change | `●●●○○` ③ | Characters' skins and clothes are "baked" by the game from source textures; the bakes run while those sources still wait for upload, so a bake can keep the old look. `VICTORY_BAKE_SOURCES=full` (on): each bake's sources are uploaded first (40 textures, 48 MB, in the test run; none skipped). Not yet seen in game at an outfit change |
| Black faces in cutscenes | `●●●○○` ③ | One character's face was black in a prologue cutscene. In run x-pops (`VICTORY_POPS=rov` alone) every face and eye in the shots is right and the picture never went white; the same scene is checked again before it is turned on |
| Blur on hair and heads | `●●○○○` ② | The game's own temporal upscaler (1440p to 4K) blends frames meant to come 30 a second; at 6-7 fps hair and faces smear. Needs frame rate |
| Lag spikes | `●●○○○` ② | Worst frames 1.3-2.8 s, when textures and pipelines arrive |
| Ray-traced shadow programs (5) | `●●●○○` ③ | All 5 translate, validate and get pipelines: the address of a view picked at a computed index comes from a host table (after VIctory-7900XTX 027e367a, reimplemented), and mask operations on a joined number read as data (VIctory-7900XTX d1f883ec). Not yet seen in game |
| Ped tint cut-out pixel shaders (10) | `●●●○○` ③ | They order overlapping pixels themselves (POPS). `VICTORY_POPS=rov` (off until seen in game) translates them with rasterizer-ordered views, so the card keeps that order: all 10 validate and get pipelines on the RTX 4070 Ti. `VICTORY_POPS=1` (no order) made characters' eyes glow |
| Mouth animation | `●●●●○` ④ | **Mouths move** since the merge of fxceless 52ac4ec0 (16-bit face-network indices moved through typed views bit-exactly, pull request #927 by tippythetaidum; SDWA sign extension): seen in game. At 5-7 fps a frame reaches the screen 300-450 ms after its mouth shape was picked, so mouths trail the voice. `VICTORY_AUDIO_DELAY_MS=<ms>` (new, off) delays the sound to match; tested on XAudio2, not yet heard in game. The frame rate is the real fix |

## Frame rate

World median on the RTX 4070 Ti, each step measured:
3.0 → 3.7 → 4.8 → 5.8 → 6.7 → **7.7 fps** (6.7 with interiors lit). Menus run at the game's own
30 fps. The frame is CPU-bound: preparing title memory for the card, not the card itself.

More: [KNOWN_FAILURES.md](KNOWN_FAILURES.md) · [MASTER_STATUS.md](MASTER_STATUS.md) ·
[ROOT_CAUSE_ANALYSIS.md](ROOT_CAUSE_ANALYSIS.md)

## Tests

Every test of the runtime's workspace passes (673, 0 failed; the GPU tests on the RTX 4070 Ti), and so does the
repository's asset policy (`cargo xtask policy`). A full translation of the title's 26,084 shader programs
with validation and pipeline creation on the card: all translate; compute 2,682 of 2,682 pipelines; no
program lost a pipeline or its validation against the run before.

## Other builds measured on this PC (RTX 4070 Ti)

| Build | Result |
|---|---|
| VIctory-7900XTX main `2de99801`, renderer ports 1-7 on, Series X | 2.0-2.5 fps and **no 3D world** (only 2D text): its ports are built for AMD cards |
| cool_fox `f4b48ad` (its own launcher, Series S) | 1.9-2.7 fps |
| HybridGt `play-fast` (this repository) | 5.0-6.9 fps median by scene, whole picture |

AnyXboxX (LooperSalty) reports 15-20 fps on Linux with Proton, an RX 9070 XT and a patched RADV; not
measured on Windows. Its graphics are a VIctory copy (`third_party/victory`).

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
| AMD FSR 3.1 upscaling | `●●●●○` ④ | **FSR 3.1.6** runs on the RTX 4070 Ti (provider checked on the card), through the game's Series S path (`--console series-s`), 720p → 1080p. No fps gain; one black face and one white frame seen |
| Series S mode | `●●●●○` ④ | Boots and plays the prologue |
| Black interiors (traced indirect light) | `●●●●○` ④ | `VICTORY_AMBIENT=1` stand-in lights them (costs about 1 fps). On in `play-fast` |
| Boat sinks ("Cal fell off the boat") | `●●●○○` ③ | fxceless's `VICTORY_TEXTURE_BACK=1` makes boats float; not yet checked at that mission in our runs |
| Water light volume shaders (above and under water) | `●●●○○` ③ | A mask parked in two halves across a loop was not followed: fixed in the translator (with a test), both now translate and validate. The card still refuses their pipelines: they need 64-lane waves |
| 27 compute programs that need 64-lane waves | `●●●●○` ④ | 21 now run as two 32-lane halves with branches taken by the whole wave (`VICTORY_WAVE_WIDE=last`, on in `play-fast`): volumetric light (5), particle emitters (7), `CS_Visibility`, hair shadows, most decal and liquid trails. In game: refused pipelines 5 -> 2, no GPU loss, fps the same. Left: 6 with thread groups of several console waves (heap defrag, triangle collection, 2 decal-trail builders) |
| Ray tracing (real traced light) | `●●●○○` ③ | Builder translates whole (52/52) and runs, but our build crashes 11 s later; VIctory-7900XTX's build survives it but loses the GPU after 3.5 min |
| 0x1c8 graphics pipelines (console path) | `●●●○○` ③ | Built, but they hide bodies (stencil not decoded). Off |
| Walls and surfaces missing or low quality | `●●●○○` ③ | The card is at its memory line (10.5 of 10.75 GB): 18,000-25,000 textures wait for room, so some surfaces are drawn untextured or at low detail. Textures are never let go while the game's texture table points at them. `VICTORY_CARD_WINDOWS_MB=512` (on): textures waiting 12,000 -> 10,200, 800 MB more free on the card, fps the same. Next: free idle textures safely |
| Lucia's clothes don't change | `●●●○○` ③ | Characters' skins and clothes are "baked" by the game from source textures; the bakes run while those sources still wait for upload, so a bake can keep the old look. `VICTORY_BAKE_SOURCES=full` (on): each bake's sources are uploaded first (40 textures, 48 MB, in the test run; none skipped). Not yet seen in game at an outfit change |
| Lag spikes | `●●○○○` ② | Worst frames 1.3-2.8 s, when textures and pipelines arrive |
| Ray-traced shadow programs (5) | `●●○○○` ② | A descriptor read as plain data |
| Ped tint cut-out pixel shaders (10) | `●●○○○` ② | A hardware input NVIDIA does not have (pixel ordering) |
| Mouth animation | `●●○○○` ② | The head's GPU programs (`ped_standard_head_cs`) all translate; four run in game (sticky lips, blend-shape mappings, the network's output extract, normals). No GPU program evaluates the face network: the game's CPU code does, and the extract's result stayed the same over ~1,500 frames of talking. So the cause is on the CPU side: the face animation is not advancing (speech timing or facial animation data). Next: trace what the CPU writes before the extract |

## Frame rate

World median on the RTX 4070 Ti, each step measured:
3.0 → 3.7 → 4.8 → 5.8 → 6.7 → **7.7 fps** (6.7 with interiors lit). Menus run at the game's own
30 fps. The frame is CPU-bound: preparing title memory for the card, not the card itself.

More: [KNOWN_FAILURES.md](KNOWN_FAILURES.md) · [MASTER_STATUS.md](MASTER_STATUS.md) ·
[ROOT_CAUSE_ANALYSIS.md](ROOT_CAUSE_ANALYSIS.md)

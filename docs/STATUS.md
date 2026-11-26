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
| 27 compute programs that need 64-lane waves | `●●○○○` ② | Translate and validate, refused by NVIDIA (32 lanes): volumetric light (5), particle emitters (7), decal trails (7), hair shadow masks (2), `CS_Visibility`, triangle collection (2), heap defrag (2), liquid trails. In the last game run 5 of them came up and were dropped |
| Ray tracing (real traced light) | `●●●○○` ③ | Builder translates whole (52/52) and runs, but our build crashes 11 s later; VIctory-7900XTX's build survives it but loses the GPU after 3.5 min |
| 0x1c8 graphics pipelines (console path) | `●●●○○` ③ | Built, but they hide bodies (stencil not decoded). Off |
| Lag spikes | `●●○○○` ② | Worst frames 1.3-2.8 s, when textures and pipelines arrive |
| Ray-traced shadow programs (5) | `●●○○○` ② | A descriptor read as plain data |
| Ped tint cut-out pixel shaders (10) | `●●○○○` ② | A hardware input NVIDIA does not have (pixel ordering) |
| Mouth animation | `●○○○○` ① | Mouths don't move when characters talk; cause not found |

## Frame rate

World median on the RTX 4070 Ti, each step measured:
3.0 → 3.7 → 4.8 → 5.8 → 6.7 → **7.7 fps** (6.7 with interiors lit). Menus run at the game's own
30 fps. The frame is CPU-bound: preparing title memory for the card, not the card itself.

More: [KNOWN_FAILURES.md](KNOWN_FAILURES.md) · [MASTER_STATUS.md](MASTER_STATUS.md) ·
[ROOT_CAUSE_ANALYSIS.md](ROOT_CAUSE_ANALYSIS.md)

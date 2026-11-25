# Performance optimisation log

Each change is measured against the run before it. Game-run numbers come from the frame line,
averaged over the steady world frames of a run.

## 2026-11-25: descriptor sync change check (patch 0012)

Run next1 spent 49.5 ms a frame in `sync_heaps`, which looks at 234,346 slots a frame and mirrors
50,399 of them.

The change check was a BLAKE3 hash of each 32-byte record, under one lock per slot. It is now the
exact 12-byte key that `HostHeap::mirror` reads (tag, view index, generation), with one lock per
64 KiB page. Same answers, by construction and checked by the benchmark's assert.

Ignored test `change_check_cost` on the test machine: **15.1 ms → 0.83 ms** for 234,000 slots. The
remaining cost is the mirroring itself. Not yet measured in a game run.

## Switches ready for the next run (next2)

| Switch | Expected effect | Evidence |
|---|---|---|
| `VICTORY_WIDE_REFRESH=1`, `VICTORY_WIDE_MERGE=1` | buffer copies (88 ms in next1) spread over threads | on in HybridGt's fastest earlier runs (3.5–3.8 fps); off in next1 |
| `VICTORY_LEVELS_ON_DEMAND=1` | textures get card memory only for the levels the title holds; the card was full in next1 (6,258 slots waiting) | upstream `a26859f`: 3.1 GB not allocated on an RTX 4070, no device loss |
| `VICTORY_FIRSTLANE_OWN=1` + `VICTORY_BVH_BUILDER=1` | ray-tracing structure builder: its last "first-lane read" programs translate, so the builder can run | ZTRX #55: world "lit" at 4–5 fps on an RTX 5060 with the builder on. The builder turns itself off if any of its pipelines is missing |

`VICTORY_FIRSTLANE_OWN` is part of the shader cache key, so next2 needs its own warm-up
(`cache-next2`).

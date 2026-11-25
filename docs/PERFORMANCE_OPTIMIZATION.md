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

## Run next2 (2026-11-25): builder + levels on demand + wide refresh. Result: crash

Shader cache warmed with `VICTORY_FIRSTLANE_OWN=1`: 25,741 translated, 244 refused (3 more than
next1 translate). The run used `VICTORY_BVH_BUILDER=1`, `VICTORY_LEVELS_ON_DEMAND=1`,
`VICTORY_WIDE_REFRESH=1` and `VICTORY_WIDE_MERGE=1`. Log: `logs/next2.log`.

- The ray-tracing structure builder ran: "all of its programs have host pipelines pipelines=53".
- Levels on demand worked: 3,014 MB of the card not allocated for levels the title does not
  hold. Still 13,932 texture slots waiting while the world loaded.
- About 2 min in, still loading, the title crashed: access violations on many title threads at
  once, with registers holding float bounds (`0xff8000007f800000`, `0xff800000ff800000`). This
  is the failure `docs/fork-ray-tracing.md` describes for a partly wrong builder (upstream run 98):
  box bounds stored over the title's own pointers.
- Likely cause: `VICTORY_FIRSTLANE_OWN=1` is a shortcut, exact only when the value is the same in
  every lane of the console wave. At least one builder program reads a value that differs per
  lane. The fix is an exact cross-wave first-lane read (the console wave's first active lane can
  be in the other host wave). Until then the builder stays off.

## Run next3 (2026-11-25): levels on demand + wide refresh + patch 0012, builder off

Same as next2 without the builder; cache `cache-next`. The user reports textures and assets now
load, at about 3 fps. Steady world frames 08:18:57–08:19:27 against next1:

| | next1 | next3 |
|---|---|---|
| fps | 1.9–3.2 (3.0 steady) | 3.0–3.4 |
| ms/frame | 309–337 | 296–332 |
| CPU preparing | 118–140 ms | 124–144 ms |
| buffer copies | 73–88 ms, 190–194 MB | 73–80 ms, 171–177 MB |
| GPU | 110–130 ms | 99–124 ms |
| copy-back thread | 154–182 ms | 148–174 ms |
| descriptor sync | 49.5 ms (234k slots, 50k mirrored) | 37.4 ms (195k slots, 41k mirrored) |
| textures waiting for memory | 6,258 (card full) | 9,537 waiting, 2,875 MB not allocated for absent levels |

FidelityFX: the title made no FidelityFX call in the Series X profile (no "FidelityFX" log line in
next1 or next3). It asked for the upscaler only in Series S mode (`--console series-s`, run fork13).

## Benchmark procedure (from run next8 on)

The same save (`HybridGt-work/data`), Series X profile, 1080p, cache `cache-next3` (pipelines
saved). `VICTORY_KEYS` presses Enter every 4 s from 20 s to 120 s. `autoclose.sh` closes the game
3 minutes after the first 3 consecutive frame lines under 500 ms with 3,000+ draws. No user
input. "Steady" means frame lines after the world settled with the camera still.

## Runs on victory-gta6 `9376fd6` (fork's CPU-side frame commit) + ZTRX shader PRs #10, #51, #56

| Run | Change against the run before | Steady world fps | ms/frame | Notes |
|---|---|---|---|---|
| next7 | new base and new cache, no saved pipelines | (first run) | 335–700 | 1,000 pipelines made in the run; cutscene draws dropped, slow load. Not a regression |
| next8 | remaps: scatter pages joined into ranges | crash | — | joined pages were several placeholders: `MapViewOfFile3` error 487. Fixed in next9 |
| next9 | remaps: both locks once per scatter call; no-op remaps skipped | **3.5–3.6** | **276–284** | 15 min, no crash. Steady frames: preparing 96–104, GPU 150–155, copy-back 165–181 ms |
| next10 | `VICTORY_CARD_FETCH_INFLIGHT=1` (fork switch) | 1.5–1.7 | 578–700 | GPU 420–480 ms (×3): 87 MB a frame copied to PC memory inside the GPU's timeline. Copy-back thread 70–90 ms. **Rejected** |

next9 against next3 (the best run on the old base): 276–284 ms vs 296–332 ms per steady world
frame. Several changes came in together (upstream `9376fd6`, the ZTRX shader PRs, the remap
locking), so the gain is not attributed to one of them.

### Remap lock (from the next9 counters)
- While loading: 127–900 lock takings a frame, 1,700–3,500 pages, **held 700–1,300 ms a frame**,
  waited for up to 12 ms. So after the change the cost is the holding: about 0.5 ms a page.
- Steady world: 4–40 pages a frame, about 1 ms held.

Next: split the held time between system calls and GPU-side notifications (timers added, next11).

### Copy-back (next9, steady)
- The submitting thread waits 154 ms a frame in "title memory read" settles (2 a frame: one for
  a title buffer copy, one for an upload fill).
- The relay thread waits 134 ms for 7 fetches.
- The GPU is 64% busy.

Next: title buffer copies on the card instead of waiting (`VICTORY_GPU_BUFFER_COPIES=1`, next11).

## Runs next11–next12: copy-back waits removed from the submitting thread

| Run | Change | Steady world fps | ms/frame | Submitting thread's settles |
|---|---|---|---|---|
| next9 | (reference) | 3.5–3.6 | 276–284 | 2 a frame, 154 ms ("title memory read": a buffer copy, an upload fill) |
| next11 | `VICTORY_GPU_BUFFER_COPIES=1` (patch 0020) | 3.7–3.8 | 261–273 | 1 a frame, 97 ms (the upload fill) |
| next12 | + `VICTORY_GPU_FILLS=1` (patch 0021) | **3.7–4.0** | **252–270** | 1–1.5 a frame, 83 ms, now "title memory changed under shader writes" |

next12 moved 15 buffer copies and 5,600–6,100 scratch fills a frame onto the GPU. CPU copies fell
from about 185 MB to about 160 MB a frame.

### What is left in a steady frame (next12, about 260 ms)

| Part | Time | Cause |
|---|---|---|
| Copy-back thread | 160–188 ms | 100 ms waiting for 7–8 fetches; 66 ms with nothing to land |
| CPU preparation | 101–117 ms | copies 53–62 ms for about 160 MB; textures 2–4 ms |
| GPU | 78–115 ms | the card is 66% busy |
| Settle "title memory changed under shader writes" | 77 ms | the title's CPU writes 85,000–102,000 runs a frame (0.21–0.26 MB, 2–3 bytes each) into pages that flights' shaders write. The merge path (`merge_under_writes`) allows 512 runs a submission, then the submission waits for every flight |

### Measured and rejected or deferred

- `VICTORY_CARD_FETCH_INFLIGHT=1`: GPU time ×3 (next10).
- Kernel reads are not the copy cost. `read_cost` benchmark: 155 MB in 64 KiB reads takes 20 ms
  through the kernel, 14 ms as a plain copy. Refreshes already read whole runs of 64 KiB pages.
  The rest of the 53–62 ms needs the diagnostics build's copy counters.
- **Masked merge on the GPU, designed but not built.** For the 85,000–102,000 tiny runs: one
  compute dispatch would write only the title's bytes into the card copy. It needs two things:
  - an indexed log of the merged runs with their send number (the copy-back skips bytes sent
    after its fetch);
  - hashing of the merged pages after the merge.

  This is the next target. It would remove the remaining 77 ms settle.

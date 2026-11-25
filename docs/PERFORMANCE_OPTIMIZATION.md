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

## Runs next13–next14: masked merge on the card (patches 0023–0025)

| Run | Change | Steady world fps | ms/frame | Settle left |
|---|---|---|---|---|
| next12 | (reference) | 3.7–4.0 | 252–270 | 77 ms "title memory changed under shader writes" |
| next13 | `VICTORY_GPU_MERGE=1`, log in a tree | 3.4–4.0 | 252–298 | that settle gone (35 ms "lists in the air"). CPU preparation rose to 125–183 ms: 242,000 runs a frame, a tree insert each |
| next14 | log as one vector per send, binary search | **3.7–4.2** | **239–272** | 69 ms "256 lists in the air" (`VICTORY_LATE_WAIT_LISTS`) |

In next14 the merge step takes 10–23 ms of the submitting thread for 114,000–486,000 dwords a
frame. Those are merged on the card in 1.5–6.4 dispatches. No device loss, no crash.

Correctness checks:
- `merge_log_tests`: masks, joined dwords, unseen runs, pruning, runs merged again. This test
  found a bug: a run merged again at the same offset replaced the older one, and that run would
  have been skipped forever.
- `card_merge_tests::the_merge_writes_the_masked_bytes_and_nothing_else`: on the RTX 4070 Ti,
  every byte of a 64 KiB buffer is checked.

## 2026-11-25, afternoon: merges, runs next16-next17

Branch `hybridgt/next3` (exported as `patches/victory-gta6-next3/`, one diff on
fxceless/victory-fork `a253706`):

- Merged fxceless/victory-gta6 `f85101b` (submit thread `VICTORY_ASYNC_SUBMIT`, copy queue
  `VICTORY_QUEUE2`, indirect draw windows, profiler) and fxceless/victory-fork `a253706` (frame
  log, driver watch, heap lock; their figure: 6.0 -> 8.9 fps with `VICTORY_INDIRECT_WINDOW=1` on
  an RTX 4070, first room). Our relay copy-queue fetch defers to `VICTORY_QUEUE2`.
- 13 ZTRX PR commits (shader lowering: f64, packed dot, quad masks, interpolation, LOD; uniform
  LDS reads, which make the last 3 BVH builder programs translate; batched texture probes).
- POPS pixel shaders translate (10 ped cut-out programs), after upstream `522d2bc`.
- xmem: no split call where our own unmap left an exact placeholder; no commit call for
  section pages committed already.
- FSR 1 (AMD, MIT, vendored) on present: `VICTORY_FSR=1`, `VICTORY_FSR_OUTPUT=WxH`; GPU test.
- `victory.exe` has an icon of its own (original design).

| Run | Build / switches | Scene | Frames |
|---|---|---|---|
| next16 | next2 + `VICTORY_FETCH_COPY_QUEUE=1` | world, user playing and driving | median 300 ms (next14: 255 ms); not the same scene, no gain claimed |
| next17 | next3 + async submit, queue2, windows, texture no-wait | **new game** (saves moved to `save-backup/`): prologue cutscene and streaming | 0.6-4.2 fps; remapping held 372 ms/frame (6,200 pages, 260 ms in system calls), card ~11 % busy, up to 866 draws dropped while pipelines were first built (new cache) |

The new-game scene is not comparable with next14's save. Next: a run on the warm cache with the
xmem change, then 1080p against 720p+FSR.

Shader translation (offline, after the POPS fix): 25,751 of 25,985 programs. Left: 196 geometry
shaders (no geometry stage in the translator; they also read a driver table through user data
0-1), 29 DXBC programs inside the console's FidelityFX DLL (never run: the host's upscaler is used),
5 shadow-ray programs (read the BVH's address from a descriptor; useless until the BVH builder
works), 1 nested waterfall (opaque lane mask), 1 lane mask exchanged under a partial EXEC.

### Run next18b (next3 + xmem change + POPS shaders, warm cache, same switches as next17)

New game, prison cell after the opening: **4.3-5.3 fps** (190-230 ms a frame, 4,700-6,600
draws), dropped 250-390 draws a frame without a host pipeline. Of a frame: copies of title
memory about 80 ms (130-160 MB), copy-back about 100 ms (50-58 MB); remapping 0.6-2 ms (the
scene was not streaming, so the xmem change is not measured by this).

Seen by the user: sunlight edges in tile-sized blocks on faces and clothes, and characters' eyes
glowing. The blocks fit a shadow mask whose edge-refining pass does not run (candidates:
`CS_ShadowRenderTiledCascadesAlpha*` and `CS_TraceShadowRays_*`, both still untranslated; three
compute pipelines the host refused, `6b82bc7d22016cb5`, `2da4beb386509b77`, `02384a60fda6f2d0`).
The eyes may come from the POPS ped shaders first drawn in this run: that path is now off unless
`VICTORY_POPS=1` until a run with it off says.

### Runs next19-next21 (in place, geometry stages)

- **next19** (POPS off, `VICTORY_TITLE_GS=1` before the switch existed, `VICTORY_INPLACE=1` with
  the probe): the title crashed 1 min 57 s in (prologue cutscene prestream; access violation in
  the title's code reading unmapped memory). Probe on the RTX 4070 Ti: private memory and a
  **committed page pool section** open in place, read and write; a reserved section is refused.
- **next20** (only `VICTORY_TITLE_GS=1`): no crash; 3.4-4.5 fps in the cell. The title made no
  pipeline with a geometry stage there.
- **next21** (`VICTORY_TITLE_GS=1 VICTORY_POOL_COMMIT=1 VICTORY_INPLACE=2`): **slower, 2.4-3.1 fps**
  (320-420 ms). In place: 81 windows 384 MB, 3,606 title buffers 40 MB (78 in page pool
  sections); 1,041 turned down (view memory scattered over the pool, or not committed whole).
  Copies fell from ~80 to 42-60 ms, copy-back stayed at ~80-100 ms, and GPU time rose to
  60-130 ms with the card ~100 % busy: placed buffers are in system memory, read over PCIe.
  Conclusion: in place only for small buffers the CPU rewrites each frame; data the card mostly
  reads stays in its own memory. Window closed by the user at the end (no crash).
- Three compute pipelines the host refused (`2da4beb386509b77` alphaResolve prefilter,
  `6b82bc7d22016cb5` / `02384a60fda6f2d0` VFX triangle clip): **the shaders require 64-lane waves;
  the RTX 4070 Ti has 32** (debug layer). The cascade shadow program
  `CS_ShadowRenderTiledCascadesAlpha*` fails for the same reason: wave64 programs whose thread
  group holds several console waves need branches made group-wide (not yet in the translator).
- Screenshots of runs: `HybridGt-work/captures/<run>/` (`tools/capture-loop.ps1`, every 30 s).

### Runs next22-next23: where the remaining frame goes

- **next22** (`VICTORY_READ_PROBE=8`): of 153,816 copied-back pieces probed (7.2 GB), the
  title's CPU touched **10,046 (6.5 %)** before the card wrote them again. About 93 % of the
  copy-back (80-100 ms a frame) is never read. 4.5 fps; heap pass 87 ms, copies 66 ms,
  copy-back 99 ms.
- **next23** (heap pass timed in parts, `VICTORY_UNWATCHED_FRAMES=2`): 4.4-4.9 fps. The heap
  pass is **45-53 ms over buffer slots** (14,000-50,000 slots re-made a frame) and **0.5-0.8 ms
  refreshing**; refreshing unwatched memory every second frame changed nothing.
- Research (web, summarised): shadPS4 (PS4 on PC, same AMD unified memory) has a "precise"
  readback mode: GPU-written pages lose CPU access and are downloaded on first touch (a 512 KiB
  window), 2.7-7.2 ms a frame measured on an RTX 5090. Xenia mirrors guest memory in one VRAM
  buffer (tiled, default heaps) and never copies back. Ryujinx moves buffers between memory kinds
  by how they are used. `D3D12_HEAP_TYPE_GPU_UPLOAD` (Resizable BAR) lets the CPU write VRAM
  directly. No project runs wave64 shaders on 32-wide NVIDIA hardware other than by groupshared
  emulation.
- Built from that (fork `70cf037`): copy-back on first touch (`VICTORY_LAZY_BACK=1`) and the
  heap pass keeping slots whose records are unchanged (on by default; `VICTORY_SLOT_PRINTS=0`).
  Measured in next24.

### Runs next24-next28: two large savings that crashed, one kept

| Run | Switches | Result |
|---|---|---|
| next24 | lazy copy-back + slot prints (first form) | crash (title writes through null) ~6 min in; copy-back 80-100 -> ~7 ms before it; texture pass 2-7 -> 105-181 ms (`ensure` took a lock per read) |
| next25 | + all-or-nothing closing, guard re-closing | crash, same kind |
| next26 | + every kernel read hooked, block filter | crash while loading **with 0 pieces deferred**: slot prints |
| next27 | lazy copy-back alone | crash (same title offset as next25): lazy copy-back on its own too |
| next28 | slot prints that also require the same copy (`VICTORY_SLOT_PRINTS=1`) | no crash; median 196 ms (5.1 fps), p90 918 ms; only 4,000-5,500 slots kept a frame (most slots show a different copy each frame) |

Both are off by default. Lazy copy-back needs every path by which the title or this runtime sees
title memory found first; the likely kind is a reader that neither faults nor goes through
`crate::host`'s reads. Frame-time spread: next23 median 235 ms, p90 663 ms, worst 3.9 s; the
spikes (one frame in ten over 0.6 s) are what play feels like.

Copies move 185 MB in 81 ms (2.3 GB/s): overhead, not bandwidth. Next: the sampling profiler and
frame log (diagnostics build, next29) to see where those milliseconds and the spikes go.

### Runs next29-next31: the crashes are not the speed changes

- Every crash of runs next19-next30 (seven reports) came at **the same moment**: the prologue's
  intro cutscene ("cine@leadout@int_prolo@int@leadout") in its *Prestream* state, 1 min 23 s to
  1 min 57 s into a new game (5 min 26 s once), the title writing through null. next26 and next30
  crashed there with **nothing deferred** by the lazy copy-back. Runs before the saves were moved
  (next1-next18) loaded a save past this cutscene.
- At that moment the title makes thousands of pipelines through the two device methods the runtime
  does not implement: **16,384 calls of 0x1c8 (graphics) and 2,048 of 0x218 (compute)** in next30.
- next31: answering them with a placeholder pipeline (no shaders) instead of a failure made the
  title draw its frame through them: ~9 draws a frame, black screen with sound. Off again
  (`VICTORY_PLACEHOLDER_PIPELINES=1` to try).
- Consequence: **methods 0x1c8 and 0x218 are part of how the Series X build renders** (not only
  Series S). Their pipelines are where draws are missing (the user sees windows and other assets
  wrong). Implementing them is now the first task for stability and picture both. The descriptions
  carry native GFX10 code with the same hash header as the shader containers' code, the register
  state and metadata records (`docs/SERIES_S_AND_FSR.md`).
- The lazy copy-back fix (other host copies told of a deferred piece) is in the tree (fork
  `5c..`), not yet measured past the cutscene.

### Device methods 0x1c8/0x218: the code is the containers'; DexNom/VIctory

- next32 (200 s, diagnostic): the descriptions name standalone code in GPU memory (no container
  around it). 0x1c8: w1/w2 registers (0x184), w3/w4 a 0xbc block starting with a hash, w5/w6 vertex
  code, w7/w8 its metadata, w9/w10 pixel code, w11/w12 its metadata. 0x218: entries of 12 words,
  w1/w2 registers (0x1c, thread group), w3/w4 hash and 3, w5/w6 code, w7/w8 metadata (0x403).
- **All 16 code headers seen (the two hash-carrying `v_nop`s) match exactly one container's native
  code in `shaders.rpf`.** Plan: an index from code header to archive record (built offline by our
  tool), then real pipelines from the containers the translator already handles: compute (0x218)
  first, then graphics (0x1c8), whose blend/depth/target state must be read from the register block.
- DexNom/VIctory (upstream VIctory + 10 commits): same stub for 0x1c8/0x218; experimental ray
  tracing on an RX 7900 XTX ("frame times can still reach several seconds"); five shadow-ray
  compute shaders and the voxel-visibility shader recovered. Ported: `76749ce` (voxel visibility
  now translates and validates here). Its shadow, ray-structure and shader-variant commits
  (`5f4c3df`, `bc3cd4b`, `921d8d3`, `ddb7332`) conflict with this tree's translator and need a
  merge by hand.

### Runs next33-next34: method 0x218 implemented

- `victory shader-index <archive> <file>` indexes 26,698 native programs by their first 12 bytes
  (984 headers shared by more than one container). With `VICTORY_NATIVE_INDEX=<file>` and
  `VICTORY_NATIVE_PIPELINES=1`, method 0x218 makes compute pipelines from those containers.
- **next33**: 1,024+ compute pipelines made, 0 missed; reached the world, 5.1-5.3 fps (188-197 ms:
  copies 50-64 ms, copy-back 82-91 ms, heap pass 53-80 ms); no crash.
- **next34** (same + lazy copy-back): crashed at the prologue cutscene, same title offset
  (`+0x23cf683`), nothing deferred. next33 passed it by timing. The remaining suspect is method
  0x1c8 (graphics), still failing there. Next: 0x1c8 from containers, with its graphics state read
  from the register block (w1/w2, 0x184 bytes, starting `4005d808`).

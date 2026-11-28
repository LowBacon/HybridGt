[← Documentation](README.md)

> Research only: nothing here has been merged or measured by HybridGt. Every figure is the other project's own, from its own machine.

# HybridGt merge candidates and new ports, 2026-11-28

**Short answer:** fast-forward our runtime base from `df2daafe` to `trustno1-coder/VIctory-NVIDIA` main first. Then port four focused items on top:
- the VRAM gate from fxceless (black/late textures)
- the write-fault changes from VIctory-Neo (CPU time)
- PR #854, which links mesh programs with their pixel program and turns on paired wave64
- a cherry-pick of Marvisak's NVIDIA crash and memory fixes, after a patch-id check

The fast-forward alone is fps, crash and memory work, all GPL-3.0, with no rebase. Nothing in this report has been run or measured on our 4070 Ti. Every fps figure is the other project's own number from its own machine.

**Base facts I checked with `gh` (read-only):**
- trustno1 main is now `9ce3cf9e` (2026-11-28 14:55Z). It is 62 commits ahead of `df2daafe` and 0 behind, so it merges cleanly as a fast-forward.
- Three items reported elsewhere are **already in trustno1 main**, so they need no separate port:
  - foxhound's texture-eviction one-liner (`r.host.used.store(0, Relaxed)` in `let_go_idle`)
  - foxhound's shared prelude pool (`VICTORY_PRELUDE_POOL`)
  - fxceless's fixed static-sampler order (`pipeline/samplers.rs`, `VICTORY_SAMPLER_ORDER`)
- `twin_exact` / `VICTORY_TWIN_EXACT` does not exist in our base or in trustno1 main. Slaktarn's native-D3D12 device-removal fix therefore does not apply to us.

## 1. Top merge candidates, ranked by expected value

| # | Source | Commit(s) | What | Why it matters for us | Evidence | Licence | Risk |
|---|---|---|---|---|---|---|---|
| 1 | trustno1-coder/VIctory-NVIDIA (fast-forward) | `df2daafe..9ce3cf9e` (62 commits) | **fps:** tearing/independent flip (8fc22547); frame generation only when asked (b4e1b082). **Memory:** memory stack on by default (63b902fe); water light, texture sweep, sampler order, prelude pool (64d4931b). **Crashes:** 74d6f240 alias UAF, 108a8971 XMem 21-bit page numbers, 1d3b4143 descriptor-slot leak, 4f5ae578 pipeline panic, e205498c fences on device removal, a12e0ae2 unspill table. **Shaders:** c1d81790 wave64 lane bits. **Window:** 3f054e46 present to the title's window. **Diagnostics:** 9fed8976 frame-time spread, ecfa1a6a POOL_STATS, 4e533e79 WAVE_WHY. **Input:** mouse and keyboard (4715a428..f4608f0c). | Hits fps, crashes, RAM, VRAM, water and shader correctness in one merge | Measured: about +2 fps (tearing); flat 14 fps removed (#515/#574); UAF/XMem traces; prelude pool cuts about 0.8 GB commit | GPL-3.0 | **Low-medium.** No rebase, but conflicts land in our 9-commit patch: F1 branding vs F1 changes in 64d4931b/5986a64d/c1cbdc9d/63b902fe/2a66ef1b; FidelityFX stand-in vs 63b902fe/c1cbdc9d FidelityFX checks (540p `0xc0000094`); validator fixes vs 0745a5de/feb90ade/0dd34bdb. Re-run the GPU hang gate after the merge. |
| 2 | fxceless/victory-fork | 2eade68c (`VICTORY_POOL_GATE`) | VRAM budget gate for new 64 MB texture blocks, with reserved room for render targets and copies | Directly targets black/late textures when VRAM is full on 12 GB | RTX 4070: was 0.8-1.0 GB over budget with 90→160 ms frames; after, 8.1-8.7 of 10 GB used and "no black texture" in a 4.5 min run | GPL-3.0 | **Medium.** fxceless is now on trustno1 `959dcaf` (same family as us), so it ports by hand into resource.rs. Code search found no POOL_GATE in trustno1 main (search index may be stale). trustno1 PR #399 is a similar gate. |
| 3 | KyleS98/VIctory-Neo | 0fb1e38e (`VICTORY_PAGE_ASK`), 38b768c7 (`FAULT_UNLOCKED`, `STORE_BESIDE`, `CARD_AWAKE=0`), per-thread object-return queues | Write fault asks about one page instead of a whole region; title threads no longer queue on the write-watch lock | This is the CPU-bound title-memory sync behind our 13-18 fps | Measured, one PC: 15.5-17.3 → 2.8-3.3 ms/frame title-thread time; return-queue lock was 3.3 of 22 ms; 30.7 → 31.7 fps (CARD_AWAKE). r32601366 reports +7-9% for CARD_AWAKE=0 under Proton. | GPL-3.0 | **Low-medium.** Built from trustno1 main of 2026-11-27, very close to our base. Skip `VICTORY_OWN_CPUS` (only helps two-CCD CPUs; the 5800X3D is one CCD). |
| 4 | fxceless/victory-fork PR #854 (open) | 5 commits, +3645/-1075 | Links each mesh program with its pixel program (only outputs the pixel shader reads); paired wave64 on by default on 32-lane hosts | Addresses **71 mesh shaders over 32 KB** and the **23 wave64 compute shaders** (same counts as ours) | 710 mesh-pixel pairs link and validate offline; RTX 4070 SUPER refusals 27 → 0; no frame cost on 5070 Ti; 3 programs still refused | GPL-3.0 | **Medium-high.** Large and unmerged, built on Classic `9a9091d1` rather than our runtime base. Paired DXIL is about 2.9x larger and untimed in a game run. Port it behind its switches. |
| 5 | Marvisak/VIctory (cherry-picks, patch-id check first) | bac1364a, ea488a9f, 219878a0, 9a12a45e, 53126284, 555c112e, 781e846d, 53193663, 106377d1, 136e39d0, 384b0c0b | **NVIDIA crashes:** B5G5R5A1 typed-UAV device removal (two fixes); null SRV in `alias_source`. **VRAM:** depth-twin scratch sharing (about 325 MB). **Leak:** page table 16.8 MiB each. **Hitches:** pipeline-library lock stall every 20 s. **CPU sync:** write ranges armed as merged runs. **Copy-back correctness** fixes. | Crashes, VRAM, hitches, CPU sync, and possibly stale or missing GPU-written data | Debug-layer logs (RTX 5090, Win10); 325 MB; tests for the lock | GPL-3.0 | **Medium.** Our base and Marvisak diverged (Marvisak is 479 ahead and 1210 behind); some may already be in. Skip e04c0781 (constant table): its stated effect is under vkd3d-proton, and we run native D3D12, so likely low value. |
| 6 | Slaktarn700/SlaktarnVI | 03f0f104 `ambient.rs` (`VICTORY_AMBIENT_REFLECT`, `AMBIENT_AIR`) | Clamps reflections that fall back to the open-sky cube to a multiple of local ambient, before transparent materials read them | Directly targets **sky light on glass**, eyes and windows, with no RT structures | Doc comment shows cornea and window cases; no fps cost given; AMD/Proton only | GPL-3.0-or-later (Belobug origin) | **Medium.** Diff it against the ambient pass already in our base. jennydeth/Hail-to-the-Thief (on trustno1 63b902fe) has related tuning: 0.8-1.5 strength, contact shadows. |
| 7 | edsuuu/VI-BR-PORT | `VICTORY_RT_MISS` + skip 30 `CSComputeSkinning*`; `CARD_COMPARE_EVERY=30`; `CARD_TOUCHED`; render-target room follows actual use | Removes GPU work that only feeds the unused BVH; fewer buffer reads; textures no longer refused while VRAM is free | fps and textures/VRAM on 8-12 GB cards | Measured, docs/BENCHMARKS.md: 6.8 → 10.9 fps (skinning); 2.4 → 0.5 GB/frame reads; RT room 3 → 1.4 GB | GPL-3.0 | **Low-medium.** Switch-gated. Check the lineage diff first. |
| 8 | Spaghetih/victory-sley-fork | mimalloc (`VICTORY_ALLOC`); CS_DefragHeap skip; `VICTORY_WAVE_WIDE_ONLY`; textures hashed by turns | RAM, fps, wave64 coverage | **Same GPU as ours (4070 Ti 12 GB).** Frame breakdown matches our diagnosis (21.7 ms copying title memory + 9.6 ms copy-back). | Measured: RAM 24.0 → 21.2 GB flat; 20.3 → 24.1 fps (defrag skip); 9 of 10 wave64 lowered | GPL-3.0-or-later | **Medium.** Lineage not confirmed. Skipping CS_DefragHeap may have side effects later in a session. |
| 9 | trustno1 open PRs | #458 (descriptor waterfall loops), #716 (195 GS_VoxelizeGeneric tracking), #822 (LDS-atomic wave64 → wave32), #82 (rays miss unbuilt structures + NaN scrub), #1157/#1094 (lock contention) | GPU time, shader coverage, white screen | #458: 27.6 → 24.4 ms GPU (VIctory-Neo); #716: 195 → 0 failures, but tracking only, still no GS pipeline; #82 is a lead on the white screen | #458 and #716 measured/tested | GPL-3.0 | **Low.** Small, our own family. #716 alone will not draw the geometry shaders. |
| 10 | naixue2233/Victory-naixue2233 | 2892ad207c (multithreaded submission copies), f9e0371a6d (CS_RaytracingResolve deny list), `MASKED_PATCH`/`MESH_LAUNCH` | CPU copy time; possible card-loss crash avoidance | CPU-bound fps, crashes | Run 362: card wait 21 → 2 ms, frame −10 to −15 ms | GPL-3.0 | **Medium.** Provenance unclear (descendant of ztrx, whose original repo is gone). |
| 11 | AnyXboxX PR #574 / El1xr | `VICTORY_LAZY_BACK` (copy back only on CPU touch); El1xr f1a346c8 `VICTORY_INPLACE_WINDOWS` | Copy-back cost (10-17 ms/frame); RAM for large windows | fps and RAM | Lazy-back has no in-game A/B yet; INPLACE: +2 MB instead of +257 MB commit per 256 MB window | GPL-3.0 / GPL-3.0-or-later | **Medium.** Different branch (Vice-Tay/AnyXboxX). |
| 12 | zm0kiemedia/Neptune_VI | 61f22060 (stale texture kept until its replacement uploads, `STALE_KEEP_MAX_MS`) | Fixes black sky / blank textures during replacement | Black/late textures | No numbers | GPL-3.0-or-later | **Low-medium.** Different branch (Marvisak/fxceless). Its THIN and CLAMP_AFTER_WRITER overlap things our base already has. |
| 13 | pepezin-ai | 1469efe2 | DLSS-G constants from every upscale | Only if frame generation is still dropped | Log evidence | GPL-3.0 | **Low value.** Written for the old fg.rs. trustno1 c1cbdc9d/b4e1b082 rework this path. |

**White screen while driving (lead, not a fix).** trustno1 issue #466 reproduces a full white screen on `df2daaf` (our base) at the first bind of `CS_RayMiss 06bb6e728c7c99bf` (RTX 4050/4060). The maintainer asks whether v2026.11.27.8 still shows it, since that version changed lighting a lot. The best paths are to test after the fast-forward (#1), then try PR #82's NaN scrub. 3f054e46 fixes a different white-window cause (a Windows Terminal pseudo-console window got the swap chain).

**Reflections, no ready candidate.** The options are:
- the title's own ray tracing (trustno1 800dada3: about 35 → 23.5 fps and a broken picture; the cost is on the CPU side)
- fxceless ray guide (its README says no gain)
- aleksesipenko/VIctory-RTX DXR sidecar (about halves fps; heads and clothing lost)
- TomTomAIbot/LinVxtory DLSS Ray Reconstruction (Proton only)
- DexNom `--rt-static-bvh` (AMD, NVIDIA untested)

Ambient clamping (#6) is the realistic short-term fix for sky light on glass.

## 2. New projects and ports worth watching (claims not verified)

**Same family as ours:**
- **KyleS98/VIctory-Neo** (GPL-3.0): 37-39 fps in the prologue cell, RTX 3090 Ti + 9950X3D, Series S. Per-commit measurements on one PC. Uses 36-42 GB RAM.
- **Spaghetih/victory-sley-fork** (GPL-3.0-or-later): 20.6 → 24-26 fps on a **4070 Ti 12 GB**. The most comparable machine to ours.
- **vilaris/VIctory-NVIDIA** (GPL-3.0): trace-based analysis on a 4070. Wait-for-idle is about 45% of GPU time and guard faults cost 51-54 ms/frame. The X9 backend was a net loss. Useful reading, not a merge.
- **jennydeth/Hail-to-the-Thief** (GPL-3.0): ambient stand-in tuning on a 3060, on trustno1 63b902fe.
- **foxhound 0.2.0-pre.5** (7c535276; 8 GB cards, DLSS and FG on Series S, profiles): released after our base. Its contents were not checked in detail.

**Other VIctory-line runtimes:**
- **edsuuu/VI-BR-PORT** (GPL-3.0): 26-29 fps on a 4060 Ti 8 GB, scripted runs.
- **Biskwit/VIctory-reloaded** (GPL-3.0): 28.8 fps in the cell on a 4090.
- **voidopcode/vilocity**: 28-33 fps on a 5080. **No LICENSE file: do not merge.**
- **Vice-Tay** (GPL-3.0): GPU read-through, 22.6 → 26.4 fps. Development is paused.
- **AnyXboxX** (GPL-3.0-or-later): about 24 fps on a 4070 SUPER, self-reported.
- **El1xr** (GPL-3.0-or-later): 23-26 fps on a 5090 at 4K, but about 4 fps on a 4070 SUPER.
- **Kelvaroq** (GPL-3.0): the "memory layer" that maps title memory behind GPU page tables (22 ms vs 140 ms) is **private and unverifiable**. Its analysis matches ours; watch it.
- **Low credibility:** Kemerd/VIctory-NVIDIA-RT (rewritten history, marketing-style numbers, no RT yet), Bebrik229 (inconsistent claims), ToLIManl/VIctory-Hydra (fps comes partly from LOD cuts).
- **Integration trees without their own measurements:** MystifiedSky, Fayberr/Victory-United, LegitCamper.

**Reflections / RT research:** aleksesipenko/VIctory-RTX, TomTomAIbot/LinVxtory (DLSS RR), Xeltax/VIcky (about 6.2 fps with RT on a 4080), whoalin1/victory-altracing-prs, nightmarecatt (BVH compacted sizes).

**Texture/VRAM:** crbns/victory-tapin (8 GB without grey or black textures, 14-15 fps on a 3070), ECHOZwei/TRIDENT (few measurements), Steenax6, gurvancampion (`TEXTURE_TURNS`), Fayberr (`TEXTURE_MAX_SIZE`).

**Licensing and other notes:**
- **XCL (xcl-dev/xcl)** relicensed GPL-2.0-only → GPL-3.0-or-later in 06f90c19 (2026-11-27), with no contributor sign-off. Treat only code written after 06f90c19, or code whose author confirms, as GPL-3.0-safe. Older XCL forks and ScarlettPort's `runtime/xcl` stay **GPL-2.0-only: do not merge.** XCL is a reference design for geometry shaders as compute + mesh and for wave64 as two halves (about 5 fps in the world).
- **zortos293/XSEmu** (XCL-based; same caveat).
- **TuxBox** (Apache-2.0; can be used in GPL-3.0 with notices kept).
- **R0Y00/Xbox2Windows** (MIT; mesh fallback and BVH test reference).
- **ProjectLeonida/mandy** (GPL-3.0; SPIR-V route, not directly usable for us).

**Avoid:**
- skornuta/victory-memory-stack (licence file removed; take the same work through trustno1 63b902fe instead)
- megaliv3/XboxBvhBuilder.dll-GTA-6 (decompiled title code; breaks our no-game-data rule)
- VIctory-GTA-6-PC-Port/.github and Yuan1145408/bird-gta6 (likely scams or malware; nothing opened)
- KestrelX and californiaAlmonds/OpenScarlett (low credibility); OpenCommunityEdition/OpenScarlett (AGPL-3.0)

**Official PC release:** not announced. A Q1 2027 date is rumour only.

## 3. Checked, nothing new to merge

- **pepezin-ai/VIctory-NVIDIA:** only 1469efe2 since 2026-11-27 03:51; inactive. Track trustno1 instead.
- **Zettrox/ztrx** (through mirrors): every code commit is already in our Classic merge base `9a9091d1`.
- **fxceless rebase:** the 5dbf5ce2 rebase onto trustno1 `959dcaf`, including the 854f57f port, is older than our base. Only fxceless's own work on top is new (2eade68c, cd935520, 3e573d25, 9e907466; 9e907466 is already in trustno1).
- **foxhound pre.3/pre.4:** the eviction-stamp fix, prelude pool and sampler order are already in trustno1 main.
- **DexNom/VIctory:** README-only commits since 2026-11-26. Its code is AMD-first and NVIDIA is untested.
- **Marvisak/VIctory:** no push since 2026-11-25 (only the candidates in #5).
- **Neptune_VI:** no commits since 2026-11-26.
- **Slaktarn 4f6a3758:** does not apply; our base has no exact-twin path.

## 4. Unreachable or removed

- **Zettrox/ztrx:** 404, account gone. Mirrors exist: random-archives/ztrx, grosterr/ztrx, MiKom/ztrx.
- **codec-d/cool_fox (CGBJ):** 404. Only the part pepezin merged in 63957271 can be traced. PCGH warns that re-uploads may carry malware.
- **Removed by GitHub, per trackers:** avatarmanz/VIctory-7900XTX and Belobug/VIctory-Belobug-AMD (reason second-hand from PortWatch: ToS). Op-lite/victory-scarlett, proneonn/Vertex, realdody/americas-recomp, victory-perf and keylime are also gone. Ports from avatarmanz now only reach us through trustno1/fxceless cherry-picks.
- **ItsNotRexed/Leonida and about 3,007 forks:** disabled by a Take-Two DMCA on 2026-11-24. This is a takedown risk for the whole fork network, HybridGt included.
- **vuyp/gta6-pc-race tracker:** stopped at 2026-11-28 00:17 UTC. The live tracker is portwatch-vi.xyz.

**Not checked:** trustno1 open PRs #919-#971 were listed but not read; fxceless main moved again to 368967f8 (13:25Z) after the reviewed head 6b3ce6ce.

The only files written were scratch copies of upstream `resource.rs`, in `C:\Users\RIVERH~1\AppData\Local\Temp\claude\e5502ff586696883\37a7070a-b8ea-4821-ad39-003ab45c4c24\scratchpad`.

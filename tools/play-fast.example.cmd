@echo off
set VICTORY_GAME_DIR=E:\GTA\GTA6.Xbox.v1.0.2.1
set VICTORY_CACHE_DIR=E:\GTA\HybridGt-work\cache-next3
set VICTORY_DATA_DIR=E:\GTA\HybridGt-work\data
set VICTORY_LOG=info
set VICTORY_DISPLAY_MODE=1920x1080
set VICTORY_FFX_DIR=E:\GTA\HybridGt-work\ffx-sdk\bin
set VICTORY_LEVELS_ON_DEMAND=1
set VICTORY_WIDE_REFRESH=1
set VICTORY_WIDE_MERGE=1
set VICTORY_GPU_BUFFER_COPIES=1
set VICTORY_GPU_FILLS=1
set VICTORY_GPU_MERGE=1
set VICTORY_ASYNC_SUBMIT=1
set VICTORY_QUEUE2=1
set VICTORY_INDIRECT_WINDOW=1
set VICTORY_TEXTURE_NO_WAIT=1
set VICTORY_JOIN_PARTS=1
set VICTORY_INDIRECT_GATHER=1
set VICTORY_CARD_REHASH_RUNS=1
set VICTORY_COPY_PLAIN=1
set VICTORY_BACK_OWN_VIEW=1
set VICTORY_NATIVE_INDEX=E:\GTA\HybridGt-work\native-index.txt
set VICTORY_NATIVE_PIPELINES=1
rem 0x1c8 graphics pipelines: OFF (with them on, character bodies were missing; the merged fxceless
rem code renders the prologue without them: A/B run 2026-11-26). Set to 1 only for tests.
set VICTORY_NATIVE_GRAPHICS=0
rem rectangle lists in 0x1c8 pipelines (the intro grids); delete to turn off
set VICTORY_NATIVE_RECTS=1
rem fxceless f6a09d3b test set (boats float with TEXTURE_BACK; texture pass on its own thread)
set VICTORY_TEXTURE_BACK=1
set VICTORY_HEAP_WORKER=1
set VICTORY_HEAP_LOCKS=1
set VICTORY_MERGE_GAP=64
set VICTORY_MERGE_RUNS=4096
set VICTORY_STAGING_POOL=0
rem less card memory for copies, more for textures (A/B 2026-11-26: 2048 -> 1024 "no room" 60 -> 20;
rem 1024 -> 512 textures waiting 12,000 -> 10,200 and 800 MB more free on the card; fps the same)
set VICTORY_CARD_WINDOWS_MB=512
set VICTORY_TEXTURE_ROOM_PERCENT=96
set VICTORY_CARD_BUFFERS_MIN_KB=64
set VICTORY_CARD_BUFFERS_MB=1280
set VICTORY_CARD_HASH_LEN=1
rem stand-in for the traced indirect light (VIctory-7900XTX 82ca1347): interiors and shadowed sides lit
rem instead of black; A/B 2026-11-26: 6.7 vs 7.7 fps median. Delete to turn off.
set VICTORY_AMBIENT=1
rem 19 compute programs NVIDIA refused for 64-lane waves get the wave-wide form (volumetric light, particles,
rem CS_Visibility, hair shadows). A/B 2026-11-26: no GPU loss, fps the same (6.8 median)
set VICTORY_WAVE_WIDE=last
rem skin and clothing bakes: their own textures are uploaded first (clothes and skins that stayed old)
set VICTORY_BAKE_SOURCES=full
rem fxceless CPU savings (A/B 2026-11-26, same route: 5.0 -> 6.9 fps median, p10 1.4 -> 3.7)
set VICTORY_LOOKUP_CACHE=1
set VICTORY_SETTLE_ROOM=1
set VICTORY_PARALLEL_REFRESH=1
E:\GTA\HybridGt-work\victory-gta6\target\release\victory.exe run "E:\GTA\GTA6.Xbox.v1.0.2.1\game_scarlett_master.exe" --execute > "E:\GTA\HybridGt-work\logs\%~n0.log" 2>&1

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
set VICTORY_CARD_WINDOWS_MB=2048
set VICTORY_CARD_BUFFERS_MIN_KB=64
set VICTORY_CARD_BUFFERS_MB=1280
set VICTORY_CARD_HASH_LEN=1
E:\GTA\HybridGt-work\victory-gta6\target\release\victory.exe run "E:\GTA\GTA6.Xbox.v1.0.2.1\game_scarlett_master.exe" --execute > "E:\GTA\HybridGt-work\logs\%~n0.log" 2>&1

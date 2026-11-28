# GPU and system memory

[← README](../README.md) · [Performance](PERFORMANCE.md) · [Troubleshooting](user/TROUBLESHOOTING.md)

## Why the runtime uses so much graphics memory

On the console, the CPU and GPU share one pool of memory. On a PC, the game's memory sits in system
RAM, and the GPU needs its own copy of whatever it reads. The runtime keeps three kinds of copies on
the graphics card:

| Copy | What it holds | How long it stays |
|---|---|---|
| Buffers | Vertex, index, constant and structured data the game's draws read | While the game's memory behind it is unchanged |
| Textures | Decoded and untiled copies of the game's textures | **Cached**: kept until room is needed |
| Render targets | Frame buffers, depth and the game's intermediate images | Their lifetime in the game |

Textures are by far the largest. **The texture cache deliberately fills most of the card.** Each new
texture is uploaded only if the card's use stays under a line, otherwise it waits. By default, the line
is the budget Windows gives the process, minus whichever is larger: 768 MB or a twentieth of the budget.
It is never under 90 % of the budget (`victory-gpu`, `resource::line_for`). On a 12 GB card that's
roughly 9.8 GB of a 10.6 GB budget.

So a graphics-memory reading close to the card's size is the cache working as designed, not a leak. At
the end of the measured route the runtime held **5.6 GB**, against 9.2 GB on the earlier runtime (see
[PERFORMANCE.md](PERFORMANCE.md)). Longer sessions in busier places fill more of it.

## The launcher's memory modes

**Settings → Memory** passes caps to the runtime. The switches are in
[`presets.rs`](../crates/hgt-launcher/src/presets.rs) and are unit-tested:

| Mode | Switches | Effect |
|---|---|---|
| **Auto** (default) | none | The runtime's own line, as above |
| **Balanced** | `VICTORY_TEXTURE_BUDGET_MB=6144`, `VICTORY_DSTORAGE_STAGING_MB=128` | Textures capped at 6 GB, smaller DirectStorage staging |
| **Low** | `VICTORY_TEXTURE_BUDGET_MB=4096`, `VICTORY_TEXTURE_ROOM_PERCENT=70`, `VICTORY_DSTORAGE_STAGING_MB=64` | Textures capped at 4 GB, 70 % line, smallest staging |

The trade-off: a lower cap means more textures wait, so surfaces stay blurry or untextured for longer
as you move. The effect of Balanced and Low on frame rate and memory **hasn't been measured yet**.

The **Frame cap** setting (`VICTORY_FRAME_INTERVAL_US`) is related. A cap of 30 or 60 lets the CPU and
GPU rest between frames, which lowers power and heat. It doesn't lower memory.

## System RAM

The game's own memory lives in system RAM, and the runtime adds staging buffers, copies of what
shaders wrote, and the shader and pipeline caches it loads. With 32 GB the runtime has reported
memory pressure: in one session it warned with 5.7 GB physically free. When that happens it limits
texture prefetching and idle upload storage. 32 GB is the tested and recommended amount.

## Descriptor views

Direct3D 12 caps a shader-visible view heap at 1,000,000 descriptors. The game addresses more: about
1.25 million in one logged session. Views past the cap read an empty slot. This doesn't cost memory,
but it is a likely cause of missing or wrong detail on some surfaces and eyes. Remapping the game's
view indices into the host heap is on the [roadmap](ROADMAP.md).

## What to check when memory runs out

1. **Diagnostics → Last run's log.** Look for `host memory pressure` or `the card has no room for more`.
2. Try **Memory → Balanced**, then **Low**.
3. Close other GPU-heavy programs. Browsers with hardware acceleration and other games count against
   the same budget.
4. Keep the shader and pipeline caches on an SSD. They are read at startup and as new effects appear.

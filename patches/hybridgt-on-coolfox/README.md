# HybridGt carried onto cool_fox (branch `hybridgt/on-coolfox`)

Base: codec-d/cool_fox `f4b48ad7` (= avatarmanz/VIctory-7900XTX 0a43566 + 6 commits; the
repository is no longer on GitHub; a local copy was used). On top: HybridGt's constant samplers,
two-wave LDS/permute lowerings, legacy geometry programs (translator and host stage), DexNom's
wave64 mask fix, the translator's one-descriptor-edge rule, the runtime's geometry-only
containers, device methods 0x218/0x1c8 from the native index, the 0x1c8 format memory and
rectangle stage. Left out because cool_fox has its own: integer-target blending, the bitcode
latch fixes, the ZTRX VALU lowerings, POPS (cool_fox has real ordered sections), the wave-wide
last form. Not carried yet: fxceless's texture thread (`VICTORY_HEAP_WORKER`).

`git apply hybridgt-on-cool_fox-f4b48ad7.patch` on a checkout of cool_fox `f4b48ad7`.

# NVIDIA and AMD files

[← Documentation](../../docs/README.md)

HybridGt doesn't carry NVIDIA's or AMD's files, and its launcher never downloads them. These scripts,
from [VIctory-NVIDIA](https://github.com/pepezin-ai/VIctory-NVIDIA) (pepezin-ai, GPL-3.0-or-later,
copied unchanged at `df2daafe`), fetch them from the vendors' own GitHub when **you** run them:

| Script | Fetches | Into | Used for |
|---|---|---|---|
| `Get-NVIDIA-DLSS.cmd` | NVIDIA Streamline 2.14.16: DLSS, DLSS Frame Generation, Reflex (about 70 MB) | `%LOCALAPPDATA%\victory\nvidia-sdk` | The *Performance* preset, and DLSS in Settings |
| `Get-AMD-FSR.cmd` | AMD FidelityFX SDK 2.4.0: the loader and FSR 3 frame generation (about 30 MB), each checked by SHA-256 | `%LOCALAPPDATA%\victory\amd-sdk\FidelityFX\bin` | FSR 3 frame generation on cards without DLSS Frame Generation |

After running one, press **Scan this PC** in the launcher's Settings: it finds both folders by
itself. Each vendor's licence is saved beside its files; read it before use.

Frame generation multiplies the frames shown, not the game's own speed. Controls answer at the real
frame rate, and generated frames show more artifacts the lower that rate is (see
[PERFORMANCE.md](../../docs/PERFORMANCE.md)).

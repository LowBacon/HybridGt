# Troubleshooting

[← README](../../README.md) · [Known issues](../KNOWN_ISSUES.md) · [GPU memory](../GPU_MEMORY.md)

Start with the launcher's **Diagnostics** page. It lists every requirement with how to fix it,
problems in the settings, the last setup's steps, and a summary of the last run's log: errors,
warnings, the most frequent messages, and how the run ended.

## Setup

| Symptom | Cause | What to do |
|---|---|---|
| INITIALIZE stops at *Dependencies* | A required piece is missing | Diagnostics says which one. Usually the Visual C++ 2015–2022 Redistributable (x64), a graphics driver without mesh shaders, or a Windows build older than 19041. |
| *Game files* is blocked | The folder isn't the game's | Choose the folder that contains `MicrosoftGame.Config` and `game_scarlett_master.exe`, or press **Scan this PC** |
| *Pipeline cache* is blocked | No session has recorded pipelines yet | Expected on a first install. Play once, then run INITIALIZE again. |
| *Shader cache* fails | The runtime isn't set, or the cache folder can't be written | Settings → Runtimes and folders. The cache needs about 20 GB free. |
| The scan finds nothing | The game is on a drive or folder the scan skipped | Set the folder by hand under Settings → Game |

## Starting the game

| Symptom | Cause | What to do |
|---|---|---|
| Black screen with sound, minutes long, on a first run | Thousands of pipelines are being made on first use | Wait it out once, then run INITIALIZE so the pipeline step makes them ahead of time |
| *Couldn't load the save* | The save belongs to another runtime's folder | Each preset keeps its own saves. Pick the preset whose saves folder holds the save. |
| The game closes during loading | The game's own 30-second hang watchdog, seen once on the earlier runtime | Start it again. If it repeats, send the Diagnostics summary. |
| PLAY does nothing | No runtime for the preset | Diagnostics → *Runtime*; set `victory.exe` under Settings, or build it (Settings → Build the runtime from source) |

## While playing

| Symptom | Cause | What to do |
|---|---|---|
| Blurry or wrongly coloured textures that sharpen later | Textures still uploading or re-uploading | Keep caches on an SSD. On 12 GB cards, try Memory → Balanced. See [GPU_MEMORY.md](../GPU_MEMORY.md). |
| Graphics memory full, stutter | The texture cache filling the card | Memory → Balanced or Low |
| The whole PC slows down | System RAM pressure | Close other programs. 32 GB is the tested amount. |
| CPU or GPU fans at full speed | The runtime runs as fast as it can | Settings → Frame cap → 30 |
| Mouths out of step with speech | Frames reach the screen late at low frame rates | Settings → Sound delay. Try 100–200 ms and adjust by ear. |
| Soft hair and faces | The game's temporal upscaler at low frame rates | Known; improves with frame rate |
| Dark pixels on water | Known issue | See [KNOWN_ISSUES.md](../KNOWN_ISSUES.md) |

## Reporting a problem

Open an issue with:

1. what you did and what you saw;
2. the launcher's version (bottom of the sidebar), the preset, and whether INITIALIZE had finished;
3. your GPU, its driver version, CPU, RAM and Windows build (the Performance page lists them);
4. the **Diagnostics** summary of the run's log.

Don't attach game files, shader dumps or screenshots of the game's own art. The log summary is enough
to start with.

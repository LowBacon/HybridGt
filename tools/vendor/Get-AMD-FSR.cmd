@echo off
rem HybridGt: downloads AMD's FSR 3 frame generation files (about 30 MB, FidelityFX SDK v2.4.0) from AMD's
rem own GitHub into %LOCALAPPDATA%\victory\amd-sdk\FidelityFX\bin, where HybridGt.exe's scan finds them,
rem each checked against its SHA-256. Run it once, yourself: the launcher downloads nothing. AMD's files,
rem under AMD's licence (saved beside them). The script is VIctory-NVIDIA's (pepezin-ai, GPL-3.0-or-later).
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0get-amd-fsr.ps1" -To "%LOCALAPPDATA%\victory\amd-sdk\FidelityFX\bin" %*
echo.
pause

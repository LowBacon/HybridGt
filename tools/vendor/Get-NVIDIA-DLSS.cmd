@echo off
rem HybridGt: downloads NVIDIA's DLSS and DLSS Frame Generation files (about 70 MB) from NVIDIA's own
rem GitHub into %LOCALAPPDATA%\victory\nvidia-sdk, where HybridGt.exe's scan finds them. Run it once,
rem yourself: the launcher downloads nothing. NVIDIA's files, under NVIDIA's licences (saved beside them).
rem The script is VIctory-NVIDIA's (pepezin-ai, GPL-3.0-or-later), unchanged.
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0get-nvidia-sdks.ps1" -To "%LOCALAPPDATA%\victory\nvidia-sdk" %*
echo.
pause

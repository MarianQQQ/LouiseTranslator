@echo off
title Louise Translator
if not exist "Louise Translator.exe" (
    echo [WARNING] "Louise Translator.exe" not found in root directory.
    echo Starting automated build...
    call build_release.bat
)

start "" "Louise Translator.exe"

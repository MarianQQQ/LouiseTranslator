@echo off
title Louise Translator [Release Build]
color 0A
echo ========================================================
echo   Compiling Louise Translator (Release)...
echo ========================================================
echo.

cargo build --release
if %ERRORLEVEL% neq 0 (
    color 0C
    echo.
    echo [ERROR] Compilation failed!
    pause
    exit /b %ERRORLEVEL%
)

echo.
echo [SUCCESS] Copying compiled binary to root directory...
copy /Y "target\release\LouiseTranslator.exe" "Louise Translator.exe"

echo.
echo ========================================================
echo   Done! Executable saved to:
echo   %CD%\Louise Translator.exe
echo ========================================================
echo.
pause

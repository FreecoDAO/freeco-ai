@echo off
REM FreEco.ai portable launcher — Windows
REM Run from a USB drive or any folder: all config/data stays alongside this
REM script (in .\data), so the drive is fully self-contained and portable
REM across machines.

setlocal
set "ROOT=%~dp0"
set "OPENFANG_HOME=%ROOT%data"
if "%OPENFANG_LISTEN%"=="" set "OPENFANG_LISTEN=127.0.0.1:4200"

REM ── Bundled local AI (fully offline) ────────────────────────────────────
REM If this drive carries Ollama + a model, use them from the drive: nothing
REM is downloaded and nothing is installed on the host machine. The model
REM lives in .\models, so the private AI travels with the USB stick.
set "OLLAMA_MODELS=%ROOT%models"
set "OLLAMA_EXE=%ROOT%ollama\windows\ollama.exe"
if exist "%OLLAMA_EXE%" (
    REM Only start it if nothing is already serving on 11434.
    >nul 2>&1 curl -s -m 2 http://127.0.0.1:11434/api/version
    if errorlevel 1 (
        echo   Starting bundled private AI from this drive...
        start "" /b "%OLLAMA_EXE%" serve
    ) else (
        echo   Using the Ollama already running on this machine.
    )
)

if /I "%PROCESSOR_ARCHITECTURE%"=="ARM64" (
    set "ARCH=aarch64"
) else (
    set "ARCH=x86_64"
)

set "EXE=%ROOT%bin\windows\%ARCH%\openfang.exe"
if not exist "%EXE%" (
    echo   Could not find a FreEco.ai binary for windows\%ARCH% at:
    echo     %EXE%
    pause
    exit /b 1
)

if not exist "%OPENFANG_HOME%" mkdir "%OPENFANG_HOME%"
if not exist "%OPENFANG_HOME%\config.toml" (
    echo   First run - starting FreEco.ai setup wizard...
    "%EXE%" init
)

echo.
echo   FreEco.ai starting - dashboard: http://%OPENFANG_LISTEN%
echo   Data directory: %OPENFANG_HOME%
echo   Close this window to stop FreEco.ai.
echo.

REM Open the dashboard a few seconds after launch, once the server has
REM had time to bind its port. Runs in a detached shell so it doesn't
REM block the foreground server below.
start "" cmd /c "timeout /t 3 >nul & start "" "http://%OPENFANG_LISTEN%""

"%EXE%" start

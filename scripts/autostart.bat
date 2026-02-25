@echo off
:: ClaudeMeter Auto-Start Setup
:: Run this script to add/remove ClaudeMeter from Windows startup
::
:: Usage:
::   autostart.bat add     - Add to startup
::   autostart.bat remove  - Remove from startup

set "APP_NAME=ClaudeMeter"
set "EXE_PATH=%~dp0claudemeter.exe"
set "REG_KEY=HKCU\SOFTWARE\Microsoft\Windows\CurrentVersion\Run"

if "%1"=="add" (
    reg add "%REG_KEY%" /v "%APP_NAME%" /t REG_SZ /d "\"%EXE_PATH%\"" /f
    echo ClaudeMeter added to Windows startup.
    echo    It will start automatically on next login.
    pause
    exit /b 0
)

if "%1"=="remove" (
    reg delete "%REG_KEY%" /v "%APP_NAME%" /f 2>nul
    echo ClaudeMeter removed from Windows startup.
    pause
    exit /b 0
)

echo Usage:
echo   %~nx0 add     - Add ClaudeMeter to Windows startup
echo   %~nx0 remove  - Remove ClaudeMeter from Windows startup
pause

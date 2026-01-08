@echo off
:: ═══════════════════════════════════════════════════════════════════════════
:: UNINSTALL.BAT - Ifá-Lang Windows Uninstaller
:: ═══════════════════════════════════════════════════════════════════════════

setlocal enabledelayedexpansion

echo.
echo  Ifá-Lang Uninstaller
echo  ═══════════════════════════════════════════════════════════════════════
echo.

set "INSTALL_DIR=C:\ifa-lang"

if not exist "%INSTALL_DIR%" (
    echo [INFO] Ifá-Lang is not installed at %INSTALL_DIR%
    pause
    exit /b 0
)

echo This will remove Ifá-Lang from your system.
echo Location: %INSTALL_DIR%
echo.
set /p CONFIRM="Are you sure? (y/n): "

if /i "%CONFIRM%" neq "y" (
    echo Cancelled.
    pause
    exit /b 0
)

echo.
echo Removing files...
rmdir /s /q "%INSTALL_DIR%" 2>nul

echo Removing from PATH...
:: Note: This removes from user PATH - system PATH requires admin
for /f "tokens=2*" %%a in ('reg query "HKCU\Environment" /v Path 2^>nul') do set "CURRENT_PATH=%%b"
if defined CURRENT_PATH (
    set "NEW_PATH=!CURRENT_PATH:%INSTALL_DIR%\bin;=!"
    set "NEW_PATH=!NEW_PATH:;%INSTALL_DIR%\bin=!"
    set "NEW_PATH=!NEW_PATH:%INSTALL_DIR%\bin=!"
    setx PATH "!NEW_PATH!" >nul 2>nul
)

echo.
echo  ═══════════════════════════════════════════════════════════════════════
echo       Ifá-Lang has been uninstalled.
echo  ═══════════════════════════════════════════════════════════════════════
echo.
echo Restart your terminal for PATH changes to take effect.
echo.

pause
endlocal

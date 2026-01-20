@echo off
:: ═══════════════════════════════════════════════════════════════════════════
:: INSTALL.BAT - Ifá-Lang Windows Installer (Rust Native)
:: ═══════════════════════════════════════════════════════════════════════════
:: Run as Administrator for best results
:: ═══════════════════════════════════════════════════════════════════════════

setlocal enabledelayedexpansion

echo.
echo  ═══════════════════════════════════════════════════════════════════════
echo       Ifá-Lang Installer v1.2.2
echo       The Yoruba Programming Language
echo  ═══════════════════════════════════════════════════════════════════════
echo.

:: Set installation directory
set "INSTALL_DIR=C:\ifa-lang"
set "SOURCE_DIR=%~dp0"

echo [1/3] Installing to %INSTALL_DIR%...

:: Create directory
if not exist "%INSTALL_DIR%" mkdir "%INSTALL_DIR%"
if not exist "%INSTALL_DIR%\bin" mkdir "%INSTALL_DIR%\bin"
if not exist "%INSTALL_DIR%\docs" mkdir "%INSTALL_DIR%\docs"
if not exist "%INSTALL_DIR%\examples" mkdir "%INSTALL_DIR%\examples"

:: Copy files
echo [2/3] Copying files...
copy "%SOURCE_DIR%bin\ifa.exe" "%INSTALL_DIR%\bin\" >nul
if %ERRORLEVEL% neq 0 (
    echo [ERROR] Failed to copy binaries. Are you running from the extracted folder?
    pause
    exit /b 1
)

xcopy "%SOURCE_DIR%docs\*" "%INSTALL_DIR%\docs\" /s /e /q /y >nul
xcopy "%SOURCE_DIR%examples\*" "%INSTALL_DIR%\examples\" /s /e /q /y >nul
copy "%SOURCE_DIR%README.md" "%INSTALL_DIR%\" >nul
copy "%SOURCE_DIR%DOCS.md" "%INSTALL_DIR%\" >nul
copy "%SOURCE_DIR%CHANGELOG.md" "%INSTALL_DIR%\" >nul
copy "%SOURCE_DIR%TUTORIAL.md" "%INSTALL_DIR%\" >nul
copy "%SOURCE_DIR%LICENSE" "%INSTALL_DIR%\" >nul

:: Add to PATH
echo [3/3] Adding to system PATH...
echo %PATH% | findstr /i /c:"%INSTALL_DIR%\bin" >nul
if %ERRORLEVEL% equ 0 (
    echo        Already in PATH!
) else (
    :: Add to user PATH
    setx PATH "%PATH%;%INSTALL_DIR%\bin" >nul
    echo        Added to PATH!
)

echo.
echo  ═══════════════════════════════════════════════════════════════════════
echo       Installation Complete!
echo  ═══════════════════════════════════════════════════════════════════════
echo.
echo   Location:   %INSTALL_DIR%
echo   Command:    ifa
echo.
echo   Try it:     ifa --version
echo.
echo   NOTE: Please restart your terminal to update PATH.
echo.
pause

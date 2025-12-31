@echo off
:: ═══════════════════════════════════════════════════════════════════════════
:: INSTALL.BAT - Ifá-Lang Windows Installer
:: The Yoruba Programming Language
:: ═══════════════════════════════════════════════════════════════════════════
:: Run as Administrator for best results
:: ═══════════════════════════════════════════════════════════════════════════

setlocal enabledelayedexpansion

echo.
echo  ═══════════════════════════════════════════════════════════════════════
echo       Ifá-Lang Installer v1.0.0
echo       The Yoruba Programming Language
echo  ═══════════════════════════════════════════════════════════════════════
echo.

:: Check for Python
echo [1/4] Checking Python installation...
where python >nul 2>nul
if %ERRORLEVEL% neq 0 (
    echo.
    echo [ERROR] Python not found!
    echo Please install Python 3.8+ from https://python.org
    echo Make sure to check "Add Python to PATH" during installation.
    echo.
    pause
    exit /b 1
)

for /f "tokens=2" %%i in ('python --version 2^>^&1') do set PYTHON_VERSION=%%i
echo        Found Python %PYTHON_VERSION%

:: Set installation directory
set "INSTALL_DIR=C:\ifa-lang"
set "SOURCE_DIR=%~dp0"

echo.
echo [2/4] Installing to %INSTALL_DIR%...

:: Create installation directory
if exist "%INSTALL_DIR%" (
    echo        Removing old installation...
    rmdir /s /q "%INSTALL_DIR%" 2>nul
)

mkdir "%INSTALL_DIR%"
mkdir "%INSTALL_DIR%\bin"
mkdir "%INSTALL_DIR%\src"
mkdir "%INSTALL_DIR%\lib"
mkdir "%INSTALL_DIR%\lib\std"
mkdir "%INSTALL_DIR%\lib\ext"
mkdir "%INSTALL_DIR%\examples"
mkdir "%INSTALL_DIR%\docs"

:: Copy files
echo        Copying core files...
xcopy "%SOURCE_DIR%bin\*" "%INSTALL_DIR%\bin\" /s /e /q /y >nul
xcopy "%SOURCE_DIR%src\*" "%INSTALL_DIR%\src\" /s /e /q /y >nul
xcopy "%SOURCE_DIR%lib\std\*" "%INSTALL_DIR%\lib\std\" /s /e /q /y >nul
xcopy "%SOURCE_DIR%lib\ext\*" "%INSTALL_DIR%\lib\ext\" /s /e /q /y >nul
xcopy "%SOURCE_DIR%examples\*" "%INSTALL_DIR%\examples\" /s /e /q /y >nul
xcopy "%SOURCE_DIR%docs\*" "%INSTALL_DIR%\docs\" /s /e /q /y >nul

:: Copy documentation
copy "%SOURCE_DIR%README.md" "%INSTALL_DIR%\" >nul 2>nul
copy "%SOURCE_DIR%DOCS.md" "%INSTALL_DIR%\" >nul 2>nul
copy "%SOURCE_DIR%TUTORIAL.md" "%INSTALL_DIR%\" >nul 2>nul
copy "%SOURCE_DIR%LICENSE" "%INSTALL_DIR%\" >nul 2>nul
copy "%SOURCE_DIR%requirements.txt" "%INSTALL_DIR%\" >nul 2>nul

echo        Done!

:: Install Python dependencies
echo.
echo [3/4] Installing Python dependencies...
python -m pip install --quiet --upgrade pip >nul 2>nul
python -m pip install --quiet -r "%INSTALL_DIR%\requirements.txt" >nul 2>nul
echo        Done!

:: Add to PATH
echo.
echo [4/4] Adding to system PATH...

:: Check if already in PATH
echo %PATH% | findstr /i /c:"%INSTALL_DIR%\bin" >nul
if %ERRORLEVEL% equ 0 (
    echo        Already in PATH!
) else (
    :: Add to user PATH (doesn't require admin)
    for /f "tokens=2*" %%a in ('reg query "HKCU\Environment" /v Path 2^>nul') do set "CURRENT_PATH=%%b"
    if defined CURRENT_PATH (
        setx PATH "%CURRENT_PATH%;%INSTALL_DIR%\bin" >nul 2>nul
    ) else (
        setx PATH "%INSTALL_DIR%\bin" >nul 2>nul
    )
    echo        Added to user PATH!
)

:: Verify installation
echo.
echo  ═══════════════════════════════════════════════════════════════════════
echo       Installation Complete!
echo  ═══════════════════════════════════════════════════════════════════════
echo.
echo   Location:   %INSTALL_DIR%
echo   Command:    ifa
echo.
echo   IMPORTANT: Restart your terminal or run:
echo              refreshenv
echo.
echo   Quick Start:
echo       ifa --help              Show all commands
echo       ifa run hello.ifa      Run an Ifá program
echo       ifa repl               Start interactive REPL
echo       ifa bytecode app.ifa   Compile to bytecode
echo       ifa build app.ifa      Compile to native binary
echo.
echo   VS Code Extension:
echo       Search "Ifá-Lang" in VS Code Extensions
echo.
echo   Àṣẹ! (It is done!)
echo.

pause
endlocal

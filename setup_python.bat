@echo off
:: ═══════════════════════════════════════════════════════════════════════════
:: SETUP_PYTHON.BAT - Download and setup embedded Python for local development
:: Run this once before building the installer locally
:: ═══════════════════════════════════════════════════════════════════════════

setlocal enabledelayedexpansion

set PYTHON_VERSION=3.11.7
set PYTHON_URL=https://www.python.org/ftp/python/%PYTHON_VERSION%/python-%PYTHON_VERSION%-embed-amd64.zip
set PYTHON_DIR=%~dp0python

echo.
echo  Setting up Embedded Python for Ifa-Lang
echo  ═══════════════════════════════════════════════════════════════════════
echo.

:: Check if already exists
if exist "%PYTHON_DIR%\python.exe" (
    echo [INFO] Python already exists at %PYTHON_DIR%
    echo        Delete the python folder to re-download.
    pause
    exit /b 0
)

:: Download Python embeddable
echo [1/4] Downloading Python %PYTHON_VERSION% embeddable...
powershell -Command "Invoke-WebRequest -Uri '%PYTHON_URL%' -OutFile 'python-embed.zip'"

if not exist "python-embed.zip" (
    echo [ERROR] Failed to download Python embeddable.
    pause
    exit /b 1
)

:: Extract
echo [2/4] Extracting...
powershell -Command "Expand-Archive -Path 'python-embed.zip' -DestinationPath '%PYTHON_DIR%' -Force"
del python-embed.zip

:: Remove _pth file to enable pip
echo [3/4] Configuring for pip...
del "%PYTHON_DIR%\python311._pth" 2>nul

:: Install pip
echo [4/4] Installing pip and dependencies...
powershell -Command "Invoke-WebRequest -Uri 'https://bootstrap.pypa.io/get-pip.py' -OutFile '%PYTHON_DIR%\get-pip.py'"
"%PYTHON_DIR%\python.exe" "%PYTHON_DIR%\get-pip.py" --quiet
del "%PYTHON_DIR%\get-pip.py"

:: Install requirements
"%PYTHON_DIR%\python.exe" -m pip install --quiet -r requirements.txt

echo.
echo  ═══════════════════════════════════════════════════════════════════════
echo       Python %PYTHON_VERSION% embedded is ready!
echo  ═══════════════════════════════════════════════════════════════════════
echo.
echo   Location: %PYTHON_DIR%
echo.
echo   You can now build the installer with Inno Setup.
echo.

pause
endlocal

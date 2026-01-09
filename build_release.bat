@echo off
:: ═══════════════════════════════════════════════════════════════════════════
:: BUILD_RELEASE.BAT - Create Ifá-Lang Release Package (Rust Native)
:: ═══════════════════════════════════════════════════════════════════════════
:: Generates: ifa-lang-v1.2.0-windows-x86_64.zip
:: ═══════════════════════════════════════════════════════════════════════════

setlocal enabledelayedexpansion

set VERSION=1.2.1
set RELEASE_NAME=ifa-lang-v%VERSION%-windows-x86_64
set SOURCE_DIR=%~dp0
set BUILD_DIR=%SOURCE_DIR%dist\%RELEASE_NAME%
set OUTPUT_ZIP=%SOURCE_DIR%dist\%RELEASE_NAME%.zip

echo.
echo  Building Ifá-Lang Release Package v%VERSION% (Native)
echo  ═══════════════════════════════════════════════════════════════════════
echo.

:: Check for Rust
where cargo >nul 2>nul
if %ERRORLEVEL% neq 0 (
    echo [ERROR] Rust/Cargo not found! Please install Rust.
    pause
    exit /b 1
)

:: Build Binary
echo [1/5] Compiling release binary...
cargo build --release -p ifa-cli
if %ERRORLEVEL% neq 0 (
    echo [ERROR] Build failed!
    pause
    exit /b 1
)

:: Create dist directory
echo [2/5] Creating build directory...
if exist "%SOURCE_DIR%dist" rmdir /s /q "%SOURCE_DIR%dist"
mkdir "%BUILD_DIR%"

:: Create structure
mkdir "%BUILD_DIR%\bin"
mkdir "%BUILD_DIR%\docs"
mkdir "%BUILD_DIR%\examples"
mkdir "%BUILD_DIR%\examples\advanced"

:: Copy files
echo [3/5] Copying files...

:: Binary
copy "%SOURCE_DIR%target\release\ifa.exe" "%BUILD_DIR%\bin\" >nul
if %ERRORLEVEL% neq 0 (
    echo [ERROR] Could not find ifa.exe in target\release!
    pause
    exit /b 1
)

:: Docs
copy "%SOURCE_DIR%README.md" "%BUILD_DIR%\" >nul
copy "%SOURCE_DIR%DOCS.md" "%BUILD_DIR%\" >nul
copy "%SOURCE_DIR%CHANGELOG.md" "%BUILD_DIR%\" >nul
copy "%SOURCE_DIR%TUTORIAL.md" "%BUILD_DIR%\" >nul
copy "%SOURCE_DIR%LICENSE" "%BUILD_DIR%\" >nul
xcopy "%SOURCE_DIR%docs\*" "%BUILD_DIR%\docs\" /s /e /q /y >nul

:: Examples
copy "%SOURCE_DIR%examples\min_*.ifa" "%BUILD_DIR%\examples\" >nul
copy "%SOURCE_DIR%examples\05_advanced\*.ifa" "%BUILD_DIR%\examples\advanced\" >nul

:: Installers
copy "%SOURCE_DIR%install.bat" "%BUILD_DIR%\" >nul
copy "%SOURCE_DIR%uninstall.bat" "%BUILD_DIR%\" >nul

:: Create ZIP
echo [4/5] Creating ZIP archive...
where powershell >nul 2>nul
if %ERRORLEVEL% equ 0 (
    powershell -Command "Compress-Archive -Path '%BUILD_DIR%\*' -DestinationPath '%OUTPUT_ZIP%' -Force"
) else (
    echo [WARNING] PowerShell not found. Please manually zip: %BUILD_DIR%
)

echo.
echo  ═══════════════════════════════════════════════════════════════════════
echo       Build Complete!
echo  ═══════════════════════════════════════════════════════════════════════
echo.
echo   Package: %OUTPUT_ZIP%
echo.
echo   To distribute:
echo   1. Upload ZIP to GitHub Releases
echo   2. Users download, extract, and run install.bat
echo.

pause
endlocal

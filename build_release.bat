@echo off
:: ═══════════════════════════════════════════════════════════════════════════
:: BUILD_RELEASE.BAT - Create Ifá-Lang Release Package
:: ═══════════════════════════════════════════════════════════════════════════
:: Generates: ifa-lang-v1.0.0-windows.zip
:: ═══════════════════════════════════════════════════════════════════════════

setlocal enabledelayedexpansion

set VERSION=1.0.0
set RELEASE_NAME=ifa-lang-v%VERSION%-windows
set SOURCE_DIR=%~dp0
set BUILD_DIR=%SOURCE_DIR%dist\%RELEASE_NAME%
set OUTPUT_ZIP=%SOURCE_DIR%dist\%RELEASE_NAME%.zip

echo.
echo  Building Ifá-Lang Release Package v%VERSION%
echo  ═══════════════════════════════════════════════════════════════════════
echo.

:: Create dist directory
echo [1/5] Creating build directory...
if exist "%SOURCE_DIR%dist" rmdir /s /q "%SOURCE_DIR%dist"
mkdir "%BUILD_DIR%"

:: Create structure
echo [2/5] Creating directory structure...
mkdir "%BUILD_DIR%\bin"
mkdir "%BUILD_DIR%\src"
mkdir "%BUILD_DIR%\lib\std"
mkdir "%BUILD_DIR%\lib\ext"
mkdir "%BUILD_DIR%\examples"
mkdir "%BUILD_DIR%\docs"

:: Copy files
echo [3/5] Copying files...

:: Core
xcopy "%SOURCE_DIR%bin\*" "%BUILD_DIR%\bin\" /s /e /q /y >nul
xcopy "%SOURCE_DIR%src\*.py" "%BUILD_DIR%\src\" /s /e /q /y >nul

:: Libraries
xcopy "%SOURCE_DIR%lib\std\*.py" "%BUILD_DIR%\lib\std\" /s /e /q /y >nul
xcopy "%SOURCE_DIR%lib\ext\*.py" "%BUILD_DIR%\lib\ext\" /s /e /q /y >nul
copy "%SOURCE_DIR%lib\Cargo.toml" "%BUILD_DIR%\lib\" >nul 2>nul
copy "%SOURCE_DIR%lib\core.rs" "%BUILD_DIR%\lib\" >nul 2>nul

:: Examples (only .ifa files and README)
for /r "%SOURCE_DIR%examples" %%f in (*.ifa) do (
    set "SUBDIR=%%~dpf"
    set "SUBDIR=!SUBDIR:%SOURCE_DIR%examples=!"
    if not exist "%BUILD_DIR%\examples!SUBDIR!" mkdir "%BUILD_DIR%\examples!SUBDIR!"
    copy "%%f" "%BUILD_DIR%\examples!SUBDIR!" >nul
)
copy "%SOURCE_DIR%examples\README.md" "%BUILD_DIR%\examples\" >nul 2>nul

:: Docs
xcopy "%SOURCE_DIR%docs\*" "%BUILD_DIR%\docs\" /s /e /q /y >nul

:: Root files
copy "%SOURCE_DIR%README.md" "%BUILD_DIR%\" >nul
copy "%SOURCE_DIR%DOCS.md" "%BUILD_DIR%\" >nul
copy "%SOURCE_DIR%TUTORIAL.md" "%BUILD_DIR%\" >nul
copy "%SOURCE_DIR%LICENSE" "%BUILD_DIR%\" >nul
copy "%SOURCE_DIR%requirements.txt" "%BUILD_DIR%\" >nul
copy "%SOURCE_DIR%install.bat" "%BUILD_DIR%\" >nul
copy "%SOURCE_DIR%uninstall.bat" "%BUILD_DIR%\" >nul
copy "%SOURCE_DIR%install.sh" "%BUILD_DIR%\" >nul
copy "%SOURCE_DIR%uninstall.sh" "%BUILD_DIR%\" >nul

:: Clean up unwanted files
echo [4/5] Cleaning up...
del /s /q "%BUILD_DIR%\*.pyc" >nul 2>nul
for /d /r "%BUILD_DIR%" %%d in (__pycache__) do @if exist "%%d" rmdir /s /q "%%d"
for /d /r "%BUILD_DIR%" %%d in (.git) do @if exist "%%d" rmdir /s /q "%%d"

:: Create ZIP
echo [5/5] Creating ZIP archive...
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
echo   Size:    
for %%A in ("%OUTPUT_ZIP%") do echo            %%~zA bytes
echo.
echo   To distribute:
echo   1. Upload ZIP to GitHub Releases
echo   2. Users download, extract, and run install.bat
echo.

pause
endlocal

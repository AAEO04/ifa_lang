@echo off
:: ═══════════════════════════════════════════════════════════════════════════
:: IFA.BAT - Ifá-Lang Command Line Interface
:: The Yoruba Programming Language
:: Uses bundled Python (no system Python required)
:: ═══════════════════════════════════════════════════════════════════════════

setlocal enabledelayedexpansion

:: Get the directory where this script is located
set "IFA_HOME=%~dp0.."
set "IFA_SRC=%IFA_HOME%\src"
set "IFA_LIB=%IFA_HOME%\lib"
set "IFA_PYTHON=%IFA_HOME%\python"

:: Set PYTHONPATH to include our lib directory
set "PYTHONPATH=%IFA_HOME%;%PYTHONPATH%"

:: Check if bundled Python exists (preferred)
if exist "%IFA_PYTHON%\python.exe" (
    "%IFA_PYTHON%\python.exe" "%IFA_SRC%\cli.py" %*
    goto :end
)

:: Fallback to system Python
where python >nul 2>nul
if %ERRORLEVEL% equ 0 (
    python "%IFA_SRC%\cli.py" %*
    goto :end
)

:: No Python found
echo [ERROR] Python not found!
echo.
echo The bundled Python is missing and no system Python was found.
echo Please reinstall Ifá-Lang or install Python 3.8+ from https://python.org
exit /b 1

:end
endlocal

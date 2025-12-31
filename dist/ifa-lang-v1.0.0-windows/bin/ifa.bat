@echo off
:: ═══════════════════════════════════════════════════════════════════════════
:: IFA.BAT - Ifá-Lang Command Line Interface
:: The Yoruba Programming Language
:: ═══════════════════════════════════════════════════════════════════════════

setlocal enabledelayedexpansion

:: Get the directory where this script is located
set "IFA_HOME=%~dp0.."
set "IFA_SRC=%IFA_HOME%\src"
set "IFA_LIB=%IFA_HOME%\lib"

:: Check if Python is available
where python >nul 2>nul
if %ERRORLEVEL% neq 0 (
    echo [ERROR] Python not found in PATH.
    echo Please install Python 3.8+ from https://python.org
    exit /b 1
)

:: Set PYTHONPATH to include our lib directory
set "PYTHONPATH=%IFA_HOME%;%PYTHONPATH%"

:: Forward all arguments to the CLI
python "%IFA_SRC%\cli.py" %*

endlocal

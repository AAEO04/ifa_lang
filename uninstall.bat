@echo off
set "INSTALL_DIR=C:\ifa-lang"

echo.
echo  Uninstalling Ifá-Lang...
echo.

if exist "%INSTALL_DIR%" (
    rmdir /s /q "%INSTALL_DIR%"
    echo  Removed %INSTALL_DIR%
) else (
    echo  %INSTALL_DIR% not found.
)

echo.
echo  NOTE: Please manually remove C:\ifa-lang\bin from your PATH environment variable.
echo.
pause

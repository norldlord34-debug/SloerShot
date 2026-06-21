@echo off
setlocal
cd /d "%~dp0"
set "EXE=%~dp0target\release\shotcli.exe"
if not exist "%EXE%" set "EXE=%~dp0target\debug\shotcli.exe"
if not exist "%EXE%" (
 echo No se encontro shotcli.exe. Compila con: cargo build -p shotcli --release
 pause
 exit /b 1
)
echo ===============================================
echo SloerShot - demo del core (CLI)
echo ===============================================
echo.
echo [1/3] Version del core:
"%EXE%" version
echo.
echo [2/3] Generando artefactos en assets\out ...
"%EXE%" demo --out "%~dp0assets\out"
echo.
echo [3/3] Parity (25 modulos, incluye QR real):
"%EXE%" parity
echo.
echo Abriendo la carpeta de resultados (PNG anotado, beautified, GIF) ...
start "" "%~dp0assets\out"
echo.
echo Listo. Pulsa una tecla para cerrar.
pause >nul

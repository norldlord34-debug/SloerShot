@echo off
call "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat" >nul 2>&1
set "SWIFT=C:\Users\jhons\AppData\Local\Programs\Swift"
set "PATH=%SWIFT%\Toolchains\6.3.2+Asserts\usr\bin;%SWIFT%\Runtimes\6.3.2\usr\bin;%PATH%"
set "SDKROOT=%SWIFT%\Platforms\6.3.2\Windows.platform\Developer\SDKs\Windows.sdk"
cd /d C:\Users\jhons\Downloads\SloerShot\apps\swift-smoke
echo === swiftc compile ===
swiftc main.swift -I CShotCore -L lib -lshotcore -o smoke.exe
if errorlevel 1 ( echo SWIFTC_FAILED & exit /b 1 )
echo === run smoke.exe ===
smoke.exe
exit /b %errorlevel%

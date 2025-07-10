@echo off
echo Starting My Launcher in debug mode...
set RUST_LOG=debug
set RUST_BACKTRACE=1

REM Run the debug build
target\debug\my-launcher.exe

REM If the app exits immediately, pause to see any error messages
if %ERRORLEVEL% neq 0 (
    echo.
    echo Application exited with error code: %ERRORLEVEL%
    pause
)
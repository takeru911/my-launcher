@echo off
echo Building and running without transparency...
set RUST_LOG=debug
set RUST_BACKTRACE=1
set NO_TRANSPARENT=1

REM Build in debug mode
cargo build

REM Run the debug build
target\debug\my-launcher.exe

if %ERRORLEVEL% neq 0 (
    echo.
    echo Application exited with error code: %ERRORLEVEL%
    pause
)
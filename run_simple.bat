@echo off
echo Starting My Launcher with simple settings...
set RUST_LOG=debug
set RUST_BACKTRACE=1
set NO_TRANSPARENT=1

REM Build in debug mode
cargo build --target x86_64-pc-windows-gnu

if %ERRORLEVEL% equ 0 (
    echo Build successful!
    echo.
    echo Starting launcher with simple window settings...
    
    REM Run the debug build
    target\x86_64-pc-windows-gnu\debug\my-launcher.exe
    
    echo.
    echo Exit code: %ERRORLEVEL%
    pause
) else (
    echo Build failed!
    pause
)
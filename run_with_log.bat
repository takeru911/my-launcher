@echo off
echo Starting My Launcher with file logging...

REM Set environment variables
set RUST_LOG=debug
set RUST_BACKTRACE=1

REM Build in release mode
echo Building release version...
cargo build --release --target x86_64-pc-windows-gnu

if %ERRORLEVEL% equ 0 (
    echo Build successful!
    echo.
    echo Starting launcher...
    echo Check logs folder in the same directory as the executable.
    echo.
    
    REM Run the release build
    target\x86_64-pc-windows-gnu\release\my-launcher.exe
    
    if %ERRORLEVEL% neq 0 (
        echo.
        echo Application exited with error code: %ERRORLEVEL%
        pause
    )
) else (
    echo Build failed!
    pause
)
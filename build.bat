@echo off
echo Building My Launcher for Windows...

echo.
echo Building debug version...
cargo build --target x86_64-pc-windows-msvc

echo.
echo Building release version...
cargo build --release --target x86_64-pc-windows-msvc

echo.
echo Build complete!
echo Debug binary: target\x86_64-pc-windows-msvc\debug\my-launcher.exe
echo Release binary: target\x86_64-pc-windows-msvc\release\my-launcher.exe

pause
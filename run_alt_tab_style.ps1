# PowerShell script to run the launcher in Alt+Tab style
Write-Host "Starting My Launcher in Alt+Tab style..." -ForegroundColor Green

# Set environment variables
$env:RUST_LOG = "debug"
$env:RUST_BACKTRACE = "1"

# Build in release mode for better performance
Write-Host "Building in release mode..." -ForegroundColor Yellow
cargo build --release

if ($LASTEXITCODE -eq 0) {
    Write-Host "Build successful! Starting launcher..." -ForegroundColor Green
    
    # Run the release build
    .\target\release\my-launcher.exe
    
    if ($LASTEXITCODE -ne 0) {
        Write-Host "Application exited with error code: $LASTEXITCODE" -ForegroundColor Red
        Read-Host "Press Enter to continue..."
    }
} else {
    Write-Host "Build failed!" -ForegroundColor Red
    Read-Host "Press Enter to continue..."
}
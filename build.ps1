# Windows用ビルドスクリプト
Write-Host "Building My Launcher for Windows..." -ForegroundColor Green

# デバッグビルド
Write-Host "`nBuilding debug version..." -ForegroundColor Yellow
cargo build --target x86_64-pc-windows-msvc

# リリースビルド
Write-Host "`nBuilding release version..." -ForegroundColor Yellow
cargo build --release --target x86_64-pc-windows-msvc

# 出力先
Write-Host "`nBuild complete!" -ForegroundColor Green
Write-Host "Debug binary: target\x86_64-pc-windows-msvc\debug\my-launcher.exe"
Write-Host "Release binary: target\x86_64-pc-windows-msvc\release\my-launcher.exe"

# サイズ確認
if (Test-Path "target\x86_64-pc-windows-msvc\release\my-launcher.exe") {
    $size = (Get-Item "target\x86_64-pc-windows-msvc\release\my-launcher.exe").Length / 1MB
    Write-Host "`nRelease binary size: $([math]::Round($size, 2)) MB" -ForegroundColor Cyan
}
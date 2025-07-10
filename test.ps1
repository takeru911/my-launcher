# Windows用テストスクリプト
Write-Host "Running My Launcher Tests..." -ForegroundColor Green

# 環境変数の設定（詳細出力）
$env:RUST_BACKTRACE = "1"

# ユニットテストを実行
Write-Host "`n=== Running Unit Tests ===" -ForegroundColor Yellow
cargo test --lib --verbose

# 統合テストを実行
Write-Host "`n=== Running Integration Tests ===" -ForegroundColor Yellow
cargo test --test integration_test --verbose

# 全テストを実行（サマリー）
Write-Host "`n=== Running All Tests (Summary) ===" -ForegroundColor Yellow
cargo test

# テストカバレッジ（tarpaulinがインストールされている場合）
if (Get-Command cargo-tarpaulin -ErrorAction SilentlyContinue) {
    Write-Host "`n=== Generating Test Coverage ===" -ForegroundColor Yellow
    cargo tarpaulin --out Html --output-dir target/coverage
    Write-Host "Coverage report generated at: target/coverage/tarpaulin-report.html" -ForegroundColor Cyan
}

Write-Host "`nTest run complete!" -ForegroundColor Green
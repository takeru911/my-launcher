@echo off
echo Running My Launcher Tests...

REM 環境変数の設定
set RUST_BACKTRACE=1

echo.
echo === Running Unit Tests ===
cargo test --lib --verbose

echo.
echo === Running Integration Tests ===
cargo test --test integration_test --verbose

echo.
echo === Running All Tests (Summary) ===
cargo test

echo.
echo Test run complete!
pause
# クイックセットアップガイド

## PowerShell実行ポリシーエラーの解決

「デジタル署名されていません」エラーが出た場合、以下のいずれかの方法を使用してください：

### 方法1: バッチファイルを使用（推奨）

コマンドプロンプトを**管理者権限**で開いて実行：

```cmd
cd C:\path\to\my-launcher
install-native-host-debug.bat angillmdfhlmokmnkmckjidojkjbampc
```

### 方法2: PowerShellの実行ポリシーを一時的に変更

PowerShellを**管理者権限**で開いて：

```powershell
# 現在のポリシーを確認
Get-ExecutionPolicy

# 一時的にポリシーを変更
Set-ExecutionPolicy -ExecutionPolicy Bypass -Scope Process

# スクリプトを実行
cd C:\path\to\my-launcher
.\install-native-host-debug.ps1 -ExtensionId angillmdfhlmokmnkmckjidojkjbampc
```

### 方法3: PowerShellコマンドを直接実行

管理者権限のPowerShellで：

```powershell
cd C:\path\to\my-launcher
powershell -ExecutionPolicy Bypass -File .\install-native-host-debug.ps1 -ExtensionId angillmdfhlmokmnkmckjidojkjbampc
```

## インストール後の確認

1. **Chromeを完全に再起動**
   - すべてのChromeウィンドウを閉じる
   - タスクマネージャーでchrome.exeプロセスがないことを確認

2. **拡張機能のコンソールを確認**
   - `chrome://extensions/` を開く
   - 「My Launcher Tab Connector」の「バックグラウンドページ」をクリック
   - コンソールでエラーがないか確認

3. **My Launcherを実行**
   ```cmd
   cd C:\path\to\my-launcher
   target\x86_64-pc-windows-gnu\debug\my-launcher.exe
   ```

4. **動作確認**
   - Tabキーでブラウザモードに切り替え
   - 文字を入力するとChromeタブが表示されるはず

## トラブルシューティング

### エラーが続く場合

1. **レジストリを確認**:
   ```cmd
   reg query "HKLM\SOFTWARE\Google\Chrome\NativeMessagingHosts\com.mylauncher.tabconnector"
   reg query "HKCU\SOFTWARE\Google\Chrome\NativeMessagingHosts\com.mylauncher.tabconnector"
   ```

2. **マニフェストファイルを確認**:
   ```cmd
   type native-host-manifest-installed.json
   ```

3. **実行ファイルの存在を確認**:
   ```cmd
   dir target\x86_64-pc-windows-gnu\debug\my-launcher-native-host.exe
   ```
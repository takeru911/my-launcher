# Native Host Setup Guide

## エラー: "Specified native messaging host not found"

このエラーは、Chrome拡張機能がNative Messaging Hostを見つけられない場合に発生します。

## 解決手順

### 1. Chrome拡張機能IDの確認
1. Chromeで `chrome://extensions/` を開く
2. 「My Launcher Tab Connector」拡張機能を探す
3. 「デベロッパーモード」を有効にする
4. 拡張機能のIDをコピー（例: `abcdefghijklmnopqrstuvwxyz123456`）

### 2. Native Hostのインストール

PowerShellを**管理者権限**で開いて、以下のコマンドを実行：

```powershell
# プロジェクトディレクトリに移動
cd C:\path\to\my-launcher

# インストールスクリプトを実行（YOUR_EXTENSION_IDを実際のIDに置き換える）
.\install-native-host-debug.ps1 -ExtensionId YOUR_EXTENSION_ID
```

例：
```powershell
.\install-native-host-debug.ps1 -ExtensionId abcdefghijklmnopqrstuvwxyz123456
```

### 3. 確認事項

インストール後、以下を確認：

1. **レジストリの確認**（PowerShellで実行）:
   ```powershell
   Get-ItemProperty "HKLM:\SOFTWARE\Google\Chrome\NativeMessagingHosts\com.mylauncher.tabconnector"
   # または
   Get-ItemProperty "HKCU:\SOFTWARE\Google\Chrome\NativeMessagingHosts\com.mylauncher.tabconnector"
   ```

2. **マニフェストファイルの確認**:
   - `native-host-manifest-installed.json` が作成されている
   - `path` が正しい実行ファイルを指している
   - `allowed_origins` に正しい拡張機能IDが含まれている

3. **実行ファイルの確認**:
   ```powershell
   Test-Path ".\target\x86_64-pc-windows-gnu\debug\my-launcher-native-host.exe"
   ```

### 4. Chromeの再起動

インストール後、必ずChromeを完全に再起動してください：
1. すべてのChromeウィンドウを閉じる
2. タスクマネージャーで `chrome.exe` プロセスがないことを確認
3. Chromeを再起動

### 5. デバッグ

Chrome拡張機能のバックグラウンドページでコンソールを確認：
1. `chrome://extensions/` を開く
2. 「My Launcher Tab Connector」の「バックグラウンドページ」をクリック
3. コンソールタブでエラーメッセージを確認

### トラブルシューティング

#### まだエラーが出る場合：

1. **パスの確認**:
   ```powershell
   # native-host-manifest-installed.json の内容を確認
   Get-Content .\native-host-manifest-installed.json
   ```

2. **権限の確認**:
   - 実行ファイルに実行権限があるか
   - ファイアウォールがブロックしていないか

3. **別の場所にインストール**:
   ```powershell
   # カスタムパスを指定
   .\install-native-host-debug.ps1 -ExtensionId YOUR_ID -HostPath "C:\MyLauncher\my-launcher-native-host.exe"
   ```

4. **ログの確認**:
   環境変数 `RUST_LOG=debug` を設定して実行：
   ```cmd
   set RUST_LOG=debug
   my-launcher-native-host.exe
   ```

### よくある問題

1. **拡張機能IDが変わった**:
   - 拡張機能を再インストールするとIDが変わります
   - 新しいIDで再度インストールスクリプトを実行

2. **パスにスペースが含まれる**:
   - スペースを含むパスは問題を起こすことがあります
   - スペースのないパスに移動することを推奨

3. **アンチウイルスソフト**:
   - 一部のアンチウイルスがNative Messagingをブロックすることがあります
   - 一時的に無効にして確認

## 動作確認

1. My Launcherを起動
2. Tabキーでブラウザモードに切り替え
3. 何か入力すると、Chromeタブが表示されるはず
4. Chrome拡張機能のコンソールに "Received tab list from Chrome" が表示される
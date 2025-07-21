# My Launcher

カスタムランチャーアプリケーション for Windows

## セットアップ

### 1. Rustのインストール

Windows環境でRustをインストールする方法：

1. [rustup.rs](https://rustup.rs/) から rustup-init.exe をダウンロード
2. 実行してデフォルト設定でインストール
3. コマンドプロンプトを再起動

または、PowerShellで以下を実行：
```powershell
winget install Rustlang.Rust.MSVC
```

### 2. ビルドと実行

```bash
# 開発モードで実行
cargo run

# リリースビルド
cargo build --release
```

## 機能

- **デュアルモード検索**: ブラウザモード（Web検索）とWindowsモード（ウィンドウ切り替え）
- **Chrome統合**: ブックマーク、履歴、開いているタブの検索と切り替え
- **高速ウィンドウ切り替え**: Alt+Tabスタイルのサムネイル表示
- **ビジュアルフィードバック**: タブ切り替え時のステータス表示

## 使い方

- テキストを入力して検索
- `Tab` キーでモード切り替え（Browser ⇔ Windows）
- `↑` `↓` キーで結果を選択
- `Enter` で実行
- `Esc` で終了

### 入力パターン

**Browserモード:**
- 任意のテキスト - Google検索、Chromeブックマーク、履歴、タブを検索
- 選択してEnterでWebページやタブに移動

**Windowsモード:**
- 空の入力 - すべてのウィンドウを表示
- テキスト入力 - ウィンドウタイトルやプロセス名で絞り込み

### Chrome Tab機能の有効化

Chrome拡張機能をインストールすることで、開いているタブの検索と切り替えが可能になります。
詳細は `chrome-extension/README.md` を参照してください。

## トラブルシューティング

### Windows環境での注意事項

1. Windows Defenderがビルドをブロックする場合があります
2. 初回ビルド時は依存関係のダウンロードに時間がかかります
3. `cargo run` で起動しない場合は、`target/debug/my-launcher.exe` を直接実行してみてください
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

## 使い方

- テキストを入力して検索
- `↑` `↓` キーで結果を選択
- `Enter` で実行
- `Esc` で終了

### 入力パターン

- `http://` or `https://` - URLを開く
- `>command` - コマンドを実行（例：`>notepad`）
- その他 - Google検索

## トラブルシューティング

### Windows環境での注意事項

1. Windows Defenderがビルドをブロックする場合があります
2. 初回ビルド時は依存関係のダウンロードに時間がかかります
3. `cargo run` で起動しない場合は、`target/debug/my-launcher.exe` を直接実行してみてください
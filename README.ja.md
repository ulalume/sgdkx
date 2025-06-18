# SGDKTool

**このツールは開発中です。圧倒的にテストが足りていません。ご利用の際はご注意ください。IssueやPRは歓迎です。**

SGDKTool は、SGDK（Sega Genesis Development Kit）を用いた開発を支援する CLI ツールです。

---

## 1. インストール方法

### SGDKTool のインストール（cargo）

```sh
cargo install --git https://github.com/ulalume/sgdktool
```

### 必要なツールのインストール（macOS）

以下のツールが必要です。Homebrew でインストールできます。

```sh
brew install make openjdk compiledb

brew tap gcenx/wine
brew install --cask --no-quarantine wine-crossover
```

- `git` は多くの場合プリインストールされていますが、必要に応じて `brew install git` でインストールしてください。
- コマンドなしで実行すると、環境チェックが動作します。必要なツールがインストールされているか確認できます。

---

## 2. 使い方

主なコマンドは以下の通りです。

- `sgdktool` : 環境チェックとヘルプ
- `sgdktool setup` : SGDK のセットアップ（クローンとパス登録）
- `sgdktool new` : SGDK テンプレートから新規プロジェクト作成
- `sgdktool make` : SGDK プロジェクトのビルド
- `sgdktool uninstall` : SGDK のアンインストールと設定削除

### 簡単な使い方例

```sh
sgdktool setup
sgdktool new your_project
cd your_project
sgdktool make
```

### 参考: コマンドなしで実行した場合の出力例

```
SGDKサポートCLIツール

Usage: sgdktool [COMMAND]

Commands:
  setup      SGDKをセットアップ（クローンとパス登録）
  new        SGDKテンプレートから新しいプロジェクトを作成
  make       makeを使ってプロジェクトをビルド
  uninstall  SGDKインストールと設定をアンインストール
  help       Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version

🩺 SGDKTool 環境チェック
✅ git: /opt/homebrew/bin/git
✅ make: /usr/bin/make
✅ java: /opt/homebrew/opt/openjdk/bin/java
✅ compiledb: /opt/homebrew/bin/compiledb
✅ wine: /opt/homebrew/bin/wine

📝 SGDK 設定情報:
SGDK パス   : /Users/kbt/Library/Application Support/sgdktool/SGDK
ブランチ     : master
コミット ID : 60c99ea912387d6f5f014673d9760ef8a79e1339
```

---

## 3. 謝辞・依存プロジェクト

- [SGDK (by Stephane-D)](https://github.com/Stephane-D/SGDK)
- [SGDK_wine (by Franticware)](https://github.com/Franticware/SGDK_wine)

これらの素晴らしいプロジェクトに感謝します。

---

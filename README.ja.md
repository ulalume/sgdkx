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

- `sgdktool`
  環境チェック・SGDKやエミュレータの設定状況・ヘルプを表示します。
  サブコマンドを指定しない場合、現在のSGDK/エミュレータのセットアップ状況を確認できます。

- `sgdktool setup [--dir パス] [--version バージョン]`
  SGDK（Sega Genesis Development Kit）をダウンロード・インストールします。
  `--dir` でインストール先ディレクトリ（省略時は設定ディレクトリ）、`--version` でブランチ名・タグ名・コミットID（省略時はmaster）を指定できます。
  例:
    - `--version develop` で developブランチ
    - `--version V2.11` でタグ V2.11
    - `--version ef9292c0` でコミットID ef9292c0
  SGDKのパスやバージョンはconfig.tomlに保存されます。

- `sgdktool setup-emu [gens|blastem] [--dir パス]`
  エミュレータ（gens または blastem）をダウンロード・セットアップします。
  `--dir` でインストール先を指定できます（省略時はデフォルトの設定ディレクトリ）。
  インストールしたエミュレータのパスはconfig.tomlに保存されます。

- `sgdktool new <プロジェクト名>`
  SGDKテンプレートから新しいプロジェクトを作成します。
  `<プロジェクト名>` という名前のディレクトリが作成され、その中にプロジェクトが生成されます。

- `sgdktool make [--project ディレクトリ] [追加オプション...]`
  `make` を使ってSGDKプロジェクトをビルドします。
  `--project` でプロジェクトディレクトリ（省略時はカレントディレクトリ）、追加オプションでmakeに渡す引数を指定できます。

- `sgdktool run [--emulator gens|blastem] [--rom パス]`
  エミュレータでROMファイルを実行します。
  `--emulator` でエミュレータ（gens または blastem）、`--rom` でROMファイルのパスを指定できます（どちらも省略可能、デフォルトは自動検出/`out/rom.bin`）。
  エミュレータが未インストールの場合はsetup-emuの実行を促すメッセージが表示されます。

- `sgdktool uninstall [--config-only]`
  SGDKのアンインストールと設定ファイルの削除を行います。
  また、`setup-emu` でインストールしたエミュレータ（gens/blastem）も、config.tomlに記載されたパスを参照して削除されます。

### 簡単な使い方例

```sh
sgdktool setup --version v2.11 # stable
sgdktool setup-emu
sgdktool new your_project
cd your_project
sgdktool make
sgdktool run
```

### 参考: コマンドなしで実行した場合の出力例

```
SGDKサポートCLIツール

Usage: sgdktool [COMMAND]

Commands:
  setup      SGDKをセットアップ（クローンとパス登録）
  setup-emu  ROMファイル実行用のエミュレータをセットアップ
  new        SGDKテンプレートから新しいプロジェクトを作成
  make       makeを使ってプロジェクトをビルド
  run        エミュレータでROMファイルを実行
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
SGDK パス   : /Users/[user]/Library/Application Support/sgdktool/SGDK
バージョン  : master
コミット ID : 60c99ea912387d6f5f014673d9760ef8a79e1339
Gens パス   : /Users/[user]/Library/Application Support/sgdktool/gens/gens.exe
blastem パス: /Users/[user]/Library/Application Support/sgdktool/blastem/blastem-win64-0.6.3-pre-b42f00a3a937/blastem.exe
```

---

## 3. 謝辞・依存プロジェクト

- [SGDK (by Stephane-D)](https://github.com/Stephane-D/SGDK)
- [SGDK_wine (by Franticware)](https://github.com/Franticware/SGDK_wine)

これらの素晴らしいプロジェクトに感謝します。

---

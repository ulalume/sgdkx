# SGDKTool

**このツールは開発中です。圧倒的にテストが足りていません。ご利用の際はご注意ください。Issue や PR は歓迎です。**

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

brew install doxygen # options
```

- `git` は多くの場合プリインストールされていますが、必要に応じて `brew install git` でインストールしてください。
- コマンドなしで実行すると、環境チェックが動作します。必要なツールがインストールされているか確認できます。

---

## 2. 使い方

主なコマンドは以下の通りです。

- `sgdktool`<br>
  環境チェック・SGDK やエミュレータの設定状況・ヘルプを表示します。

- `sgdktool setup [--version バージョン]`<br>
  SGDK（Sega Genesis Development Kit）をダウンロード・インストールします。<br>
  `--version` でブランチ名・タグ名・コミット ID（省略時は master）を指定できます。<br>
  例:
  - `--version V2.11` でタグ V2.11
  - `--version ef9292c0` でコミット ID ef9292c0<br>
    SGDK のパスやバージョンは config.toml に保存されます。<br>
    さらに、**doxygen がインストールされていて SGDK ドキュメントが存在しない場合は、自動的にドキュメントを生成します。**

- `sgdktool setup-emu [gens|blastem]`<br>
  エミュレータ（gens または blastem）をダウンロード・セットアップします。<br>
  インストールしたエミュレータのパスは config.toml に保存されます。

- `sgdktool new <プロジェクト名>`<br>
  SGDK サンプルから新しいプロジェクトを作成します。<br>

- `sgdktool run [--emulator gens|blastem] [--rom パス]`
  エミュレータで ROM ファイルを実行します。<br>
  `--emulator` でエミュレータ（gens または blastem）、`--rom` で ROM ファイルのパスを指定できます（どちらも省略可能、デフォルトは自動検出/`out/rom.bin`）。

- `sgdktool uninstall [--config-only]`
  SGDK のアンインストールと設定ファイルの削除を行います。<br>
  また、`setup-emu` でインストールしたエミュレータ（gens/blastem）も、config.toml に記載されたパスを参照して削除されます。

- `sgdktool doc`
  SGDK ドキュメントが存在すればブラウザで開きます。

---

#### 実験的な機能

- `sgdktool web-export [--rom <パス>] [--dir <親ディレクトリ>]`
  **【実験的】** ROMファイルとWebエミュレータ用テンプレートをエクスポートします。<br>
  このコマンドはWebエミュレータのテンプレート（HTML/JS/WASM）とROMを指定ディレクトリ配下の `web-export` ディレクトリにコピーします。<br>
  生成されたディレクトリをWebサーバで公開することで、ブラウザ上でゲームを動かせます。

- `sgdktool web-server [--dir <ディレクトリ>] [--port <ポート>]`
  **【実験的】** `web-export` ディレクトリを組み込みHTTPサーバで公開します（WASM対応のCOOP/COEPヘッダ付き）。<br>
  デフォルトでは `web-export` ディレクトリを `localhost:8080` で公開します。<br>
  ディレクトリやポートはオプションで変更できます。<br>
  例: `sgdktool web-server --dir web-export --port 9000`

### 簡単な使い方例

```sh
sgdktool setup --version v2.11 # stable
sgdktool setup-emu
sgdktool new your_project
cd your_project
make
sgdktool run
```

### 参考: コマンドなしで実行した場合の出力例

```
A CLI tool for SGDK-based development

Usage: sgdktool [COMMAND]

Commands:
  setup       Setup SGDK for development
  doc         Show SGDK documentation status
  setup-emu   Setup emulator for running ROM files
  new         Create a new SGDK project
  run         Run ROM file with emulator
  uninstall   Uninstall SGDK installation and configuration
  web-export  Export ROM and web emulator template for web deployment
  web-server  Serve web-export directory with HTTP server (with COOP/COEP headers)
  open        Open SGDK installation directory
  setup-web   Setup web export template
  help        Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version

🩺 SGDKTool Environment Check
✅ git: /opt/homebrew/bin/git
✅ make: /usr/bin/make
✅ java: /opt/homebrew/opt/openjdk/bin/java
✅ compiledb: /opt/homebrew/bin/compiledb
✅ doxygen: /opt/homebrew/bin/doxygen
✅ wine: /opt/homebrew/bin/wine

📝 SGDKTool Configuration: /Users/[user]/.sgdktool/data/config.toml
SGDK Path   : /Users/[user]/.sgdktool/data/SGDK
Version     : v2.11
Commit ID   : ef9292c03fe33a2f8af3a2589ab856a53dcef35c
Gens Path   : /Users/[user]/.sgdktool/data/gens/gens.exe
blastem Path: Not installed

📄 SGDK documentation: /Users/[user]/.sgdktool/data/SGDK/doc/html/index.html
```

---

## 3. 謝辞・依存プロジェクト

- [SGDK (by Stephane-D)](https://github.com/Stephane-D/SGDK)
- [SGDK_wine (by Franticware)](https://github.com/Franticware/SGDK_wine)
- [jgenesis (by James Groth)](https://github.com/jsgroth/jgenesis)

これらの素晴らしいプロジェクトに感謝します。

---

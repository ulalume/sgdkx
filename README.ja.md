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

- `sgdktool setup [--dir パス] [--version バージョン]`<br>
  SGDK（Sega Genesis Development Kit）をダウンロード・インストールします。<br>
  `--dir` でインストール先ディレクトリ（省略時は設定ディレクトリ）、`--version` でブランチ名・タグ名・コミット ID（省略時は master）を指定できます。<br>
  例:

  - `--version V2.11` でタグ V2.11
  - `--version ef9292c0` でコミット ID ef9292c0<br>
    SGDK のパスやバージョンは config.toml に保存されます。<br>
    さらに、**doxygen がインストールされていて SGDK ドキュメントが存在しない場合は、自動的にドキュメントを生成します。**

- `sgdktool setup-emu [gens|blastem] [--dir パス]`<br>
  エミュレータ（gens または blastem）をダウンロード・セットアップします。<br>
  `--dir` でインストール先を指定できます（省略時はデフォルトの設定ディレクトリ）。<br>
  インストールしたエミュレータのパスは config.toml に保存されます。

- `sgdktool new <プロジェクト名>`<br>
  SGDK サンプルから新しいプロジェクトを作成します。<br>

- `sgdktool make [--project ディレクトリ] [追加オプション...]`<br>
  `make` を使って SGDK プロジェクトをビルドします。<br>
  `--project` でプロジェクトディレクトリ（省略時はカレントディレクトリ）、追加オプションで make に渡す引数を指定できます。

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
sgdktool make
sgdktool run
```

### 参考: コマンドなしで実行した場合の出力例

```
SGDKサポートCLIツール

Usage: sgdktool [COMMAND]

Commands:
  setup       SGDKをセットアップ（クローンとパス登録）
  setup-emu   ROMファイル実行用のエミュレータをセットアップ
  new         SGDKサンプルから新しいプロジェクトを作成
  make        makeを使ってプロジェクトをビルド
  run         エミュレータでROMファイルを実行
  doc         SGDKドキュメントが存在すれば開く
  web-export  ROMとWebエミュレータテンプレートをエクスポート
  web-server  web-exportディレクトリをHTTPサーバで公開（COOP/COEPヘッダ付き）
  uninstall   SGDKインストールと設定をアンインストール
  help        Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version

🩺 SGDKTool 環境チェック
✅ git: /opt/homebrew/bin/git
✅ make: /usr/bin/make
✅ java: /opt/homebrew/opt/openjdk/bin/java
✅ compiledb: /opt/homebrew/bin/compiledb
✅ doxygen: /opt/homebrew/bin/doxygen
✅ wine: /opt/homebrew/bin/wine

📝 SGDKToolの設定ファイル: /Users/[user]/Library/Application Support/sgdktool/config.toml
SGDK パス   : /Users/[user]/Library/Application Support/sgdktool/SGDK
バージョン  : master
コミット ID : 2c27b80443db8ad7e803cf2eb919b2122d458fae
Gens パス   : 未インストール
blastem パス: 未インストール

📄 SGDKドキュメント: /Users/[user]/Library/Application Support/sgdktool/SGDK/doc/html/index.html
```

---

## 3. 謝辞・依存プロジェクト

- [SGDK (by Stephane-D)](https://github.com/Stephane-D/SGDK)
- [SGDK_wine (by Franticware)](https://github.com/Franticware/SGDK_wine)
- [jgenesis (by James Groth)](https://github.com/jsgroth/jgenesis)

これらの素晴らしいプロジェクトに感謝します。

---
